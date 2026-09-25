#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
podman-compose -f "$ROOT/z4/part2/compose.yaml" down -v
podman-compose -f "$ROOT/z4/part2/compose.yaml" up -d
for c in z4-university z4-personal; do until podman exec "$c" pg_isready -U postgres >/dev/null; do sleep 1; done; done
for sql in 01_schema.sql 02_reference_data.sql; do
  podman exec -i z4-university psql -v ON_ERROR_STOP=1 -U postgres -d faculty_db < "$ROOT/z1/postgresql/sql/$sql"
done
DB_HOST=127.0.0.1 DB_PORT=54323 DB_NAME=faculty_db DB_USER=postgres DB_PASSWORD=postgres cargo run --manifest-path "$ROOT/z1/client/Cargo.toml" --bin fake_data -- --reset
for table in students student_addresses student_phones; do
  podman exec z4-university pg_dump -U postgres -d faculty_db --data-only --table="faculty.$table" | podman exec -i z4-personal psql -v ON_ERROR_STOP=1 -U postgres -d personal_db
done
podman exec -i z4-university psql -v ON_ERROR_STOP=1 -U postgres -d faculty_db <<'SQL'
ALTER TABLE faculty.enrollments DROP CONSTRAINT enrollments_student_id_fkey;
DROP TABLE faculty.student_phones, faculty.student_addresses, faculty.students;
SQL
"$ROOT/z4/scripts/make_part2_client.sh"
echo "Часть 2 готова: FDW НЕ настроен. Запускай z4/part2/client_app."
