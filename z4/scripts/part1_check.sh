#!/usr/bin/env bash
set -euo pipefail
for c in z4-primary z4-mirror; do
 echo "=== $c ==="
 podman exec "$c" psql -U postgres -d faculty_db -Atc "SELECT 'students='||count(*) FROM faculty.students UNION ALL SELECT 'enrollments='||count(*) FROM faculty.enrollments UNION ALL SELECT 'grades='||count(*) FROM faculty.grades;"
done
