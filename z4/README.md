# Задание 4 — PostgreSQL 18 в Podman

## Требования

- Podman и `podman compose`;
- Rust/Cargo — только для запуска неизменённого генератора `z1/client/src/bin/fake_data.rs`;
- репозиторий должен содержать `z1/` рядом с `z4/`.

## Часть 1: снимок и зеркало

```bash
cd z4/scripts
chmod +x *.sh
./part1_bootstrap.sh
./part1_restore_mirror.sh
./part1_check.sh
```

`z4-primary` — исходный экземпляр. Скрипт создаёт логический снимок `z4/part1/dumps/faculty_snapshot.dump` через `pg_dump -Fc`. `z4-mirror` — чистый экземпляр, куда снимок восстанавливается через `pg_restore`. Оба контейнера независимы и используют разные named volumes.

Для запуска неизменённого Rust-клиента первой лабораторной:

```bash
# primary
DB_HOST=127.0.0.1 DB_PORT=54321 DB_NAME=faculty_db DB_USER=postgres DB_PASSWORD=postgres cargo run --manifest-path z1/client/Cargo.toml
# mirror
DB_HOST=127.0.0.1 DB_PORT=54322 DB_NAME=faculty_db DB_USER=postgres DB_PASSWORD=postgres cargo run --manifest-path z1/client/Cargo.toml
```

## Части 2–3: распределённая схема

```bash
cd z4/scripts
./part2_bootstrap.sh
./part2_check.sh
```

`z4-personal` содержит только `faculty.students`, `faculty.student_addresses`, `faculty.student_phones`. `z4-university` содержит учебные таблицы. После переноса персональные таблицы удаляются из `z4-university` и заменяются внешними таблицами с теми же именами через `postgres_fdw`. Поэтому исходные SQL-запросы клиента первой лабораторной не требуют переписывания: обращения к `faculty.students` автоматически читают ПДн с `z4-personal`.

```bash
DB_HOST=127.0.0.1 DB_PORT=54323 DB_NAME=faculty_db DB_USER=postgres DB_PASSWORD=postgres cargo run --manifest-path z1/client/Cargo.toml
```

## Важно

Декларативный FOREIGN KEY нельзя направить на foreign table в другой физической БД. Поэтому в `faculty.enrollments` на `z4-university` сохраняется `student_id`, но FK на удалённую `students` снимается. Все остальные локальные FK первой лабораторной остаются. Проверку наличия студента для новых записей можно выполнять запросом к внешней таблице `faculty.students`.
