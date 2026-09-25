\set ON_ERROR_STOP on
BEGIN;
SET search_path TO faculty, public;


-- 3.1 Справочник допустимых доп. параметров для направления (специальности).
--     Например, для "Программной инженерии" — параметр "Язык программирования"
--     (текстовый), для "Экономики" — "Стипендия" (числовой) и т.д.
CREATE TABLE IF NOT EXISTS faculty.param_types (
    id smallint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    code varchar(20) NOT NULL UNIQUE  -- 'numeric' | 'text'
);
INSERT INTO faculty.param_types(code) VALUES ('numeric'), ('text')
ON CONFLICT (code) DO NOTHING;

CREATE TABLE IF NOT EXISTS faculty.direction_param_defs (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    direction_id integer NOT NULL REFERENCES faculty.directions(id),
    param_name varchar(150) NOT NULL,
    param_type_id smallint NOT NULL REFERENCES faculty.param_types(id),
    UNIQUE (direction_id, param_name)
);
COMMENT ON TABLE faculty.direction_param_defs
    IS 'Список доп. параметров, доступных для направления (специальности)';

-- 3.2 Привязка параметра к КОНКРЕТНОЙ группе — какие параметры из
--     доступных по направлению эта группа фактически ведёт
--     (позволяет у каждой группы свой индивидуальный набор параметров).
CREATE TABLE IF NOT EXISTS faculty.group_param_defs (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    group_id integer NOT NULL REFERENCES faculty.student_groups(id),
    param_def_id integer NOT NULL REFERENCES faculty.direction_param_defs(id),
    UNIQUE (group_id, param_def_id)
);
COMMENT ON TABLE faculty.group_param_defs
    IS 'Индивидуальный список доп. параметров, включённых у конкретной группы';

-- 3.3 Значения параметров по студентам (числовые и текстовые — раздельно,
--     чтобы иметь типобезопасные значения и не терять точность/сортировку).
CREATE TABLE IF NOT EXISTS faculty.student_param_values (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    group_param_def_id integer NOT NULL REFERENCES faculty.group_param_defs(id),
    student_id bigint NOT NULL REFERENCES faculty.students(id) ON DELETE CASCADE,
    value_numeric numeric,
    value_text text,
    UNIQUE (group_param_def_id, student_id)
);
COMMENT ON TABLE faculty.student_param_values
    IS 'Значения доп. параметров студентов; используется только одна из колонок value_numeric/value_text согласно типу параметра';

-- Триггер: значение должно соответствовать заявленному типу параметра
CREATE OR REPLACE FUNCTION faculty.trg_check_param_value_type() RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    v_type_code varchar(20);
BEGIN
    SELECT pt.code INTO v_type_code
    FROM faculty.group_param_defs gpd
    JOIN faculty.direction_param_defs dpd ON dpd.id = gpd.param_def_id
    JOIN faculty.param_types pt ON pt.id = dpd.param_type_id
    WHERE gpd.id = NEW.group_param_def_id;

    IF v_type_code = 'numeric' AND NEW.value_text IS NOT NULL THEN
        RAISE EXCEPTION 'Параметр числовой — value_text должен быть NULL';
    END IF;
    IF v_type_code = 'text' AND NEW.value_numeric IS NOT NULL THEN
        RAISE EXCEPTION 'Параметр текстовый — value_numeric должен быть NULL';
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_check_param_value_type ON faculty.student_param_values;
CREATE TRIGGER trg_check_param_value_type
    BEFORE INSERT OR UPDATE ON faculty.student_param_values
    FOR EACH ROW
    EXECUTE FUNCTION faculty.trg_check_param_value_type();

-- ---------------------------------------------------------------------
-- 3.4 Функция: среднее и суммарное значение по числовому параметру
--     для конкретной группы.
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.numeric_param_stats(
    p_group_id integer,
    p_param_name varchar
) RETURNS TABLE (
    avg_value numeric,
    sum_value numeric,
    values_count integer
)
LANGUAGE sql
STABLE
AS $$
    SELECT round(avg(spv.value_numeric), 2), sum(spv.value_numeric), count(spv.value_numeric)::integer
    FROM faculty.group_param_defs gpd
    JOIN faculty.direction_param_defs dpd ON dpd.id = gpd.param_def_id
    JOIN faculty.param_types pt ON pt.id = dpd.param_type_id
    JOIN faculty.student_param_values spv ON spv.group_param_def_id = gpd.id
    WHERE gpd.group_id = p_group_id
      AND dpd.param_name = p_param_name
      AND pt.code = 'numeric';
$$;

COMMENT ON FUNCTION faculty.numeric_param_stats(integer, varchar)
    IS 'Среднее и суммарное значение числового доп. параметра для группы';


-- ---------------------------------------------------------------------
-- 3.5 Функция текстового поиска по текстовым доп. параметрам.
--     Возвращает ассоциативный массив: student_id -> найденное значение.
--     Параметры: искомая подстрока/значение (обязательный),
--                имя параметра (опционально — фильтр по параметру),
--                группа (опционально — фильтр по группе),
--                студент (опционально — фильтр по студенту).
--     Дополнительно возвращаем "человекочитаемую" таблицу-детализацию
--     (параметр, группа, студент, значение) отдельной функцией ниже —
--     согласно заданию "с указанием, в каком параметре, в какой группе,
--     у какого человека найдено значение".
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.search_text_param(
    p_search_value text,
    p_param_name varchar DEFAULT NULL,
    p_group_id integer DEFAULT NULL,
    p_student_id bigint DEFAULT NULL
) RETURNS hstore
LANGUAGE sql
STABLE
AS $$
    SELECT coalesce(hstore(array_agg(spv.student_id::text), array_agg(spv.value_text)), hstore(''))
    FROM faculty.student_param_values spv
    JOIN faculty.group_param_defs gpd ON gpd.id = spv.group_param_def_id
    JOIN faculty.direction_param_defs dpd ON dpd.id = gpd.param_def_id
    JOIN faculty.param_types pt ON pt.id = dpd.param_type_id
    WHERE pt.code = 'text'
      AND spv.value_text ILIKE '%' || p_search_value || '%'
      AND (p_param_name IS NULL OR dpd.param_name = p_param_name)
      AND (p_group_id IS NULL OR gpd.group_id = p_group_id)
      AND (p_student_id IS NULL OR spv.student_id = p_student_id);
$$;

COMMENT ON FUNCTION faculty.search_text_param(text, varchar, integer, bigint)
    IS 'Текстовый поиск по значению текстового доп. параметра; результат — ассоциативный массив student_id -> найденное значение (hstore)';

-- Детализирующая версия — та же выборка, но с указанием параметра,
-- группы и студента для каждой находки (более наглядно для демонстрации).
CREATE OR REPLACE FUNCTION faculty.search_text_param_detailed(
    p_search_value text,
    p_param_name varchar DEFAULT NULL,
    p_group_id integer DEFAULT NULL,
    p_student_id bigint DEFAULT NULL
) RETURNS TABLE (
    student_id bigint,
    student_name text,
    group_number text,
    param_name text,
    found_value text
)
LANGUAGE sql
STABLE
AS $$
    SELECT spv.student_id,
           concat_ws(' ', st.surname, st.first_name, st.patronymic),
           g.group_number,
           dpd.param_name,
           spv.value_text
    FROM faculty.student_param_values spv
    JOIN faculty.group_param_defs gpd ON gpd.id = spv.group_param_def_id
    JOIN faculty.direction_param_defs dpd ON dpd.id = gpd.param_def_id
    JOIN faculty.param_types pt ON pt.id = dpd.param_type_id
    JOIN faculty.student_groups g ON g.id = gpd.group_id
    JOIN faculty.students st ON st.id = spv.student_id
    WHERE pt.code = 'text'
      AND spv.value_text ILIKE '%' || p_search_value || '%'
      AND (p_param_name IS NULL OR dpd.param_name = p_param_name)
      AND (p_group_id IS NULL OR gpd.group_id = p_group_id)
      AND (p_student_id IS NULL OR spv.student_id = p_student_id)
    ORDER BY g.group_number, st.surname;
$$;

COMMENT ON FUNCTION faculty.search_text_param_detailed(text, varchar, integer, bigint)
    IS 'Текстовый поиск по значению текстового доп. параметра с указанием параметра, группы и студента для каждой находки';

COMMIT;