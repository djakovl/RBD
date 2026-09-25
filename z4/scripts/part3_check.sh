#!/usr/bin/env bash
set -euo pipefail
podman exec z4-university psql -U postgres -d faculty_db -c "\det+ faculty.*"
podman exec z4-university psql -U postgres -d faculty_db -c "SELECT s.email,e.id AS enrollment_id FROM faculty.students s JOIN faculty.enrollments e ON e.student_id=s.id LIMIT 5;"
