pub const DIRECTIONS: &str = r#"
SELECT id, name AS label FROM faculty.directions ORDER BY name
"#;

pub const GROUPS: &str = r#"
SELECT id, group_number AS label FROM faculty.student_groups ORDER BY group_number
"#;

pub const SUBJECTS: &str = r#"
SELECT id, name AS label FROM faculty.subjects ORDER BY name
"#;

pub const TEACHERS: &str = r#"
SELECT id, concat_ws(' ', surname, first_name, patronymic) AS label
FROM faculty.teachers ORDER BY surname, first_name, patronymic
"#;

pub const STUDENTS: &str = r#"
SELECT id, concat_ws(' ', surname, first_name, patronymic) || ' — ' || email AS label
FROM faculty.students ORDER BY surname, first_name, patronymic
"#;

// =====================================================================
// Задание 3. Хранимые функции
// =====================================================================

pub const F01_AVG_GRADE_GROUP_SUBJECT: &str = r#"
SELECT faculty.avg_grade_group_subject($1, $2)::text AS "Средняя оценка"
"#;

pub const F02_AVG_GRADE_DIRECTION_SUBJECT: &str = r#"
SELECT faculty.avg_grade_direction_subject($1, $2)::text AS "Средняя оценка"
"#;

pub const F03_COUNT_EXCELLENT_STUDENTS: &str = r#"
SELECT faculty.count_excellent_students($1)::text AS "Отличников"
"#;

pub const F04_HAS_FAILED_EXAMS: &str = r#"
SELECT concat_ws(' ', s.surname, s.first_name, s.patronymic)::text AS "Студент",
       faculty.has_failed_exams(s.id)::text AS "Есть несданные"
FROM faculty.students s
WHERE s.id = $1
"#;

pub const F05_MISSED_BY_STUDENT: &str = r#"
SELECT student_name::text AS "Студент", missed_count::text AS "Пропусков"
FROM faculty.missed_lessons_by_student()
ORDER BY missed_count DESC
LIMIT 30
"#;

pub const F06_MISSED_BY_GROUP: &str = r#"
SELECT group_number::text AS "Группа", missed_count::text AS "Пропусков"
FROM faculty.missed_lessons_by_group()
"#;

pub const F07_MISSED_BY_DIRECTION: &str = r#"
SELECT direction_name::text AS "Направление", missed_count::text AS "Пропусков"
FROM faculty.missed_lessons_by_direction()
"#;

pub const F08_MISSED_BY_TEACHER: &str = r#"
SELECT teacher_name::text AS "Преподаватель", missed_count::text AS "Пропусков"
FROM faculty.missed_lessons_by_teacher()
"#;

// =====================================================================
// Задание 3. Триггеры — просмотр агрегатных таблиц и тестовые сценарии
// =====================================================================

pub const T09_GROUP_SUBJECT_AVG: &str = r#"
SELECT g.group_number::text AS "Группа", s.name::text AS "Предмет",
       t.avg_grade::text AS "Средняя оценка", t.grades_count::text AS "Оценок",
       to_char(t.updated_at, 'DD.MM.YYYY HH24:MI:SS')::text AS "Обновлено"
FROM faculty.group_subject_avg_grades t
JOIN faculty.student_groups g ON g.id = t.group_id
JOIN faculty.direction_subjects ds ON ds.id = t.direction_subject_id
JOIN faculty.subjects s ON s.id = ds.subject_id
ORDER BY t.updated_at DESC
LIMIT 30
"#;

pub const T10_GROUP_OVERALL_AVG: &str = r#"
SELECT g.group_number::text AS "Группа",
       t.avg_grade::text AS "Средняя оценка", t.grades_count::text AS "Оценок",
       to_char(t.updated_at, 'DD.MM.YYYY HH24:MI:SS')::text AS "Обновлено"
FROM faculty.group_overall_avg_grades t
JOIN faculty.student_groups g ON g.id = t.group_id
ORDER BY t.updated_at DESC
"#;

pub const T11_SUBJECT_AVG: &str = r#"
SELECT s.name::text AS "Предмет",
       t.avg_grade::text AS "Средняя оценка", t.grades_count::text AS "Оценок",
       to_char(t.updated_at, 'DD.MM.YYYY HH24:MI:SS')::text AS "Обновлено"
FROM faculty.subject_avg_grades t
JOIN faculty.direction_subjects ds ON ds.id = t.direction_subject_id
JOIN faculty.subjects s ON s.id = ds.subject_id
ORDER BY t.updated_at DESC
LIMIT 30
"#;

pub const T_PICK_GRADE_ROW: &str = r#"
SELECT gr.id, concat_ws(' ', st.surname, st.first_name, st.patronymic) || ' — ' ||
       s.name || ' (оценка: ' || coalesce(gr.grade::text, 'нет') || ')' AS label
FROM faculty.grades gr
JOIN faculty.enrollments e ON e.id = gr.enrollment_id
JOIN faculty.students st ON st.id = e.student_id
JOIN faculty.direction_subjects ds ON ds.id = gr.direction_subject_id
JOIN faculty.subjects s ON s.id = ds.subject_id
ORDER BY gr.id
LIMIT 20
"#;

pub const T14_INSERT_INVALID_GRADE: &str = r#"
INSERT INTO faculty.grades(enrollment_id, direction_subject_id, grade)
SELECT e.id, ds.id, 1
FROM faculty.enrollments e
JOIN faculty.direction_subjects ds
     ON ds.direction_id = (SELECT g.direction_id FROM faculty.student_groups g WHERE g.id = e.group_id)
WHERE e.id = $1
LIMIT 1
"#;

pub const T15_UPDATE_WRONG_TEACHER: &str = r#"
UPDATE faculty.grades
SET grade = 5, graded_by_teacher_id = $2
WHERE id = $1
"#;

pub const T15_UPDATE_CORRECT_TEACHER: &str = r#"
UPDATE faculty.grades g
SET grade = 5, graded_by_teacher_id = ds.teacher_id
FROM faculty.direction_subjects ds
WHERE ds.id = g.direction_subject_id AND g.id = $1
"#;

// =====================================================================
// Задание 3. Доп. параметры групп
// =====================================================================

pub const P12_NUMERIC_PARAM_STATS: &str = r#"
SELECT avg_value::text AS "Среднее", sum_value::text AS "Сумма",
       values_count::text AS "Значений"
FROM faculty.numeric_param_stats($1, $2)
"#;

pub const P13_SEARCH_TEXT_PARAM: &str = r#"
SELECT student_name::text AS "Студент", group_number::text AS "Группа",
       param_name::text AS "Параметр", found_value::text AS "Значение"
FROM faculty.search_text_param_detailed($1)
"#;

pub const PARAM_NAMES_NUMERIC: &str = r#"
SELECT dpd.id AS id, dpd.param_name AS label
FROM faculty.direction_param_defs dpd
JOIN faculty.param_types pt ON pt.id = dpd.param_type_id
WHERE pt.code = 'numeric'
ORDER BY dpd.param_name
"#;

// =====================================================================
// Задание 3. Демонстрация сломанного триггера (самозацикливание)
// =====================================================================

pub const B16_ENABLE_BROKEN_TRIGGER: &str = "SELECT faculty.enable_broken_trigger()";
pub const B16_DISABLE_BROKEN_TRIGGER: &str = "SELECT faculty.disable_broken_trigger()";
pub const B16_TRIGGER_UPDATE: &str = "UPDATE faculty.grades SET grade = 5 WHERE id = $1";