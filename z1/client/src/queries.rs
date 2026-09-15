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

pub const Q01: &str = r#"
SELECT d.name::text AS "Направление", g.group_number::text AS "Группа",
       count(e.id)::text AS "Студентов"
FROM faculty.directions d
JOIN faculty.student_groups g ON g.direction_id = d.id
LEFT JOIN faculty.enrollments e ON e.group_id = g.id
WHERE d.id = $1
GROUP BY d.name, g.id, g.group_number
ORDER BY g.group_number
"#;

pub const Q02: &str = r#"
SELECT concat_ws(' ', s.surname, s.first_name, s.patronymic)::text AS "Студент",
       g.group_number::text AS "Группа", d.name::text AS "Направление",
       s.email::text AS "Email"
FROM faculty.students s
JOIN faculty.enrollments e ON e.student_id = s.id
JOIN faculty.student_groups g ON g.id = e.group_id
JOIN faculty.directions d ON d.id = g.direction_id
WHERE s.surname ILIKE $1
ORDER BY s.surname, s.first_name, g.group_number
"#;

pub const Q03: &str = r#"
SELECT to_char(s.birth_date, 'DD.MM.YYYY')::text AS "Дата рождения",
       concat_ws(' ', s.surname, s.first_name, s.patronymic)::text AS "Студент",
       string_agg(DISTINCT g.group_number, ', ' ORDER BY g.group_number)::text AS "Группы"
FROM faculty.students s
JOIN faculty.enrollments e ON e.student_id = s.id
JOIN faculty.student_groups g ON g.id = e.group_id
WHERE extract(month FROM s.birth_date)::int = $1
GROUP BY s.id
ORDER BY extract(day FROM s.birth_date), s.surname
"#;

pub const Q04: &str = r#"
SELECT concat_ws(' ', s.surname, s.first_name, s.patronymic)::text AS "Студент",
       to_char(s.birth_date, 'DD.MM.YYYY')::text AS "Дата рождения",
       extract(year FROM age(current_date, s.birth_date))::int::text AS "Возраст"
FROM faculty.students s
JOIN faculty.enrollments e ON e.student_id = s.id
WHERE e.group_id = $1
ORDER BY s.surname, s.first_name
"#;

pub const Q05: &str = r#"
SELECT to_char(s.birth_date, 'DD.MM.YYYY')::text AS "Дата рождения",
       concat_ws(' ', s.surname, s.first_name, s.patronymic)::text AS "Студент",
       s.email::text AS "Email"
FROM faculty.students s
WHERE extract(month FROM s.birth_date) = extract(month FROM current_date)
ORDER BY extract(day FROM s.birth_date), s.surname
"#;

pub const Q06: &str = r#"
SELECT d.name::text AS "Направление", count(e.id)::text AS "Студентов"
FROM faculty.directions d
LEFT JOIN faculty.student_groups g ON g.direction_id = d.id
LEFT JOIN faculty.enrollments e ON e.group_id = g.id
GROUP BY d.id, d.name
ORDER BY d.name
"#;

pub const Q07: &str = r#"
SELECT g.group_number::text AS "Группа",
       count(*) FILTER (WHERE ft.is_budget)::text AS "Бюджетные",
       count(*) FILTER (WHERE NOT ft.is_budget)::text AS "Внебюджетные",
       count(*)::text AS "Всего"
FROM faculty.student_groups g
JOIN faculty.enrollments e ON e.group_id = g.id
JOIN faculty.funding_types ft ON ft.id = e.funding_type_id
WHERE g.direction_id = $1
GROUP BY g.id, g.group_number
ORDER BY g.group_number
"#;

pub const Q08: &str = r#"
SELECT d.name::text AS "Направление", g.group_number::text AS "Группа",
       s.name::text AS "Предмет",
       concat_ws(' ', t.surname, t.first_name, t.patronymic)::text AS "Преподаватель"
FROM faculty.direction_subjects ds
JOIN faculty.directions d ON d.id = ds.direction_id
JOIN faculty.subjects s ON s.id = ds.subject_id
JOIN faculty.teachers t ON t.id = ds.teacher_id
JOIN faculty.student_groups g ON g.direction_id = d.id
WHERE s.id = $1 AND ($2::int IS NULL OR t.id = $2)
ORDER BY d.name, g.group_number
"#;

pub const Q09: &str = r#"
SELECT s.name::text AS "Дисциплина", count(DISTINCT e.student_id)::text AS "Студентов"
FROM faculty.direction_subjects ds
JOIN faculty.subjects s ON s.id = ds.subject_id
JOIN faculty.student_groups g ON g.direction_id = ds.direction_id
JOIN faculty.enrollments e ON e.group_id = g.id
GROUP BY s.id, s.name
ORDER BY count(DISTINCT e.student_id) DESC, s.name
LIMIT 1
"#;

pub const Q10: &str = r#"
SELECT concat_ws(' ', t.surname, t.first_name, t.patronymic)::text AS "Преподаватель",
       count(DISTINCT e.student_id)::text AS "Студентов"
FROM faculty.teachers t
JOIN faculty.direction_subjects ds ON ds.teacher_id = t.id
JOIN faculty.student_groups g ON g.direction_id = ds.direction_id
JOIN faculty.enrollments e ON e.group_id = g.id
WHERE t.id = $1
GROUP BY t.id
"#;

pub const Q11: &str = r#"
SELECT s.name::text AS "Дисциплина",
       count(g.id) FILTER (WHERE g.grade >= 3)::text AS "Сдали",
       count(g.id)::text AS "Всего оценок",
       round(100.0 * count(g.id) FILTER (WHERE g.grade >= 3) / nullif(count(g.id), 0), 2)::text AS "Доля, %"
FROM faculty.subjects s
JOIN faculty.direction_subjects ds ON ds.subject_id = s.id
LEFT JOIN faculty.grades g ON g.direction_subject_id = ds.id AND g.grade IS NOT NULL
WHERE s.id = $1
GROUP BY s.id, s.name
"#;

pub const Q12: &str = r#"
SELECT s.name::text AS "Дисциплина",
       round(avg(g.grade) FILTER (WHERE g.grade >= 3), 2)::text AS "Средняя оценка сдавших"
FROM faculty.subjects s
JOIN faculty.direction_subjects ds ON ds.subject_id = s.id
LEFT JOIN faculty.grades g ON g.direction_subject_id = ds.id
WHERE s.id = $1
GROUP BY s.id, s.name
"#;

pub const Q13: &str = r#"
SELECT g.group_number::text AS "Группа", d.name::text AS "Направление",
       round(avg(gr.grade) FILTER (WHERE gr.grade >= 3), 2)::text AS "Средняя оценка"
FROM faculty.student_groups g
JOIN faculty.directions d ON d.id = g.direction_id
JOIN faculty.enrollments e ON e.group_id = g.id
JOIN faculty.grades gr ON gr.enrollment_id = e.id
GROUP BY g.id, d.name
HAVING count(gr.grade) FILTER (WHERE gr.grade >= 3) > 0
ORDER BY avg(gr.grade) FILTER (WHERE gr.grade >= 3) DESC
LIMIT 1
"#;

pub const Q14: &str = r#"
SELECT concat_ws(' ', s.surname, s.first_name, s.patronymic)::text AS "Студент",
       g.group_number::text AS "Группа",
       round(avg(gr.grade), 2)::text AS "Средняя оценка"
FROM faculty.students s
JOIN faculty.enrollments e ON e.student_id = s.id
JOIN faculty.student_groups g ON g.id = e.group_id
JOIN faculty.grades gr ON gr.enrollment_id = e.id
WHERE g.direction_id = $1
GROUP BY s.id, e.id, g.group_number
HAVING count(*) > 0
   AND count(*) = count(gr.grade)
   AND min(gr.grade) = 5
ORDER BY s.surname, s.first_name
"#;

pub const Q15: &str = r#"
SELECT concat_ws(' ', s.surname, s.first_name, s.patronymic)::text AS "Студент",
       g.group_number::text AS "Группа",
       count(*) FILTER (WHERE gr.grade IS NULL OR gr.grade = 2)::text AS "Несданных"
FROM faculty.students s
JOIN faculty.enrollments e ON e.student_id = s.id
JOIN faculty.student_groups g ON g.id = e.group_id
JOIN faculty.grades gr ON gr.enrollment_id = e.id
GROUP BY s.id, e.id, g.group_number
HAVING count(*) FILTER (WHERE gr.grade IS NULL OR gr.grade = 2) >= $1
ORDER BY count(*) FILTER (WHERE gr.grade IS NULL OR gr.grade = 2) DESC, s.surname
"#;

pub const Q16: &str = r#"
SELECT to_char(l.lesson_date, 'DD.MM.YYYY')::text AS "Дата",
       g.group_number::text AS "Группа",
       s.name::text AS "Предмет",
       count(a.enrollment_id) FILTER (WHERE a.attended)::text AS "Присутствовали",
       count(a.enrollment_id)::text AS "Всего"
FROM faculty.lessons l
JOIN faculty.student_groups g ON g.id = l.group_id
JOIN faculty.direction_subjects ds ON ds.id = l.direction_subject_id
JOIN faculty.subjects s ON s.id = ds.subject_id
LEFT JOIN faculty.attendance a ON a.lesson_id = l.id
WHERE s.id = $1
GROUP BY l.id, g.group_number, s.name
ORDER BY l.lesson_date, g.group_number
"#;

pub const Q17: &str = r#"
SELECT concat_ws(' ', st.surname, st.first_name, st.patronymic)::text AS "Студент",
       g.group_number::text AS "Группа", count(*)::text AS "Пропусков"
FROM faculty.attendance a
JOIN faculty.enrollments e ON e.id = a.enrollment_id
JOIN faculty.students st ON st.id = e.student_id
JOIN faculty.student_groups g ON g.id = e.group_id
JOIN faculty.lessons l ON l.id = a.lesson_id
JOIN faculty.direction_subjects ds ON ds.id = l.direction_subject_id
WHERE ds.subject_id = $1 AND NOT a.attended
GROUP BY st.id, g.group_number
ORDER BY count(*) DESC, st.surname
"#;

pub const Q18: &str = r#"
SELECT
    result.student::text AS "Студент",
    result.group_number::text AS "Группа",
    result.subject_name::text AS "Предмет"
FROM (
    SELECT DISTINCT
        st.id AS student_id,
        st.surname,
        st.first_name,
        st.patronymic,
        concat_ws(' ', st.surname, st.first_name, st.patronymic) AS student,
        g.group_number,
        s.name AS subject_name
    FROM faculty.lessons l
    JOIN faculty.direction_subjects ds ON ds.id = l.direction_subject_id
    JOIN faculty.subjects s ON s.id = ds.subject_id
    JOIN faculty.student_groups g ON g.id = l.group_id
    JOIN faculty.attendance a ON a.lesson_id = l.id AND a.attended
    JOIN faculty.enrollments e ON e.id = a.enrollment_id
    JOIN faculty.students st ON st.id = e.student_id
    WHERE ds.teacher_id = $1
) AS result
ORDER BY result.subject_name, result.group_number,
         result.surname, result.first_name, result.patronymic
"#;

pub const Q19: &str = r#"
SELECT s.name::text AS "Предмет",
       count(a.lesson_id) FILTER (WHERE a.attended)::text AS "Посещено занятий",
       coalesce(sum(extract(epoch FROM (ls.end_time - ls.start_time)) / 60)
           FILTER (WHERE a.attended), 0)::int::text AS "Минут"
FROM faculty.students st
JOIN faculty.enrollments e ON e.student_id = st.id
JOIN faculty.attendance a ON a.enrollment_id = e.id
JOIN faculty.lessons l ON l.id = a.lesson_id
JOIN faculty.lesson_slots ls ON ls.id = l.slot_id
JOIN faculty.direction_subjects ds ON ds.id = l.direction_subject_id
JOIN faculty.subjects s ON s.id = ds.subject_id
WHERE st.id = $1
GROUP BY s.id, s.name
ORDER BY s.name
"#;
