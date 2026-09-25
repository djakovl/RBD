
\set ON_ERROR_STOP on
BEGIN;
SET search_path TO faculty, public;

-- ---------------------------------------------------------------------
-- 2.1 Проверка корректности выставляемой оценки: допустимы 2,3,4,5 (или NULL)
--     В схеме уже есть CHECK (grade BETWEEN 2 AND 5), но по заданию нужен
--     именно триггер — делаем его явно, с понятным сообщением об ошибке.
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.trg_check_grade_value() RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.grade IS NOT NULL AND NEW.grade NOT IN (2,3,4,5) THEN
        RAISE EXCEPTION 'Недопустимая оценка %: разрешены только 2, 3, 4, 5 или NULL', NEW.grade;
    END IF;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_check_grade_value ON faculty.grades;
CREATE TRIGGER trg_check_grade_value
    BEFORE INSERT OR UPDATE OF grade ON faculty.grades
    FOR EACH ROW
    EXECUTE FUNCTION faculty.trg_check_grade_value();

COMMENT ON FUNCTION faculty.trg_check_grade_value()
    IS 'Триггер: допустимые значения оценки — 2,3,4,5 либо NULL';

-- ---------------------------------------------------------------------
-- 2.2 Проверка корректности выставляемой оценки преподавателем:
--     оценку может ставить только тот преподаватель, который назначен
--     на данный предмет у данной группы (направления группы).
--     Т.к. в таблице grades нет столбца "кто поставил", добавляем его,
--     чтобы триггер имел смысл (по условию задания).
-- ---------------------------------------------------------------------
ALTER TABLE faculty.grades
    ADD COLUMN IF NOT EXISTS graded_by_teacher_id integer REFERENCES faculty.teachers(id);

CREATE OR REPLACE FUNCTION faculty.trg_check_teacher_assignment() RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    v_assigned_teacher integer;
BEGIN
    IF NEW.graded_by_teacher_id IS NULL THEN
        RETURN NEW; -- если преподаватель не указан явно, проверку не делаем
    END IF;

    SELECT ds.teacher_id INTO v_assigned_teacher
    FROM faculty.direction_subjects ds
    WHERE ds.id = NEW.direction_subject_id;

    IF v_assigned_teacher IS NULL THEN
        RAISE EXCEPTION 'Не найдено назначение предмета % на направление', NEW.direction_subject_id;
    END IF;

    IF v_assigned_teacher <> NEW.graded_by_teacher_id THEN
        RAISE EXCEPTION
            'Преподаватель % не назначен на предмет (direction_subject_id=%) — оценку может выставить только преподаватель %',
            NEW.graded_by_teacher_id, NEW.direction_subject_id, v_assigned_teacher;
    END IF;

    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_check_teacher_assignment ON faculty.grades;
CREATE TRIGGER trg_check_teacher_assignment
    BEFORE INSERT OR UPDATE OF grade, graded_by_teacher_id ON faculty.grades
    FOR EACH ROW
    EXECUTE FUNCTION faculty.trg_check_teacher_assignment();

COMMENT ON FUNCTION faculty.trg_check_teacher_assignment()
    IS 'Триггер: оценку может выставить только преподаватель, назначенный на предмет у направления группы';

-- ---------------------------------------------------------------------
-- Вспомогательные таблицы-агрегаты для триггеров 2.3-2.5
-- ---------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS faculty.group_subject_avg_grades (
    group_id integer NOT NULL REFERENCES faculty.student_groups(id),
    direction_subject_id integer NOT NULL REFERENCES faculty.direction_subjects(id),
    avg_grade numeric,
    grades_count integer NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (group_id, direction_subject_id)
);
COMMENT ON TABLE faculty.group_subject_avg_grades
    IS 'Средняя оценка группы по конкретному предмету (обновляется триггером после INSERT/UPDATE в grades)';

CREATE TABLE IF NOT EXISTS faculty.group_overall_avg_grades (
    group_id integer PRIMARY KEY REFERENCES faculty.student_groups(id),
    avg_grade numeric,
    grades_count integer NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now()
);
COMMENT ON TABLE faculty.group_overall_avg_grades
    IS 'Средняя оценка группы по всем предметам (обновляется триггером после INSERT/UPDATE в grades)';

CREATE TABLE IF NOT EXISTS faculty.subject_avg_grades (
    direction_subject_id integer PRIMARY KEY REFERENCES faculty.direction_subjects(id),
    avg_grade numeric,
    grades_count integer NOT NULL DEFAULT 0,
    updated_at timestamptz NOT NULL DEFAULT now()
);
COMMENT ON TABLE faculty.subject_avg_grades
    IS 'Средняя оценка по предмету (across всех направлений/групп, назначенных на этот direction_subject_id) — обновляется триггером';

-- ---------------------------------------------------------------------
-- 2.3 AFTER INSERT/UPDATE на grades: пересчитать среднее группы ПО ЭТОМУ предмету
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.trg_update_group_subject_avg() RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    v_group_id integer;
    v_avg numeric;
    v_cnt integer;
BEGIN
    SELECT e.group_id INTO v_group_id
    FROM faculty.enrollments e
    WHERE e.id = NEW.enrollment_id;

    SELECT round(avg(gr.grade), 2), count(gr.grade)
    INTO v_avg, v_cnt
    FROM faculty.enrollments e
    JOIN faculty.grades gr ON gr.enrollment_id = e.id
    WHERE e.group_id = v_group_id
      AND gr.direction_subject_id = NEW.direction_subject_id
      AND gr.grade IS NOT NULL;

    INSERT INTO faculty.group_subject_avg_grades(group_id, direction_subject_id, avg_grade, grades_count, updated_at)
    VALUES (v_group_id, NEW.direction_subject_id, v_avg, coalesce(v_cnt, 0), now())
    ON CONFLICT (group_id, direction_subject_id)
    DO UPDATE SET avg_grade = EXCLUDED.avg_grade,
                  grades_count = EXCLUDED.grades_count,
                  updated_at = now();

    RETURN NULL; -- AFTER-триггер, возвращаемое значение игнорируется
END;
$$;

DROP TRIGGER IF EXISTS trg_update_group_subject_avg ON faculty.grades;
CREATE TRIGGER trg_update_group_subject_avg
    AFTER INSERT OR UPDATE OF grade ON faculty.grades
    FOR EACH ROW
    EXECUTE FUNCTION faculty.trg_update_group_subject_avg();

COMMENT ON FUNCTION faculty.trg_update_group_subject_avg()
    IS 'Триггер: после внесения оценки пересчитывает среднюю оценку группы по этому предмету';

-- ---------------------------------------------------------------------
-- 2.4 AFTER INSERT/UPDATE на grades: пересчитать общее среднее группы (по всем оценкам)
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.trg_update_group_overall_avg() RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    v_group_id integer;
    v_avg numeric;
    v_cnt integer;
BEGIN
    SELECT e.group_id INTO v_group_id
    FROM faculty.enrollments e
    WHERE e.id = NEW.enrollment_id;

    SELECT round(avg(gr.grade), 2), count(gr.grade)
    INTO v_avg, v_cnt
    FROM faculty.enrollments e
    JOIN faculty.grades gr ON gr.enrollment_id = e.id
    WHERE e.group_id = v_group_id
      AND gr.grade IS NOT NULL;

    INSERT INTO faculty.group_overall_avg_grades(group_id, avg_grade, grades_count, updated_at)
    VALUES (v_group_id, v_avg, coalesce(v_cnt, 0), now())
    ON CONFLICT (group_id)
    DO UPDATE SET avg_grade = EXCLUDED.avg_grade,
                  grades_count = EXCLUDED.grades_count,
                  updated_at = now();

    RETURN NULL;
END;
$$;

DROP TRIGGER IF EXISTS trg_update_group_overall_avg ON faculty.grades;
CREATE TRIGGER trg_update_group_overall_avg
    AFTER INSERT OR UPDATE OF grade ON faculty.grades
    FOR EACH ROW
    EXECUTE FUNCTION faculty.trg_update_group_overall_avg();

COMMENT ON FUNCTION faculty.trg_update_group_overall_avg()
    IS 'Триггер: после внесения оценки пересчитывает общую среднюю оценку группы по всем предметам';

-- ---------------------------------------------------------------------
-- 2.5 AFTER INSERT/UPDATE на grades: пересчитать среднее по предмету
--     (глобально, по всем группам/направлениям, использующим этот direction_subject_id)
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.trg_update_subject_avg() RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    v_avg numeric;
    v_cnt integer;
BEGIN
    SELECT round(avg(gr.grade), 2), count(gr.grade)
    INTO v_avg, v_cnt
    FROM faculty.grades gr
    WHERE gr.direction_subject_id = NEW.direction_subject_id
      AND gr.grade IS NOT NULL;

    INSERT INTO faculty.subject_avg_grades(direction_subject_id, avg_grade, grades_count, updated_at)
    VALUES (NEW.direction_subject_id, v_avg, coalesce(v_cnt, 0), now())
    ON CONFLICT (direction_subject_id)
    DO UPDATE SET avg_grade = EXCLUDED.avg_grade,
                  grades_count = EXCLUDED.grades_count,
                  updated_at = now();

    RETURN NULL;
END;
$$;

DROP TRIGGER IF EXISTS trg_update_subject_avg ON faculty.grades;
CREATE TRIGGER trg_update_subject_avg
    AFTER INSERT OR UPDATE OF grade ON faculty.grades
    FOR EACH ROW
    EXECUTE FUNCTION faculty.trg_update_subject_avg();

COMMENT ON FUNCTION faculty.trg_update_subject_avg()
    IS 'Триггер: после внесения оценки пересчитывает среднюю оценку по предмету (across групп/направлений)';

COMMIT;

