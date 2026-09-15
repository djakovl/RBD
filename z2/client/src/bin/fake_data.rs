use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate, Utc};
use faculty_mongo_client::{config::MongoConfig, db};
use mongodb::bson::{doc, oid::ObjectId, DateTime as BsonDateTime, Document};
use mongodb::sync::Database;
use rand::{seq::SliceRandom, Rng};
use std::collections::HashMap;

const SURNAMES: &[&str] = &[
    "Иванов", "Петров", "Сидоров", "Смирнов", "Кузнецов", "Попов", "Соколов",
    "Лебедев", "Козлов", "Новиков", "Морозов", "Волков", "Орлов", "Васильев",
    "Зайцев", "Павлов", "Семёнов", "Голубев", "Виноградов", "Богданов",
];
const FIRST_NAMES: &[&str] = &[
    "Александр", "Иван", "Максим", "Артём", "Дмитрий", "Никита", "Михаил",
    "Анна", "Мария", "Елена", "Дарья", "Ольга", "Полина", "София", "Алина",
];
const PATRONYMICS: &[&str] = &[
    "Александрович", "Иванович", "Сергеевич", "Алексеевич", "Дмитриевич",
    "Михайлович", "Александровна", "Ивановна", "Сергеевна", "Алексеевна",
    "Дмитриевна", "Михайловна",
];
const CITIES: &[&str] = &["Москва", "Тула", "Калуга", "Рязань", "Тверь", "Коломна", "Подольск"];
const STREETS: &[&str] = &[
    "Ленина", "Советская", "Мира", "Молодёжная", "Садовая", "Школьная",
    "Лесная", "Центральная",
];

fn main() {
    if let Err(error) = run() {
        eprintln!("Ошибка генератора: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let settings = MongoConfig::from_env()?;
    let database = db::connect(&settings)?;
    let reset = std::env::args().any(|arg| arg == "--reset");

    if reset {
        for name in [
            "attendance", "lessons", "grades", "enrollments", "students",
        ] {
            database
                .collection::<Document>(name)
                .delete_many(doc! {}, None)?;
        }
        println!("Данные студентов, зачислений, оценок, занятий и посещаемости очищены.");
    } else {
        let existing = database
            .collection::<Document>("students")
            .count_documents(doc! {}, None)?;
        if existing > 0 {
            anyhow::bail!(
                "студенты уже существуют; используйте cargo run --bin fake_data -- --reset"
            );
        }
    }

    let groups: Vec<Document> = database
        .collection::<Document>("student_groups")
        .find(doc! {}, None)?
        .collect::<Result<_, _>>()?;
    let funding_types: Vec<Document> = database
        .collection::<Document>("funding_types")
        .find(doc! {}, None)?
        .collect::<Result<_, _>>()?;

    if groups.is_empty() || funding_types.is_empty() {
        anyhow::bail!("сначала выполните mongo/01_schema.js и mongo/02_reference_data.js");
    }

    let funding_ids: Vec<ObjectId> = funding_types
        .iter()
        .map(|doc| doc.get_object_id("_id").unwrap())
        .collect();

    let mut rng = rand::thread_rng();
    let mut enrollment_count = 0usize;

    for group in &groups {
        let group_id = group.get_object_id("_id")?;
        let direction_id = group.get_object_id("direction_id")?;
        let count = rng.gen_range(15..=24);

        for _ in 0..count {
            let student_id = insert_student(&database, &mut rng)?;
            let funding_id = *funding_ids.choose(&mut rng).unwrap();

            let enrollment_id = insert_one_get_id(
                &database,
                "enrollments",
                doc! {
                    "student_id": student_id,
                    "group_id": group_id,
                    "funding_type_id": funding_id
                },
            )?;

            insert_grades(&database, enrollment_id, direction_id, &mut rng)?;
            enrollment_count += 1;
        }
    }

    create_lessons_and_attendance(&database, &mut rng)?;

    println!(
        "Создано студентов/зачислений: {enrollment_count}. Оценки, занятия и посещаемость добавлены."
    );
    Ok(())
}

/// В mongodb 2.8 insert_one принимает второй аргумент Option<InsertOneOptions>
/// и сразу возвращает Result<InsertOneResult> — без метода .run().
fn insert_one_get_id(database: &Database, collection: &str, doc: Document) -> Result<ObjectId> {
    let result = database
        .collection::<Document>(collection)
        .insert_one(doc, None)?;
    result
        .inserted_id
        .as_object_id()
        .context("MongoDB вернул не ObjectId")
}

fn to_bson_date(date: NaiveDate) -> BsonDateTime {
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();
    BsonDateTime::from_millis(datetime.and_utc().timestamp_millis())
}

fn insert_student(database: &Database, rng: &mut impl Rng) -> Result<ObjectId> {
    let surname = SURNAMES.choose(rng).unwrap();
    let first_name = FIRST_NAMES.choose(rng).unwrap();
    let patronymic = PATRONYMICS.choose(rng).unwrap();

    let birth = NaiveDate::from_ymd_opt(
        rng.gen_range(1998..=2007),
        rng.gen_range(1..=12),
        rng.gen_range(1..=28),
    )
    .unwrap();

    let token: u32 = rng.gen();
    let email = format!("student.{token}@example.test");

    // У каждого студента будет от одного до трёх адресов.
    // Первый всегда основной и зарегистрирован по месту проживания.
    let address_count = rng.gen_range(1..=3);
    let mut addresses = Vec::with_capacity(address_count);

    for index in 0..address_count {
        let city = CITIES.choose(rng).unwrap();
        let street = STREETS.choose(rng).unwrap();
        let house = rng.gen_range(1..=150).to_string();

        let kind = match index {
            0 => "registration",
            1 => "temporary",
            _ => "home",
        };

        addresses.push(doc! {
            "kind": kind,
            "city": *city,
            "street": *street,
            "house": house,
            "is_primary": index == 0
        });
    }

    let phone_count = rng.gen_range(1..=3);

    let phones: Vec<String> = (0..phone_count)
        .map(|_| {
            format!(
                "+79{:09}",
                rng.gen_range(0..1_000_000_000u64)
            )
        })
        .collect();

    insert_one_get_id(
        database,
        "students",
        doc! {
            "surname": *surname,
            "first_name": *first_name,
            "patronymic": *patronymic,
            "birth_date": to_bson_date(birth),
            "email": email,
            "addresses": addresses,
            "phones": phones
        },
    )
}

fn insert_grades(
    database: &Database,
    enrollment_id: ObjectId,
    direction_id: ObjectId,
    rng: &mut impl Rng,
) -> Result<()> {
    let assignments: Vec<Document> = database
        .collection::<Document>("direction_subjects")
        .find(doc! { "direction_id": direction_id }, None)?
        .collect::<Result<_, _>>()?;

    for assignment in assignments {
        let ds_id = assignment.get_object_id("_id")?;
        let grade: Option<i32> = if rng.gen_bool(0.08) {
            None
        } else {
            Some(*[2i32, 3, 3, 4, 4, 4, 5, 5].choose(rng).unwrap())
        };
        let date = Utc::now().date_naive() - Duration::days(rng.gen_range(1..=180));

        let mut record = doc! {
            "enrollment_id": enrollment_id,
            "direction_subject_id": ds_id,
            "exam_date": to_bson_date(date)
        };
        match grade {
            Some(value) => { record.insert("grade", value); }
            None => { record.insert("grade", mongodb::bson::Bson::Null); }
        }

        database
            .collection::<Document>("grades")
            .insert_one(record, None)?;
    }
    Ok(())
}

fn create_lessons_and_attendance(database: &Database, rng: &mut impl Rng) -> Result<()> {
    let groups: Vec<Document> = database
        .collection::<Document>("student_groups")
        .find(doc! {}, None)?
        .collect::<Result<_, _>>()?;
    let slots: Vec<ObjectId> = database
        .collection::<Document>("lesson_slots")
        .find(doc! {}, None)?
        .collect::<Result<Vec<Document>, _>>()?
        .iter()
        .map(|doc| doc.get_object_id("_id").unwrap())
        .collect();
    let today = Utc::now().date_naive();

    for group in groups {
        let group_id = group.get_object_id("_id")?;
        let direction_id = group.get_object_id("direction_id")?;

        let subjects: Vec<ObjectId> = database
            .collection::<Document>("direction_subjects")
            .find(doc! { "direction_id": direction_id }, None)?
            .collect::<Result<Vec<Document>, _>>()?
            .iter()
            .map(|doc| doc.get_object_id("_id").unwrap())
            .collect();

        let enrollments: Vec<ObjectId> = database
            .collection::<Document>("enrollments")
            .find(doc! { "group_id": group_id }, None)?
            .collect::<Result<Vec<Document>, _>>()?
            .iter()
            .map(|doc| doc.get_object_id("_id").unwrap())
            .collect();

        if subjects.is_empty() {
            continue;
        }

        let mut occupied: HashMap<NaiveDate, Vec<ObjectId>> = HashMap::new();

        for offset in 1..=20 {
            let date = today - Duration::days(offset);
            let available: Vec<ObjectId> = slots
                .iter()
                .copied()
                .filter(|slot| {
                    !occupied
                        .get(&date)
                        .is_some_and(|used| used.contains(slot))
                })
                .collect();

            let Some(slot_id) = available.choose(rng).copied() else {
                continue;
            };
            let ds_id = *subjects.choose(rng).unwrap();

            let lesson_id = insert_one_get_id(
                database,
                "lessons",
                doc! {
                    "group_id": group_id,
                    "direction_subject_id": ds_id,
                    "lesson_date": to_bson_date(date),
                    "slot_id": slot_id
                },
            )?;

            occupied.entry(date).or_default().push(slot_id);

            for enrollment_id in &enrollments {
                let attended = rng.gen_bool(0.82);
                database
                    .collection::<Document>("attendance")
                    .insert_one(
                        doc! {
                            "lesson_id": lesson_id,
                            "enrollment_id": *enrollment_id,
                            "attended": attended
                        },
                        None,
                    )?;
            }
        }
    }
    Ok(())
}
