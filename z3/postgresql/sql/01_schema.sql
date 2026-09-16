\set ON_ERROR_STOP on
BEGIN;

CREATE SCHEMA IF NOT EXISTS faculty;
SET search_path TO faculty, public;

CREATE TABLE directions (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name varchar(200) NOT NULL UNIQUE
);

CREATE TABLE student_groups (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    group_number varchar(30) NOT NULL UNIQUE,
    direction_id integer NOT NULL REFERENCES directions(id)
);

CREATE TABLE funding_types (
    id smallint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    code varchar(30) NOT NULL UNIQUE,
    name varchar(100) NOT NULL UNIQUE,
    is_budget boolean NOT NULL
);

CREATE TABLE students (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    surname varchar(100) NOT NULL,
    first_name varchar(100) NOT NULL,
    patronymic varchar(100),
    birth_date date NOT NULL,
    email varchar(254) NOT NULL UNIQUE
);

CREATE TABLE student_addresses (
    student_id bigint PRIMARY KEY REFERENCES students(id) ON DELETE CASCADE,
    city varchar(100) NOT NULL,
    street varchar(150) NOT NULL,
    house varchar(20) NOT NULL
);

CREATE TABLE student_phones (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    student_id bigint NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    phone_number varchar(30) NOT NULL,
    UNIQUE (student_id, phone_number)
);

CREATE TABLE enrollments (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    student_id bigint NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    group_id integer NOT NULL REFERENCES student_groups(id),
    funding_type_id smallint NOT NULL REFERENCES funding_types(id),
    UNIQUE (student_id, group_id)
);

CREATE TABLE subjects (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name varchar(200) NOT NULL UNIQUE
);

CREATE TABLE teachers (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    surname varchar(100) NOT NULL,
    first_name varchar(100) NOT NULL,
    patronymic varchar(100),
    UNIQUE (surname, first_name, patronymic)
);

CREATE TABLE direction_subjects (
    id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    direction_id integer NOT NULL REFERENCES directions(id),
    subject_id integer NOT NULL REFERENCES subjects(id),
    teacher_id integer NOT NULL REFERENCES teachers(id),
    UNIQUE (direction_id, subject_id)
);

CREATE TABLE grades (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    enrollment_id bigint NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    direction_subject_id integer NOT NULL REFERENCES direction_subjects(id),
    grade smallint,
    exam_date date,
    UNIQUE (enrollment_id, direction_subject_id),
    CHECK (grade IS NULL OR grade BETWEEN 2 AND 5)
);

CREATE TABLE lesson_slots (
    id smallint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    pair_number smallint NOT NULL UNIQUE,
    start_time time NOT NULL,
    end_time time NOT NULL,
    CHECK (end_time > start_time)
);

CREATE TABLE lessons (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    group_id integer NOT NULL REFERENCES student_groups(id),
    direction_subject_id integer NOT NULL REFERENCES direction_subjects(id),
    lesson_date date NOT NULL,
    slot_id smallint NOT NULL REFERENCES lesson_slots(id),
    UNIQUE (group_id, lesson_date, slot_id)
);

CREATE TABLE attendance (
    lesson_id bigint NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
    enrollment_id bigint NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    attended boolean NOT NULL,
    PRIMARY KEY (lesson_id, enrollment_id)
);

CREATE INDEX idx_groups_direction ON student_groups(direction_id);
CREATE INDEX idx_enrollments_student ON enrollments(student_id);
CREATE INDEX idx_enrollments_group ON enrollments(group_id);
CREATE INDEX idx_direction_subjects_subject ON direction_subjects(subject_id);
CREATE INDEX idx_direction_subjects_teacher ON direction_subjects(teacher_id);
CREATE INDEX idx_grades_enrollment ON grades(enrollment_id);
CREATE INDEX idx_lessons_group_date ON lessons(group_id, lesson_date);
CREATE INDEX idx_attendance_enrollment ON attendance(enrollment_id);

COMMIT;
