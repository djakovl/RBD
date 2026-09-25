#!/usr/bin/env bash
set -euo pipefail
# Выполнять ТОЛЬКО после part2_split_bootstrap.sh. Это отделяет часть 3 от части 2.
podman exec -i z4-university psql -v ON_ERROR_STOP=1 -U postgres -d faculty_db <<'SQL'
CREATE EXTENSION IF NOT EXISTS postgres_fdw;
CREATE SERVER personal_srv FOREIGN DATA WRAPPER postgres_fdw OPTIONS (host 'personal',port '5432',dbname 'personal_db');
CREATE USER MAPPING FOR postgres SERVER personal_srv OPTIONS (user 'postgres',password 'postgres');
CREATE FOREIGN TABLE faculty.students (id bigint,surname varchar(100),first_name varchar(100),patronymic varchar(100),birth_date date,email varchar(254)) SERVER personal_srv OPTIONS (schema_name 'faculty',table_name 'students');
CREATE FOREIGN TABLE faculty.student_addresses (student_id bigint,city varchar(100),street varchar(150),house varchar(20)) SERVER personal_srv OPTIONS (schema_name 'faculty',table_name 'student_addresses');
CREATE FOREIGN TABLE faculty.student_phones (id bigint,student_id bigint,phone_number varchar(30)) SERVER personal_srv OPTIONS (schema_name 'faculty',table_name 'student_phones');
SQL
echo "Часть 3 готова: запусти исходный z1/client с DB_PORT=54323."
