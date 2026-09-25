#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

podman-compose -f "$ROOT/z4/part1/compose.yaml" up -d

until podman exec z4-primary pg_isready -U postgres >/dev/null; do
    sleep 1
done

# 00 создаёт faculty_db, поэтому выполняется в системной базе postgres.
podman exec -i z4-primary \
    psql -v ON_ERROR_STOP=1 \
    -U postgres \
    -d postgres \
    < "$ROOT/z1/postgresql/sql/00_create_database.sql"

# Схема и справочные данные должны попасть именно в faculty_db.
for sql in 01_schema.sql 02_reference_data.sql; do
    podman exec -i z4-primary \
        psql -v ON_ERROR_STOP=1 \
        -U postgres \
        -d faculty_db \
        < "$ROOT/z1/postgresql/sql/$sql"
done

# Генератор первой лабораторной теперь подключается туда, где реально есть faculty.
DB_HOST=127.0.0.1 \
DB_PORT=54321 \
DB_NAME=faculty_db \
DB_USER=postgres \
DB_PASSWORD=postgres \
cargo run \
    --manifest-path "$ROOT/z1/client/Cargo.toml" \
    --bin fake_data \
    -- --reset

mkdir -p "$ROOT/z4/part1/dumps"

podman exec z4-primary \
    pg_dump -U postgres -Fc faculty_db \
    > "$ROOT/z4/part1/dumps/faculty_snapshot.dump"

echo "Снимок создан: z4/part1/dumps/faculty_snapshot.dump"