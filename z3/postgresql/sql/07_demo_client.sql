\set ON_ERROR_STOP on
CREATE EXTENSION IF NOT EXISTS hstore;
SET search_path TO faculty, public;

-- ---------------------------------------------------------------------
-- Таблица описания пунктов демо-меню (текстовое меню без внешних языков)
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS faculty.demo_menu_items (
    code smallint PRIMARY KEY,
    title text NOT NULL
);
TRUNCATE faculty.demo_menu_items;
INSERT INTO faculty.demo_menu_items(code, title) VALUES
    (1,  'Средняя оценка по предмету для группы'),
    (2,  'Средняя оценка по предмету для направления'),
    (3,  'Количество отличников в группе'),
    (4,  'Есть ли у студента несданные экзамены'),
    (5,  'Пропуски по каждому студенту'),
    (6,  'Пропуски по каждой группе'),
    (7,  'Пропуски по каждому направлению'),
    (8,  'Пропуски по каждому преподавателю'),
    (9,  'Агрегаты: среднее группы по предмету (таблица)'),
    (10, 'Агрегаты: среднее группы по всем предметам (таблица)'),
    (11, 'Агрегаты: среднее по предмету глобально (таблица)'),
    (12, 'Числовой доп. параметр: среднее и сумма по группе'),
    (13, 'Текстовый доп. параметр: поиск значения (детально)'),
    (14, 'Тест триггера: недопустимая оценка (должен быть EXCEPTION)'),
    (15, 'Тест триггера: оценка "чужим" преподавателем (должен быть EXCEPTION)');

-- ---------------------------------------------------------------------
-- Функция вывода меню
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.demo_menu() RETURNS SETOF faculty.demo_menu_items
LANGUAGE sql STABLE
AS $$
    SELECT * FROM faculty.demo_menu_items ORDER BY code;
$$;

-- Показать меню:
-- SELECT * FROM faculty.demo_menu();

-- ---------------------------------------------------------------------
-- Универсальная функция-диспетчер демо-клиента.
-- p_code       — номер пункта меню (см. faculty.demo_menu())
-- p_arg_int    — числовой параметр (id группы/направления/студента/предмета), если нужен
-- p_arg_int2   — второй числовой параметр (например subject_id при p_arg_int = group_id)
-- p_arg_text   — текстовый параметр (например строка поиска, имя параметра)
-- Возвращает результат как json, чтобы единообразно выводить любые запросы.
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.demo_run(
    p_code integer,
    p_arg_int bigint DEFAULT NULL,
    p_arg_int2 integer DEFAULT NULL,
    p_arg_text text DEFAULT NULL
) RETURNS jsonb
LANGUAGE plpgsql
AS $$
DECLARE
    v_result jsonb;
BEGIN
    CASE p_code
    WHEN 1 THEN
        SELECT jsonb_build_object('avg_grade', faculty.avg_grade_group_subject(p_arg_int::integer, p_arg_int2))
        INTO v_result;
    WHEN 2 THEN
        SELECT jsonb_build_object('avg_grade', faculty.avg_grade_direction_subject(p_arg_int::integer, p_arg_int2))
        INTO v_result;
    WHEN 3 THEN
        SELECT jsonb_build_object('excellent_count', faculty.count_excellent_students(p_arg_int::integer))
        INTO v_result;
    WHEN 4 THEN
        SELECT jsonb_build_object('student_id', p_arg_int, 'has_failed_exams', faculty.has_failed_exams(p_arg_int))
        INTO v_result;
    WHEN 5 THEN
        SELECT jsonb_agg(to_jsonb(t)) FROM faculty.missed_lessons_by_student() t INTO v_result;
    WHEN 6 THEN
        SELECT jsonb_agg(to_jsonb(t)) FROM faculty.missed_lessons_by_group() t INTO v_result;
    WHEN 7 THEN
        SELECT jsonb_agg(to_jsonb(t)) FROM faculty.missed_lessons_by_direction() t INTO v_result;
    WHEN 8 THEN
        SELECT jsonb_agg(to_jsonb(t)) FROM faculty.missed_lessons_by_teacher() t INTO v_result;
    WHEN 9 THEN
        SELECT jsonb_agg(to_jsonb(t)) FROM faculty.group_subject_avg_grades t INTO v_result;
    WHEN 10 THEN
        SELECT jsonb_agg(to_jsonb(t)) FROM faculty.group_overall_avg_grades t INTO v_result;
    WHEN 11 THEN
        SELECT jsonb_agg(to_jsonb(t)) FROM faculty.subject_avg_grades t INTO v_result;
    WHEN 12 THEN
        SELECT jsonb_agg(to_jsonb(t)) FROM faculty.numeric_param_stats(p_arg_int::integer, p_arg_text) t INTO v_result;
    WHEN 13 THEN
        SELECT jsonb_agg(to_jsonb(t)) FROM faculty.search_text_param_detailed(p_arg_text) t INTO v_result;
    WHEN 14 THEN
        BEGIN
            INSERT INTO faculty.grades(enrollment_id, direction_subject_id, grade)
            VALUES (p_arg_int, p_arg_int2, 1);
            v_result := jsonb_build_object('status', 'unexpected_success');
        EXCEPTION WHEN OTHERS THEN
            v_result := jsonb_build_object('status', 'expected_exception', 'message', SQLERRM);
        END;
    WHEN 15 THEN
        BEGIN
            UPDATE faculty.grades
            SET grade = 5, graded_by_teacher_id = p_arg_int2
            WHERE id = p_arg_int;
            v_result := jsonb_build_object('status', 'unexpected_success');
        EXCEPTION WHEN OTHERS THEN
            v_result := jsonb_build_object('status', 'expected_exception', 'message', SQLERRM);
        END;
    ELSE
        v_result := jsonb_build_object('error', format('Пункт меню %s не найден', p_code));
    END CASE;

    RETURN coalesce(v_result, '[]'::jsonb);
END;
$$;

COMMENT ON FUNCTION faculty.demo_run(integer, bigint, integer, text)
    IS 'Демонстрационный клиент: выполняет выбранный пункт меню (см. faculty.demo_menu())';