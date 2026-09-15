use anyhow::Result;
use mongodb::bson::Document;
use mongodb::sync::Database;

use crate::{queries, selectors, table};

pub const MENU: &[(&str, &str)] = &[
    ("1", "Списки групп по направлению"),
    ("2", "Студенты по первой букве фамилии"),
    ("3", "Дни рождения выбранного месяца"),
    ("4", "Возраст студентов выбранной группы"),
    ("5", "Именинники текущего месяца"),
    ("6", "Количество студентов по направлениям"),
    ("7", "Бюджетные и внебюджетные места"),
    ("8", "Группы по предмету и преподавателю"),
    ("9", "Самая массовая дисциплина"),
    ("10", "Количество студентов у преподавателя"),
    ("11", "Доля сдавших по дисциплине"),
    ("12", "Средняя оценка сдавших"),
    ("13", "Группа с максимальной средней оценкой"),
    ("14", "Отличники без несданных экзаменов"),
    ("15", "Кандидаты на отчисление"),
    ("16", "Посещения по предмету"),
    ("17", "Пропуски по предмету"),
    ("18", "Студенты на занятиях преподавателя"),
    ("19", "Время студента по предметам"),
];

/// Запускает пайплайн на указанной коллекции и печатает результат.
/// Коллекция — стартовая точка агрегации ($match применяется к ней первой
/// стадией), а не единственный источник данных: $lookup внутри пайплайна
/// подключает все остальные необходимые коллекции.
/// В mongodb 2.8 aggregate() принимает второй аргумент — Option<AggregateOptions>
/// и сразу возвращает Result<Cursor<Document>>, без метода .run().
fn show(db: &Database, collection: &str, pipeline: Vec<Document>) -> Result<()> {
    let coll = db.collection::<Document>(collection);
    let cursor = coll.aggregate(pipeline, None)?;
    let docs: Vec<Document> = cursor.collect::<Result<_, _>>()?;
    table::print_documents(&docs);
    Ok(())
}

pub fn run(db: &Database, choice: &str) -> Result<()> {
    match choice {
        "1" => {
            let id = selectors::direction(db)?;
            let pipeline = queries::q01_groups_by_direction(id);
            show(db, "student_groups", pipeline)
        }
        "2" => {
            let letter = selectors::read_letter("Первая буква фамилии: ")?;
            let pipeline = queries::q02_students_by_letter(&letter);
            show(db, "students", pipeline)
        }
        "3" => {
            let month = selectors::read_i32("Номер месяца (1-12): ", 1, 12)?;
            let pipeline = queries::q03_birthdays_by_month(month);
            show(db, "students", pipeline)
        }
        "4" => {
            let id = selectors::group(db)?;
            let pipeline = queries::q04_ages_by_group(id);
            show(db, "enrollments", pipeline)
        }
        "5" => {
            let pipeline = queries::q05_birthdays_current_month();
            show(db, "students", pipeline)
        }
        "6" => {
            let pipeline = queries::q06_student_count_by_direction();
            show(db, "directions", pipeline)
        }
        "7" => {
            let pipeline = queries::q07_funding_by_group();
            show(db, "student_groups", pipeline)
        }
        "8" => {
            let subject = selectors::subject(db)?;
            let teacher = selectors::teacher(db, true)?;
            let pipeline = queries::q08_groups_by_subject(subject, teacher);
            show(db, "direction_subjects", pipeline)
        }
        "9" => {
            let pipeline = queries::q09_most_popular_subject();
            show(db, "direction_subjects", pipeline)
        }
        "10" => {
            let id = selectors::teacher(db, false)?.expect("выбор обязателен");
            let pipeline = queries::q10_students_by_teacher(id);
            show(db, "teachers", pipeline)
        }
        "11" => {
            let id = selectors::subject(db)?;
            let pipeline = queries::q11_pass_rate_by_subject(id);
            show(db, "subjects", pipeline)
        }
        "12" => {
            let id = selectors::subject(db)?;
            let pipeline = queries::q12_average_grade_by_subject(id);
            show(db, "subjects", pipeline)
        }
        "13" => {
            let pipeline = queries::q13_top_group_by_average();
            show(db, "student_groups", pipeline)
        }
        "14" => {
            let id = selectors::direction(db)?;
            let pipeline = queries::q14_excellent_students(id);
            show(db, "student_groups", pipeline)
        }
        "15" => {
            let count = selectors::read_i32("Минимум несданных экзаменов (1-20): ", 1, 20)?;
            let pipeline = queries::q15_expulsion_candidates(count);
            show(db, "enrollments", pipeline)
        }
        "16" => {
            let id = selectors::subject(db)?;
            let pipeline = queries::q16_attendance_by_subject(id);
            show(db, "lessons", pipeline)
        }
        "17" => {
            let id = selectors::subject(db)?;
            let pipeline = queries::q17_absences_by_subject(id);
            show(db, "attendance", pipeline)
        }
        "18" => {
            let id = selectors::teacher(db, false)?.expect("выбор обязателен");
            let pipeline = queries::q18_students_by_teacher(id);
            show(db, "attendance", pipeline)
        }
        "19" => {
            let Some(id) = selectors::student_search(db)? else {
                return Ok(());
            };
            let pipeline = queries::q19_time_by_subject(id);
            show(db, "enrollments", pipeline)
        }
        _ => {
            println!("Неизвестная задача.");
            Ok(())
        }
    }
}
