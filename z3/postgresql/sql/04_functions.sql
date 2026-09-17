
\set ON_ERROR_STOP on
BEGIN;
SET search_path TO faculty, public;

-- ---------------------------------------------------------------------
-- 1.1 Средняя оценка по предмету для заданной группы
--     grade IS NULL не учитывается (нет оценки != 2), т.к. это отдельная
--     семантика "не сдавал"; см. п.1.3 про отличников и п.1.4 про несданные.
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.avg_grade_group_subject(
    p_group_id integer,
    p_subject_id integer
) RETURNS numeric
LANGUAGE sql
STABLE
AS $$
    SELECT round(avg(gr.grade), 2)
    FROM faculty.enrollments e
    JOIN faculty.direction_subjects ds
         ON ds.subject_id = p_subject_id
        AND ds.direction_id = (SELECT g.direction_id FROM faculty.student_groups g WHERE g.id = p_group_id)
    JOIN faculty.grades gr
         ON gr.enrollment_id = e.id
        AND gr.direction_subject_id = ds.id
    WHERE e.group_id = p_group_id
      AND gr.grade IS NOT NULL;
$$;

COMMENT ON FUNCTION faculty.avg_grade_group_subject(integer, integer)
    IS 'Средняя оценка по предмету (subject_id) для указанной группы (group_id)';



-- ---------------------------------------------------------------------
-- 1.2 Средняя оценка по предмету для заданного направления
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.avg_grade_direction_subject(
    p_direction_id integer,
    p_subject_id integer
) RETURNS numeric
LANGUAGE sql
STABLE
AS $$
    SELECT round(avg(gr.grade), 2)
    FROM faculty.direction_subjects ds
    JOIN faculty.grades gr ON gr.direction_subject_id = ds.id
    WHERE ds.direction_id = p_direction_id
      AND ds.subject_id = p_subject_id
      AND gr.grade IS NOT NULL;
$$;

COMMENT ON FUNCTION faculty.avg_grade_direction_subject(integer, integer)
    IS 'Средняя оценка по предмету (subject_id) для указанного направления (direction_id)';



-- ---------------------------------------------------------------------
-- 1.3 Количество отличников в группе.
--     Отличник = у всех назначенных предметов направления итоговая
--     оценка = 5 (отсутствие оценки считается неудовлетворительной, т.е.
--     таких студентов в число отличников не включаем).
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.count_excellent_students(
    p_group_id integer
) RETURNS integer
LANGUAGE sql
STABLE
AS $$
    SELECT count(*)::integer
    FROM (
        SELECT e.id
        FROM faculty.enrollments e
        JOIN faculty.direction_subjects ds
             ON ds.direction_id = (SELECT g.direction_id FROM faculty.student_groups g WHERE g.id = p_group_id)
        LEFT JOIN faculty.grades gr
             ON gr.enrollment_id = e.id
            AND gr.direction_subject_id = ds.id
        WHERE e.group_id = p_group_id
        GROUP BY e.id
        HAVING count(*) = count(*) FILTER (WHERE gr.grade = 5)
    ) AS excellent;
$$;

COMMENT ON FUNCTION faculty.count_excellent_students(integer)
    IS 'Количество отличников в группе (все оценки = 5; отсутствие оценки — неуд.)';


-- ---------------------------------------------------------------------
-- 1.4 Логическая функция: есть ли у студента несданные экзамены
--     (оценка = 2 или оценка отсутствует по любому назначенному предмету).
--     Принимает student_id (глобально по всем его enrollments).
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.has_failed_exams(
    p_student_id bigint
) RETURNS boolean
LANGUAGE sql
STABLE
AS $$
    SELECT EXISTS (
        SELECT 1
        FROM faculty.enrollments e
        JOIN faculty.direction_subjects ds
             ON ds.direction_id = (SELECT g.direction_id FROM faculty.student_groups g WHERE g.id = e.group_id)
        LEFT JOIN faculty.grades gr
             ON gr.enrollment_id = e.id
            AND gr.direction_subject_id = ds.id
        WHERE e.student_id = p_student_id
          AND (gr.grade IS NULL OR gr.grade = 2)
    );
$$;

COMMENT ON FUNCTION faculty.has_failed_exams(bigint)
    IS 'true, если у студента есть оценка 2 или отсутствующая оценка по назначенному предмету направления';

-- ---------------------------------------------------------------------
-- 1.5 Количество пропущенных занятий для каждого студента
--     Возвращает таблицу (student_id, student_name, missed_count)
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.missed_lessons_by_student()
RETURNS TABLE (
    student_id bigint,
    student_name text,
    missed_count bigint
)
LANGUAGE sql
STABLE
AS $$
    SELECT s.id,
           concat_ws(' ', s.surname, s.first_name, s.patronymic),
           count(*) FILTER (WHERE NOT a.attended)
    FROM faculty.students s
    JOIN faculty.enrollments e ON e.student_id = s.id
    JOIN faculty.attendance a ON a.enrollment_id = e.id
    GROUP BY s.id, s.surname, s.first_name, s.patronymic
    ORDER BY count(*) FILTER (WHERE NOT a.attended) DESC, s.surname;
$$;

COMMENT ON FUNCTION faculty.missed_lessons_by_student()
    IS 'Количество пропущенных занятий по каждому студенту';

-- ---------------------------------------------------------------------
-- 1.6 Количество пропущенных занятий для каждой группы
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.missed_lessons_by_group()
RETURNS TABLE (
    group_id integer,
    group_number text,
    missed_count bigint
)
LANGUAGE sql
STABLE
AS $$
    SELECT g.id, g.group_number, count(*) FILTER (WHERE NOT a.attended)
    FROM faculty.student_groups g
    JOIN faculty.enrollments e ON e.group_id = g.id
    JOIN faculty.attendance a ON a.enrollment_id = e.id
    GROUP BY g.id, g.group_number
    ORDER BY g.group_number;
$$;

COMMENT ON FUNCTION faculty.missed_lessons_by_group()
    IS 'Количество пропущенных занятий по каждой группе';

-- ---------------------------------------------------------------------
-- 1.7 Количество пропущенных занятий для каждого направления
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.missed_lessons_by_direction()
RETURNS TABLE (
    direction_id integer,
    direction_name text,
    missed_count bigint
)
LANGUAGE sql
STABLE
AS $$
    SELECT d.id, d.name, count(*) FILTER (WHERE NOT a.attended)
    FROM faculty.directions d
    JOIN faculty.student_groups g ON g.direction_id = d.id
    JOIN faculty.enrollments e ON e.group_id = g.id
    JOIN faculty.attendance a ON a.enrollment_id = e.id
    GROUP BY d.id, d.name
    ORDER BY d.name;
$$;

COMMENT ON FUNCTION faculty.missed_lessons_by_direction()
    IS 'Количество пропущенных занятий по каждому направлению';

-- ---------------------------------------------------------------------
-- 1.8 Количество пропущенных занятий для каждого преподавателя
--     (занятия, которые вёл преподаватель по своим назначениям на
--     предмет-направление; пропуск = attended = false)
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.missed_lessons_by_teacher()
RETURNS TABLE (
    teacher_id integer,
    teacher_name text,
    missed_count bigint
)
LANGUAGE sql
STABLE
AS $$
    SELECT t.id,
           concat_ws(' ', t.surname, t.first_name, t.patronymic),
           count(*) FILTER (WHERE NOT a.attended)
    FROM faculty.teachers t
    JOIN faculty.direction_subjects ds ON ds.teacher_id = t.id
    JOIN faculty.lessons l ON l.direction_subject_id = ds.id
    JOIN faculty.attendance a ON a.lesson_id = l.id
    GROUP BY t.id, t.surname, t.first_name, t.patronymic
    ORDER BY count(*) FILTER (WHERE NOT a.attended) DESC, t.surname;
$$;

COMMENT ON FUNCTION faculty.missed_lessons_by_teacher()
    IS 'Количество пропущенных занятий по каждому преподавателю (по его предметам)';


COMMIT;
