use mongodb::bson::{doc, oid::ObjectId, Document};

/// Каждая функция строит aggregation pipeline — прямой аналог SQL-запроса
/// из PostgreSQL-версии клиента. $lookup заменяет JOIN, $group — GROUP BY,
/// $sort — ORDER BY, $project — финальный список выводимых столбцов
/// (используется как замена AS "Название столбца" в SQL).
///
/// Референсы через _id (student_id, group_id, direction_id и т.д.) — это то,
/// что в задании называется "внешние ключи": MongoDB не проверяет их
/// существование на сервере, поэтому целостность гарантируется только тем,
/// что все вставки идут через код в fake_data.rs, который сначала находит
/// _id родительского документа и только потом создаёт дочерний.

/// Задача 1. Список групп по направлению с указанием ФИО и признака бюджета.
pub fn q01_groups_by_direction(direction_id: ObjectId) -> Vec<Document> {
    vec![
        doc! { "$match": { "direction_id": direction_id } },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "_id",
                "foreignField": "group_id",
                "as": "enrollment"
            }
        },
        doc! { "$unwind": { "path": "$enrollment", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "enrollment.student_id",
                "foreignField": "_id",
                "as": "student"
            }
        },
        doc! { "$unwind": { "path": "$student", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "funding_types",
                "localField": "enrollment.funding_type_id",
                "foreignField": "_id",
                "as": "funding"
            }
        },
        doc! { "$unwind": { "path": "$funding", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$project": {
                "_id": 0,
                "Группа": "$group_number",
                "Студент": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                { "$ifNull": ["$student.surname", ""] }, " ",
                                { "$ifNull": ["$student.first_name", ""] }, " ",
                                { "$ifNull": ["$student.patronymic", ""] }
                            ]
                        }
                    }
                },
                "Финансирование": { "$ifNull": ["$funding.name", "—"] }
            }
        },
        doc! { "$sort": { "Группа": 1, "Студент": 1 } },
    ]
}

/// Задача 2. Студенты по первой букве фамилии.
pub fn q02_students_by_letter(letter: &str) -> Vec<Document> {
    let pattern = format!("^{}", regex_escape(letter));
    vec![
        doc! {
            "$match": {
                "surname": { "$regex": pattern, "$options": "i" }
            }
        },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "_id",
                "foreignField": "student_id",
                "as": "enrollment"
            }
        },
        doc! { "$unwind": "$enrollment" },
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "enrollment.group_id",
                "foreignField": "_id",
                "as": "group"
            }
        },
        doc! { "$unwind": "$group" },
        doc! {
            "$lookup": {
                "from": "directions",
                "localField": "group.direction_id",
                "foreignField": "_id",
                "as": "direction"
            }
        },
        doc! { "$unwind": "$direction" },
        doc! {
            "$project": {
                "_id": 0,
                "Студент": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                "$surname", " ", "$first_name", " ",
                                { "$ifNull": ["$patronymic", ""] }
                            ]
                        }
                    }
                },
                "Группа": "$group.group_number",
                "Направление": "$direction.name",
                "Email": "$email"
            }
        },
        doc! { "$sort": { "Студент": 1, "Группа": 1 } },
    ]
}

/// Задача 3. Дни рождения в выбранном месяце.
pub fn q03_birthdays_by_month(month: i32) -> Vec<Document> {
    vec![
        doc! {
            "$addFields": {
                "birth_month": { "$month": "$birth_date" },
                "birth_day": { "$dayOfMonth": "$birth_date" }
            }
        },
        doc! { "$match": { "birth_month": month } },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "_id",
                "foreignField": "student_id",
                "as": "enrollment"
            }
        },
        doc! { "$unwind": { "path": "$enrollment", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "enrollment.group_id",
                "foreignField": "_id",
                "as": "group"
            }
        },
        doc! { "$unwind": { "path": "$group", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$group": {
                "_id": "$_id",
                "surname": { "$first": "$surname" },
                "first_name": { "$first": "$first_name" },
                "birth_day": { "$first": "$birth_day" },
                "groups": { "$addToSet": "$group.group_number" }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "День": "$birth_day",
                "Студент": {
                    "$concat": [
                        "$surname", " ",
                        { "$substrCP": ["$first_name", 0, 1] }, "."
                    ]
                },
                "Группы": {
                    "$reduce": {
                        "input": "$groups",
                        "initialValue": "",
                        "in": {
                            "$cond": [
                                { "$eq": ["$$value", ""] },
                                "$$this",
                                { "$concat": ["$$value", ", ", "$$this"] }
                            ]
                        }
                    }
                }
            }
        },
        doc! { "$sort": { "День": 1, "Студент": 1 } },
    ]
}

/// Задача 4. Возраст студентов выбранной группы.
pub fn q04_ages_by_group(group_id: ObjectId) -> Vec<Document> {
    vec![
        doc! { "$match": { "group_id": group_id } },
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "student_id",
                "foreignField": "_id",
                "as": "student"
            }
        },
        doc! { "$unwind": "$student" },
        doc! {
            "$project": {
                "_id": 0,
                "Студент": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                "$student.surname", " ", "$student.first_name", " ",
                                { "$ifNull": ["$student.patronymic", ""] }
                            ]
                        }
                    }
                },
                "Дата рождения": {
                    "$dateToString": { "format": "%d.%m.%Y", "date": "$student.birth_date" }
                },
                "Возраст": {
                    "$dateDiff": {
                        "startDate": "$student.birth_date",
                        "endDate": "$$NOW",
                        "unit": "year"
                    }
                }
            }
        },
        doc! { "$sort": { "Студент": 1 } },
    ]
}

/// Задача 5. Именинники текущего месяца (без параметров).
pub fn q05_birthdays_current_month() -> Vec<Document> {
    vec![
        doc! {
            "$addFields": {
                "birth_month": { "$month": "$birth_date" },
                "current_month": { "$month": "$$NOW" }
            }
        },
        doc! { "$match": { "$expr": { "$eq": ["$birth_month", "$current_month"] } } },
        doc! {
            "$project": {
                "_id": 0,
                "Дата рождения": {
                    "$dateToString": { "format": "%d.%m.%Y", "date": "$birth_date" }
                },
                "Студент": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                "$surname", " ", "$first_name", " ",
                                { "$ifNull": ["$patronymic", ""] }
                            ]
                        }
                    }
                },
                "Email": "$email"
            }
        },
        doc! { "$sort": { "Дата рождения": 1 } },
    ]
}

/// Задача 6. Количество студентов по направлениям.
pub fn q06_student_count_by_direction() -> Vec<Document> {
    vec![
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "_id",
                "foreignField": "direction_id",
                "as": "groups"
            }
        },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "groups._id",
                "foreignField": "group_id",
                "as": "enrollments"
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Направление": "$name",
                "Студентов": { "$size": "$enrollments" }
            }
        },
        doc! { "$sort": { "Направление": 1 } },
    ]
}

/// Задача 7. Бюджетные и внебюджетные места по группам.
pub fn q07_funding_by_group() -> Vec<Document> {
    vec![
        doc! {
            "$lookup": {
                "from": "directions",
                "localField": "direction_id",
                "foreignField": "_id",
                "as": "direction"
            }
        },
        doc! { "$unwind": "$direction" },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "_id",
                "foreignField": "group_id",
                "as": "enrollment"
            }
        },
        doc! { "$unwind": { "path": "$enrollment", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$lookup": {
                "from": "funding_types",
                "localField": "enrollment.funding_type_id",
                "foreignField": "_id",
                "as": "funding"
            }
        },
        doc! { "$unwind": { "path": "$funding", "preserveNullAndEmptyArrays": true } },
        doc! {
            "$group": {
                "_id": "$_id",
                "group_number": { "$first": "$group_number" },
                "direction_name": { "$first": "$direction.name" },
                "budget": {
                    "$sum": { "$cond": [{ "$eq": ["$funding.is_budget", true] }, 1, 0] }
                },
                "non_budget": {
                    "$sum": { "$cond": [{ "$eq": ["$funding.is_budget", false] }, 1, 0] }
                },
                "total": { "$sum": { "$cond": [{ "$ifNull": ["$enrollment._id", false] }, 1, 0] } }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Направление": "$direction_name",
                "Группа": "$group_number",
                "Бюджетные": "$budget",
                "Внебюджетные": "$non_budget",
                "Всего": "$total"
            }
        },
        doc! { "$sort": { "Направление": 1, "Группа": 1 } },
    ]
}

/// Задача 8. Группы по предмету и (опционально) преподавателю.
pub fn q08_groups_by_subject(subject_id: ObjectId, teacher_id: Option<ObjectId>) -> Vec<Document> {
    let mut match_stage = doc! { "subject_id": subject_id };
    if let Some(id) = teacher_id {
        match_stage.insert("teacher_id", id);
    }

    vec![
        doc! { "$match": match_stage },
        doc! {
            "$lookup": {
                "from": "directions",
                "localField": "direction_id",
                "foreignField": "_id",
                "as": "direction"
            }
        },
        doc! { "$unwind": "$direction" },
        doc! {
            "$lookup": {
                "from": "subjects",
                "localField": "subject_id",
                "foreignField": "_id",
                "as": "subject"
            }
        },
        doc! { "$unwind": "$subject" },
        doc! {
            "$lookup": {
                "from": "teachers",
                "localField": "teacher_id",
                "foreignField": "_id",
                "as": "teacher"
            }
        },
        doc! { "$unwind": "$teacher" },
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "direction_id",
                "foreignField": "direction_id",
                "as": "group"
            }
        },
        doc! { "$unwind": "$group" },
        doc! {
            "$project": {
                "_id": 0,
                "Направление": "$direction.name",
                "Группа": "$group.group_number",
                "Предмет": "$subject.name",
                "Преподаватель": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                "$teacher.surname", " ", "$teacher.first_name", " ",
                                { "$ifNull": ["$teacher.patronymic", ""] }
                            ]
                        }
                    }
                }
            }
        },
        doc! { "$sort": { "Направление": 1, "Группа": 1 } },
    ]
}

/// Задача 9. Самая массовая дисциплина.
pub fn q09_most_popular_subject() -> Vec<Document> {
    vec![
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "direction_id",
                "foreignField": "direction_id",
                "as": "groups"
            }
        },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "groups._id",
                "foreignField": "group_id",
                "as": "enrollments"
            }
        },
        doc! {
            "$lookup": {
                "from": "subjects",
                "localField": "subject_id",
                "foreignField": "_id",
                "as": "subject"
            }
        },
        doc! { "$unwind": "$subject" },
        doc! {
            "$group": {
                "_id": "$subject_id",
                "subject_name": { "$first": "$subject.name" },
                "students": { "$addToSet": "$enrollments.student_id" }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Дисциплина": "$subject_name",
                "Студентов": {
                    "$size": { "$reduce": {
                        "input": "$students",
                        "initialValue": [],
                        "in": { "$setUnion": ["$$value", "$$this"] }
                    }}
                }
            }
        },
        doc! { "$sort": { "Студентов": -1, "Дисциплина": 1 } },
        doc! { "$limit": 1 },
    ]
}

/// Задача 10. Количество студентов у выбранного преподавателя.
pub fn q10_students_by_teacher(teacher_id: ObjectId) -> Vec<Document> {
    vec![
        doc! { "$match": { "_id": teacher_id } },
        doc! {
            "$lookup": {
                "from": "direction_subjects",
                "localField": "_id",
                "foreignField": "teacher_id",
                "as": "assignments"
            }
        },
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "assignments.direction_id",
                "foreignField": "direction_id",
                "as": "groups"
            }
        },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "groups._id",
                "foreignField": "group_id",
                "as": "enrollments"
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Преподаватель": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                "$surname", " ", "$first_name", " ",
                                { "$ifNull": ["$patronymic", ""] }
                            ]
                        }
                    }
                },
                "Студентов": {
                    "$size": {
                        "$setUnion": ["$enrollments.student_id", []]
                    }
                }
            }
        },
    ]
}

/// Задача 11. Доля сдавших студентов по дисциплине (нет оценки или 2 = не сдал).
pub fn q11_pass_rate_by_subject(subject_id: ObjectId) -> Vec<Document> {
    vec![
        doc! { "$match": { "_id": subject_id } },
        doc! {
            "$lookup": {
                "from": "direction_subjects",
                "localField": "_id",
                "foreignField": "subject_id",
                "as": "assignments"
            }
        },
        doc! {
            "$lookup": {
                "from": "grades",
                "localField": "assignments._id",
                "foreignField": "direction_subject_id",
                "as": "grades"
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Дисциплина": "$name",
                "Сдали": {
                    "$size": {
                        "$filter": {
                            "input": "$grades",
                            "as": "grade",
                            "cond": { "$gte": ["$$grade.grade", 3] }
                        }
                    }
                },
                "Всего оценок": {
                    "$size": {
                        "$filter": {
                            "input": "$grades",
                            "as": "grade",
                            "cond": { "$ne": ["$$grade.grade", None::<i32>] }
                        }
                    }
                },
                "students_total": { "$size": "$grades" }
            }
        },
        doc! {
            "$addFields": {
                "Доля, %": {
                    "$cond": [
                        { "$eq": ["$students_total", 0] },
                        0.0,
                        { "$round": [
                            { "$multiply": [
                                { "$divide": ["$Сдали", "$students_total"] }, 100
                            ]}, 2
                        ]}
                    ]
                }
            }
        },
        doc! { "$project": { "students_total": 0 } },
    ]
}

/// Задача 12. Средняя оценка сдавших студентов по дисциплине.
pub fn q12_average_grade_by_subject(subject_id: ObjectId) -> Vec<Document> {
    vec![
        doc! { "$match": { "_id": subject_id } },
        doc! {
            "$lookup": {
                "from": "direction_subjects",
                "localField": "_id",
                "foreignField": "subject_id",
                "as": "assignments"
            }
        },
        doc! {
            "$lookup": {
                "from": "grades",
                "localField": "assignments._id",
                "foreignField": "direction_subject_id",
                "as": "grades"
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Дисциплина": "$name",
                "passing_grades": {
                    "$filter": {
                        "input": "$grades.grade",
                        "as": "grade",
                        "cond": { "$gte": ["$$grade", 3] }
                    }
                }
            }
        },
        doc! {
            "$project": {
                "Дисциплина": 1,
                "Средняя оценка сдавших": {
                    "$cond": [
                        { "$eq": [{ "$size": "$passing_grades" }, 0] },
                        None::<f64>,
                        { "$round": [{ "$avg": "$passing_grades" }, 2] }
                    ]
                }
            }
        },
    ]
}

/// Задача 13. Группа с максимальной средней оценкой (включая не сдавших).
pub fn q13_top_group_by_average() -> Vec<Document> {
    vec![
        doc! {
            "$lookup": {
                "from": "directions",
                "localField": "direction_id",
                "foreignField": "_id",
                "as": "direction"
            }
        },
        doc! { "$unwind": "$direction" },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "_id",
                "foreignField": "group_id",
                "as": "enrollments"
            }
        },
        doc! {
            "$lookup": {
                "from": "grades",
                "localField": "enrollments._id",
                "foreignField": "enrollment_id",
                "as": "grades"
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Группа": "$group_number",
                "Направление": "$direction.name",
                "grade_values": {
                    "$map": {
                        "input": "$grades",
                        "as": "grade",
                        "in": { "$ifNull": ["$$grade.grade", 2] }
                    }
                }
            }
        },
        doc! {
            "$addFields": {
                "Средняя оценка": {
                    "$cond": [
                        { "$eq": [{ "$size": "$grade_values" }, 0] },
                        None::<f64>,
                        { "$round": [{ "$avg": "$grade_values" }, 2] }
                    ]
                }
            }
        },
        doc! { "$project": { "grade_values": 0 } },
        doc! { "$match": { "Средняя оценка": { "$ne": None::<f64> } } },
        doc! { "$sort": { "Средняя оценка": -1 } },
        doc! { "$limit": 1 },
    ]
}

/// Задача 14. Отличники (все оценки 5) без несданных экзаменов, по направлению.
pub fn q14_excellent_students(direction_id: ObjectId) -> Vec<Document> {
    vec![
        doc! { "$match": { "direction_id": direction_id } },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "_id",
                "foreignField": "group_id",
                "as": "enrollment"
            }
        },
        doc! { "$unwind": "$enrollment" },
        doc! {
            "$lookup": {
                "from": "grades",
                "localField": "enrollment._id",
                "foreignField": "enrollment_id",
                "as": "grades"
            }
        },
        doc! { "$match": { "grades": { "$ne": [] } } },
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "enrollment.student_id",
                "foreignField": "_id",
                "as": "student"
            }
        },
        doc! { "$unwind": "$student" },
        doc! {
            "$project": {
                "_id": 0,
                "Группа": "$group_number",
                "Студент": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                "$student.surname", " ", "$student.first_name", " ",
                                { "$ifNull": ["$student.patronymic", ""] }
                            ]
                        }
                    }
                },
                "min_grade": { "$min": "$grades.grade" },
                "has_missing": {
                    "$in": [None::<i32>, "$grades.grade"]
                }
            }
        },
        doc! { "$match": { "min_grade": 5, "has_missing": false } },
        doc! { "$project": { "min_grade": 0, "has_missing": 0 } },
        doc! { "$sort": { "Студент": 1 } },
    ]
}

/// Задача 15. Кандидаты на отчисление (не сдано минимум N предметов).
pub fn q15_expulsion_candidates(min_failed: i32) -> Vec<Document> {
    vec![
        doc! {
            "$lookup": {
                "from": "grades",
                "localField": "_id",
                "foreignField": "enrollment_id",
                "as": "grades"
            }
        },
        doc! {
            "$addFields": {
                "failed": {
                    "$size": {
                        "$filter": {
                            "input": "$grades",
                            "as": "grade",
                            "cond": {
                                "$or": [
                                    { "$eq": ["$$grade.grade", None::<i32>] },
                                    { "$eq": ["$$grade.grade", 2] }
                                ]
                            }
                        }
                    }
                }
            }
        },
        doc! { "$match": { "failed": { "$gte": min_failed } } },
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "student_id",
                "foreignField": "_id",
                "as": "student"
            }
        },
        doc! { "$unwind": "$student" },
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "group_id",
                "foreignField": "_id",
                "as": "group"
            }
        },
        doc! { "$unwind": "$group" },
        doc! {
            "$project": {
                "_id": 0,
                "Студент": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                "$student.surname", " ", "$student.first_name", " ",
                                { "$ifNull": ["$student.patronymic", ""] }
                            ]
                        }
                    }
                },
                "Группа": "$group.group_number",
                "Несданных": "$failed"
            }
        },
        doc! { "$sort": { "Несданных": -1, "Студент": 1 } },
    ]
}

/// Задача 16. Посещённые занятия по выбранному предмету.
pub fn q16_attendance_by_subject(subject_id: ObjectId) -> Vec<Document> {
    vec![
        doc! {
            "$lookup": {
                "from": "direction_subjects",
                "localField": "direction_subject_id",
                "foreignField": "_id",
                "as": "ds"
            }
        },
        doc! { "$unwind": "$ds" },
        doc! { "$match": { "ds.subject_id": subject_id } },
        doc! {
            "$lookup": {
                "from": "subjects",
                "localField": "ds.subject_id",
                "foreignField": "_id",
                "as": "subject"
            }
        },
        doc! { "$unwind": "$subject" },
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "group_id",
                "foreignField": "_id",
                "as": "group"
            }
        },
        doc! { "$unwind": "$group" },
        doc! {
            "$lookup": {
                "from": "attendance",
                "localField": "_id",
                "foreignField": "lesson_id",
                "as": "attendance"
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Дата": { "$dateToString": { "format": "%d.%m.%Y", "date": "$lesson_date" } },
                "Группа": "$group.group_number",
                "Предмет": "$subject.name",
                "Присутствовали": {
                    "$size": {
                        "$filter": {
                            "input": "$attendance",
                            "as": "record",
                            "cond": { "$eq": ["$$record.attended", true] }
                        }
                    }
                },
                "Всего": { "$size": "$attendance" }
            }
        },
        doc! { "$sort": { "Дата": 1, "Группа": 1 } },
    ]
}

/// Задача 17. Пропуски занятий по выбранному предмету.
pub fn q17_absences_by_subject(subject_id: ObjectId) -> Vec<Document> {
    vec![
        doc! { "$match": { "attended": false } },
        doc! {
            "$lookup": {
                "from": "lessons",
                "localField": "lesson_id",
                "foreignField": "_id",
                "as": "lesson"
            }
        },
        doc! { "$unwind": "$lesson" },
        doc! {
            "$lookup": {
                "from": "direction_subjects",
                "localField": "lesson.direction_subject_id",
                "foreignField": "_id",
                "as": "ds"
            }
        },
        doc! { "$unwind": "$ds" },
        doc! { "$match": { "ds.subject_id": subject_id } },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "enrollment_id",
                "foreignField": "_id",
                "as": "enrollment"
            }
        },
        doc! { "$unwind": "$enrollment" },
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "enrollment.student_id",
                "foreignField": "_id",
                "as": "student"
            }
        },
        doc! { "$unwind": "$student" },
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "enrollment.group_id",
                "foreignField": "_id",
                "as": "group"
            }
        },
        doc! { "$unwind": "$group" },
        doc! {
            "$group": {
                "_id": "$student._id",
                "surname": { "$first": "$student.surname" },
                "first_name": { "$first": "$student.first_name" },
                "patronymic": { "$first": "$student.patronymic" },
                "group_number": { "$first": "$group.group_number" },
                "count": { "$sum": 1 }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Студент": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                "$surname", " ", "$first_name", " ",
                                { "$ifNull": ["$patronymic", ""] }
                            ]
                        }
                    }
                },
                "Группа": "$group_number",
                "Пропусков": "$count"
            }
        },
        doc! { "$sort": { "Пропусков": -1, "Студент": 1 } },
    ]
}

/// Задача 18. Студенты, присутствовавшие на занятиях выбранного преподавателя.
pub fn q18_students_by_teacher(teacher_id: ObjectId) -> Vec<Document> {
    vec![
        doc! { "$match": { "attended": true } },
        doc! {
            "$lookup": {
                "from": "lessons",
                "localField": "lesson_id",
                "foreignField": "_id",
                "as": "lesson"
            }
        },
        doc! { "$unwind": "$lesson" },
        doc! {
            "$lookup": {
                "from": "direction_subjects",
                "localField": "lesson.direction_subject_id",
                "foreignField": "_id",
                "as": "ds"
            }
        },
        doc! { "$unwind": "$ds" },
        doc! { "$match": { "ds.teacher_id": teacher_id } },
        doc! {
            "$lookup": {
                "from": "subjects",
                "localField": "ds.subject_id",
                "foreignField": "_id",
                "as": "subject"
            }
        },
        doc! { "$unwind": "$subject" },
        doc! {
            "$lookup": {
                "from": "student_groups",
                "localField": "lesson.group_id",
                "foreignField": "_id",
                "as": "group"
            }
        },
        doc! { "$unwind": "$group" },
        doc! {
            "$lookup": {
                "from": "enrollments",
                "localField": "enrollment_id",
                "foreignField": "_id",
                "as": "enrollment"
            }
        },
        doc! { "$unwind": "$enrollment" },
        doc! {
            "$lookup": {
                "from": "students",
                "localField": "enrollment.student_id",
                "foreignField": "_id",
                "as": "student"
            }
        },
        doc! { "$unwind": "$student" },
        // Уникальные комбинации студент/предмет/группа формируются через
        // $group — прямой аналог SELECT DISTINCT из PostgreSQL-версии;
        // сортировка выполняется уже после устранения дублей.
        doc! {
            "$group": {
                "_id": {
                    "student_id": "$student._id",
                    "subject_name": "$subject.name",
                    "group_number": "$group.group_number"
                },
                "surname": { "$first": "$student.surname" },
                "first_name": { "$first": "$student.first_name" },
                "patronymic": { "$first": "$student.patronymic" }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Студент": {
                    "$trim": {
                        "input": {
                            "$concat": [
                                "$surname", " ", "$first_name", " ",
                                { "$ifNull": ["$patronymic", ""] }
                            ]
                        }
                    }
                },
                "Группа": "$_id.group_number",
                "Предмет": "$_id.subject_name"
            }
        },
        doc! { "$sort": { "Предмет": 1, "Группа": 1, "Студент": 1 } },
    ]
}

/// Задача 19. Время, затраченное выбранным студентом на изучение каждого предмета.
pub fn q19_time_by_subject(student_id: ObjectId) -> Vec<Document> {
    vec![
        doc! { "$match": { "student_id": student_id } },
        doc! {
            "$lookup": {
                "from": "attendance",
                "localField": "_id",
                "foreignField": "enrollment_id",
                "as": "attendance"
            }
        },
        doc! { "$unwind": "$attendance" },
        doc! { "$match": { "attendance.attended": true } },
        doc! {
            "$lookup": {
                "from": "lessons",
                "localField": "attendance.lesson_id",
                "foreignField": "_id",
                "as": "lesson"
            }
        },
        doc! { "$unwind": "$lesson" },
        doc! {
            "$lookup": {
                "from": "lesson_slots",
                "localField": "lesson.slot_id",
                "foreignField": "_id",
                "as": "slot"
            }
        },
        doc! { "$unwind": "$slot" },
        doc! {
            "$lookup": {
                "from": "direction_subjects",
                "localField": "lesson.direction_subject_id",
                "foreignField": "_id",
                "as": "ds"
            }
        },
        doc! { "$unwind": "$ds" },
        doc! {
            "$lookup": {
                "from": "subjects",
                "localField": "ds.subject_id",
                "foreignField": "_id",
                "as": "subject"
            }
        },
        doc! { "$unwind": "$subject" },
        doc! {
            "$addFields": {
                "start_minutes": {
                    "$let": {
                        "vars": { "parts": { "$split": ["$slot.start_time", ":"] } },
                        "in": {
                            "$add": [
                                { "$multiply": [{ "$toInt": { "$arrayElemAt": ["$$parts", 0] } }, 60] },
                                { "$toInt": { "$arrayElemAt": ["$$parts", 1] } }
                            ]
                        }
                    }
                },
                "end_minutes": {
                    "$let": {
                        "vars": { "parts": { "$split": ["$slot.end_time", ":"] } },
                        "in": {
                            "$add": [
                                { "$multiply": [{ "$toInt": { "$arrayElemAt": ["$$parts", 0] } }, 60] },
                                { "$toInt": { "$arrayElemAt": ["$$parts", 1] } }
                            ]
                        }
                    }
                }
            }
        },
        doc! {
            "$group": {
                "_id": "$ds.subject_id",
                "subject_name": { "$first": "$subject.name" },
                "lessons_count": { "$sum": 1 },
                "total_minutes": { "$sum": { "$subtract": ["$end_minutes", "$start_minutes"] } }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "Предмет": "$subject_name",
                "Посещено занятий": "$lessons_count",
                "Минут": "$total_minutes"
            }
        },
        doc! { "$sort": { "Предмет": 1 } },
    ]
}

fn regex_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        if "\\.^$|()[]{}*+?".contains(ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}
