# Faculty demo client (MongoDB)

Консольный клиент MongoDB на Rust. Задание идентично заданию 1 (PostgreSQL),
меняется только СУБД. Ниже описано, как в MongoDB реализованы требования
"третья нормальная форма" и "внешние ключи в таблицах", раз в document-модели
их нет как декларативных серверных constraint.

## Как трактованы "3НФ" и "внешние ключи" в MongoDB

MongoDB не поддерживает FOREIGN KEY как constraint: сервер не проверяет,
что ссылка на _id существует в другой коллекции, и не делает ON DELETE
CASCADE. Единственный серверный механизм контроля структуры документа —
валидатор `$jsonSchema`, который проверяет типы, обязательные поля и
допустимые значения при insert/update.

Поэтому нормализация и целостность обеспечиваются гибридно:

1. **Схема данных (уровень модели).** Сущности разложены так же, как в
   3НФ-модели PostgreSQL: `directions`, `student_groups`, `funding_types`,
   `students`, `enrollments`, `subjects`, `teachers`, `direction_subjects`,
   `grades`, `lesson_slots`, `lessons`, `attendance`. Производные и
   повторяющиеся данные не хранятся в нескольких местах: например, название
   направления не копируется в документ студента — оно достаётся через
   `$lookup` от `group_id` -> `direction_id` -> `directions.name`.

2. **Ссылки вместо внешних ключей.** Поля `direction_id`, `group_id`,
   `student_id`, `subject_id`, `teacher_id`, `enrollment_id`, `lesson_id`
   хранят `ObjectId` документа из другой коллекции — это прямой аналог
   FOREIGN KEY, но без серверной проверки существования.

3. **3НФ на уровне организации работы, а не декларации.** Поскольку
   MongoDB не может объявить "это внешний ключ" как в SQL, дисциплина
   поддерживается на уровне кода:
   - все вставки идут только через `fake_data.rs` / клиентский код, который
     сначала находит `_id` родителя и только потом создаёт дочерний документ;
   - `mongo/03_validation.js` — отдельный скрипт, который агрегацией с
     `$lookup` находит "осиротевшие" ссылки (аналог проверки FK) и должен
     показывать 0 после каждого наполнения БД;
   - структура и обязательность полей фиксируются `$jsonSchema`-валидатором
     на каждой коллекции (файл `mongo/01_schema.js`) — аналог `NOT NULL` и
     `CHECK` из PostgreSQL.

4. **Embedding используется по делу, а не вместо нормализации.** Адрес и
   телефоны встроены прямо в документ `students`, а не выделены в отдельные
   коллекции — это оправдано, поскольку они принадлежат только одному
   студенту и всегда читаются вместе с ним. Все данные, которые совместно
   используются несколькими сущностями (направления, группы, предметы,
   преподаватели, учебный план), вынесены в отдельные коллекции и связаны
   через ссылки — то есть нормализованы в духе 3НФ.

## Структура проекта

- `mongo/01_schema.js` — создание коллекций и валидаторов `$jsonSchema`.
- `mongo/02_reference_data.js` — направления, группы, типы финансирования,
  предметы, преподаватели, учебный план, время пар.
- `mongo/03_validation.js` — подсчёт документов и проверка ссылочной
  целостности (аналог проверки внешних ключей).
- `mongo/99_reset_schema.js` — удаление всей базы `faculty_db`.
- `src/bin/fake_data.rs` — генератор тестовых студентов, зачислений, оценок,
  занятий и посещаемости.
- `src/main.rs` — консольное меню из 19 задач.
- `src/queries.rs` — aggregation pipeline для каждой задачи ($lookup вместо
  JOIN, $group вместо GROUP BY, $sort вместо ORDER BY, $project — выводимые
  столбцы).
- `src/selectors.rs` — выбор направления/группы/предмета/преподавателя из
  справочников и поиск студента по части ФИО/email.
- `src/table.rs` — универсальный табличный вывод документов результата.

## Настройка

```bash
cp .env.example .env
```

`.env`:

```dotenv
MONGO_URI=mongodb://localhost:27017
MONGO_DB=faculty_db
```

## Запуск MongoDB и mongosh через Podman

```bash
podman pod create --name faculty-mongo-pod -p 127.0.0.1:27017:27017

podman run -d \
  --name faculty-mongo \
  --pod faculty-mongo-pod \
  -v faculty-mongo-data:/data/db \
  docker.io/library/mongo:8.0
```

Проверка:

```bash
podman exec -it faculty-mongo mongosh --eval "db.runCommand({ ping: 1 })"
```

## Первичное наполнение

```bash
podman exec -i faculty-mongo mongosh mongodb://localhost:27017 < mongo/01_schema.js
podman exec -i faculty-mongo mongosh mongodb://localhost:27017 < mongo/02_reference_data.js
cargo run --bin fake_data
podman exec -i faculty-mongo mongosh mongodb://localhost:27017 < mongo/03_validation.js
cargo run
```

Полная пересборка (удаляет всю базу):

```bash
podman exec -i faculty-mongo mongosh mongodb://localhost:27017 < mongo/99_reset_schema.js
podman exec -i faculty-mongo mongosh mongodb://localhost:27017 < mongo/01_schema.js
podman exec -i faculty-mongo mongosh mongodb://localhost:27017 < mongo/02_reference_data.js
cargo run --bin fake_data
```

Повторное наполнение тестовыми данными без пересборки схемы:

```bash
cargo run --bin fake_data -- --reset
```

## Соответствие задачам из ТЗ

Меню и логика задач 1-19 перенесены из PostgreSQL-версии без изменения
смысла запросов — только SQL заменён на aggregation pipeline:

| № | Задача | Стартовая коллекция |
|---|---|---|
| 1 | Группы по направлению, ФИО, бюджет/внебюджет | `student_groups` |
| 2 | Студенты по первой букве фамилии | `students` |
| 3 | Дни рождения выбранного месяца | `students` |
| 4 | Возраст студентов группы | `enrollments` |
| 5 | Именинники текущего месяца | `students` |
| 6 | Количество студентов по направлениям | `directions` |
| 7 | Бюджетные/внебюджетные места по группам | `student_groups` |
| 8 | Группы по предмету и преподавателю | `direction_subjects` |
| 9 | Самая массовая дисциплина | `direction_subjects` |
| 10 | Студентов у преподавателя | `teachers` |
| 11 | Доля сдавших по дисциплине | `subjects` |
| 12 | Средняя оценка сдавших | `subjects` |
| 13 | Группа с максимальной средней оценкой | `student_groups` |
| 14 | Отличники без несданных экзаменов | `student_groups` |
| 15 | Кандидаты на отчисление | `enrollments` |
| 16 | Посещения по предмету | `lessons` |
| 17 | Пропуски по предмету | `attendance` |
| 18 | Студенты на занятиях преподавателя | `attendance` |
| 19 | Время студента по предметам | `enrollments` |

## Статус

Черновая версия: `cargo check`/`cargo fmt`/`cargo clippy` в среде сборки не
выполнены (в среде отсутствует `cargo`), запуск против реальной MongoDB не
проверялся. Перед защитой обязательно прогнать:

```bash
cargo fmt --all -- --check
cargo check --locked
cargo clippy --all-targets --locked -- -D warnings
```

и вручную проверить все 19 пунктов меню, а также `mongo/03_validation.js`
на реальных данных.
