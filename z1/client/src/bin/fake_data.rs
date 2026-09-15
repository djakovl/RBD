use anyhow::{Context, Result};
use chrono::{Duration, NaiveDate, Utc};
use faculty_client::{config::DbConfig, db};
use postgres::Transaction;
use rand::{seq::SliceRandom, Rng};
use std::collections::HashMap;

const SURNAMES: &[&str] = &[
    "Иванов",
    "Петров",
    "Сидоров",
    "Смирнов",
    "Кузнецов",
    "Попов",
    "Соколов",
    "Лебедев",
    "Козлов",
    "Новиков",
    "Морозов",
    "Волков",
    "Орлов",
    "Васильев",
    "Зайцев",
    "Павлов",
    "Семёнов",
    "Голубев",
    "Виноградов",
    "Богданов",
];
const FIRST_NAMES: &[&str] = &[
    "Александр",
    "Иван",
    "Максим",
    "Артём",
    "Дмитрий",
    "Никита",
    "Михаил",
    "Анна",
    "Мария",
    "Елена",
    "Дарья",
    "Ольга",
    "Полина",
    "София",
    "Алина",
];
const PATRONYMICS: &[&str] = &[
    "Александрович",
    "Иванович",
    "Сергеевич",
    "Алексеевич",
    "Дмитриевич",
    "Михайлович",
    "Александровна",
    "Ивановна",
    "Сергеевна",
    "Алексеевна",
    "Дмитриевна",
    "Михайловна",
];
const CITIES: &[&str] = &[
    "Москва",
    "Тула",
    "Калуга",
    "Рязань",
    "Тверь",
    "Коломна",
    "Подольск",
];
const STREETS: &[&str] = &[
    "Ленина",
    "Советская",
    "Мира",
    "Молодёжная",
    "Садовая",
    "Школьная",
    "Лесная",
    "Центральная",
];

fn main() {
    if let Err(error) = run() {
        eprintln!("Ошибка генератора: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let settings = DbConfig::from_env()?;
    let mut client = db::connect(&settings)?;
    let mut tx = client.transaction()?;
    let reset = std::env::args().any(|arg| arg == "--reset");

    if reset {
        tx.batch_execute(
            "TRUNCATE faculty.attendance, faculty.lessons, faculty.grades, \
             faculty.enrollments, faculty.student_phones, faculty.student_addresses, \
             faculty.students RESTART IDENTITY CASCADE",
        )?;
    } else {
        let existing: i64 = tx
            .query_one("SELECT count(*) FROM faculty.students", &[])?
            .get(0);
        if existing > 0 {
            anyhow::bail!(
                "студенты уже существуют; используйте cargo run --bin fake_data -- --reset"
            );
        }
    }

    let group_rows = tx.query(
        "SELECT id, direction_id FROM faculty.student_groups ORDER BY id",
        &[],
    )?;
    let funding_rows = tx.query("SELECT id FROM faculty.funding_types ORDER BY id", &[])?;
    if group_rows.is_empty() || funding_rows.is_empty() {
        anyhow::bail!("сначала выполните sql/01_schema.sql и sql/02_reference_data.sql");
    }
    let funding_ids: Vec<i16> = funding_rows.iter().map(|r| r.get(0)).collect();
    let mut rng = rand::thread_rng();
    let mut enrollment_count = 0usize;

    for group in &group_rows {
        let group_id: i32 = group.get(0);
        let count = rng.gen_range(15..=24);
        for _ in 0..count {
            let student_id = insert_student(&mut tx, &mut rng)?;
            let funding_id = *funding_ids.choose(&mut rng).unwrap();
            let enrollment_id: i64 = tx.query_one(
                "INSERT INTO faculty.enrollments(student_id, group_id, funding_type_id) VALUES ($1,$2,$3) RETURNING id",
                &[&student_id, &group_id, &funding_id],
            )?.get(0);
            insert_grades(&mut tx, enrollment_id, group.get(1), &mut rng)?;
            enrollment_count += 1;
        }
    }

    create_lessons_and_attendance(&mut tx, &mut rng)?;
    tx.commit()?;
    println!("Создано студентов/зачислений: {enrollment_count}. Оценки, занятия и посещаемость добавлены.");
    Ok(())
}

fn insert_student(tx: &mut Transaction<'_>, rng: &mut impl Rng) -> Result<i64> {
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
    let student_id: i64 = tx.query_one(
        "INSERT INTO faculty.students(surname, first_name, patronymic, birth_date, email) VALUES ($1,$2,$3,$4,$5) RETURNING id",
        &[surname, first_name, patronymic, &birth, &email],
    )?.get(0);

    let city = CITIES.choose(rng).unwrap();
    let street = STREETS.choose(rng).unwrap();
    let house = rng.gen_range(1..=150).to_string();
    tx.execute(
        "INSERT INTO faculty.student_addresses(student_id, city, street, house) VALUES ($1,$2,$3,$4)",
        &[&student_id, city, street, &house],
    )?;
    for _ in 0..rng.gen_range(1..=3) {
        let phone = format!("+79{:09}", rng.gen_range(0..1_000_000_000u64));
        tx.execute(
            "INSERT INTO faculty.student_phones(student_id, phone_number) VALUES ($1,$2) ON CONFLICT DO NOTHING",
            &[&student_id, &phone],
        )?;
    }
    Ok(student_id)
}

fn insert_grades(
    tx: &mut Transaction<'_>,
    enrollment_id: i64,
    direction_id: i32,
    rng: &mut impl Rng,
) -> Result<()> {
    let subjects = tx.query(
        "SELECT id FROM faculty.direction_subjects WHERE direction_id=$1",
        &[&direction_id],
    )?;
    for row in subjects {
        let ds_id: i32 = row.get(0);
        let grade: Option<i16> = if rng.gen_bool(0.08) {
            None
        } else {
            Some(*[2i16, 3, 3, 4, 4, 4, 5, 5].choose(rng).unwrap())
        };
        let date = Utc::now().date_naive() - Duration::days(rng.gen_range(1..=180));
        tx.execute(
            "INSERT INTO faculty.grades(enrollment_id, direction_subject_id, grade, exam_date) VALUES ($1,$2,$3,$4)",
            &[&enrollment_id, &ds_id, &grade, &date],
        )?;
    }
    Ok(())
}

fn create_lessons_and_attendance(tx: &mut Transaction<'_>, rng: &mut impl Rng) -> Result<()> {
    let groups = tx.query(
        "SELECT id, direction_id FROM faculty.student_groups ORDER BY id",
        &[],
    )?;
    let slots: Vec<i16> = tx
        .query("SELECT id FROM faculty.lesson_slots ORDER BY id", &[])?
        .iter()
        .map(|r| r.get(0))
        .collect();
    let today = Utc::now().date_naive();

    for group in groups {
        let group_id: i32 = group.get(0);
        let direction_id: i32 = group.get(1);
        let subjects: Vec<i32> = tx
            .query(
                "SELECT id FROM faculty.direction_subjects WHERE direction_id=$1 ORDER BY id",
                &[&direction_id],
            )?
            .iter()
            .map(|r| r.get(0))
            .collect();
        let enrollments: Vec<i64> = tx
            .query(
                "SELECT id FROM faculty.enrollments WHERE group_id=$1",
                &[&group_id],
            )?
            .iter()
            .map(|r| r.get(0))
            .collect();
        let mut occupied: HashMap<NaiveDate, Vec<i16>> = HashMap::new();

        for offset in 1..=20 {
            let date = today - Duration::days(offset);
            let available: Vec<i16> = slots
                .iter()
                .copied()
                .filter(|slot| !occupied.get(&date).is_some_and(|used| used.contains(slot)))
                .collect();
            let Some(slot_id) = available.choose(rng).copied() else {
                continue;
            };
            let ds_id = *subjects
                .choose(rng)
                .context("у направления нет предметов")?;
            let lesson_id: i64 = tx.query_one(
                "INSERT INTO faculty.lessons(group_id, direction_subject_id, lesson_date, slot_id) VALUES ($1,$2,$3,$4) RETURNING id",
                &[&group_id, &ds_id, &date, &slot_id],
            )?.get(0);
            occupied.entry(date).or_default().push(slot_id);
            for enrollment_id in &enrollments {
                let attended = rng.gen_bool(0.82);
                tx.execute(
                    "INSERT INTO faculty.attendance(lesson_id, enrollment_id, attended) VALUES ($1,$2,$3)",
                    &[&lesson_id, enrollment_id, &attended],
                )?;
            }
        }
    }
    Ok(())
}
