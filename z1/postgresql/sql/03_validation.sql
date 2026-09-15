\set ON_ERROR_STOP on
SELECT current_database() AS database_name;
SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'faculty'
ORDER BY table_name;
SELECT d.name AS direction, count(g.id) AS groups_count
FROM faculty.directions d
LEFT JOIN faculty.student_groups g ON g.direction_id = d.id
GROUP BY d.id, d.name
ORDER BY d.name;
