use anyhow::{bail, Result};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::options::FindOptions;
use mongodb::sync::Database;
use std::io::{self, Write};

pub fn read_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(value.trim().to_owned())
}

pub fn read_i32(prompt: &str, min: i32, max: i32) -> Result<i32> {
    loop {
        let raw = read_line(prompt)?;
        match raw.parse::<i32>() {
            Ok(value) if (min..=max).contains(&value) => return Ok(value),
            _ => println!("Введите число от {min} до {max}."),
        }
    }
}

pub fn read_letter(prompt: &str) -> Result<String> {
    loop {
        let value = read_line(prompt)?;
        if value.chars().count() == 1 && value.chars().all(char::is_alphabetic) {
            return Ok(value);
        }
        println!("Введите ровно одну букву.");
    }
}

/// Общий выбор документа из справочной коллекции по одному текстовому полю.
/// В mongodb 2.8 find() принимает второй аргумент — Option<FindOptions>,
/// поэтому сортировка задаётся через builder FindOptions, а не через
/// метод .sort() на результате find(), как в mongodb 3.x.
fn choose_id(
    db: &Database,
    title: &str,
    collection: &str,
    label_field: &str,
    sort_field: &str,
    allow_all: bool,
) -> Result<Option<ObjectId>> {
    let coll = db.collection::<mongodb::bson::Document>(collection);
    let options = FindOptions::builder()
        .sort(doc! { sort_field: 1 })
        .build();
    let cursor = coll.find(doc! {}, options)?;
    let rows: Vec<mongodb::bson::Document> = cursor.collect::<Result<_, _>>()?;

    if rows.is_empty() {
        bail!("справочник пуст: {title}");
    }

    println!("\n{title}");
    if allow_all {
        println!("0. Все");
    }
    for (index, row) in rows.iter().enumerate() {
        let label = row.get_str(label_field).unwrap_or("?");
        println!("{}. {label}", index + 1);
    }

    loop {
        let raw = read_line("Введите номер: ")?;
        if allow_all && raw == "0" {
            return Ok(None);
        }
        if let Ok(number) = raw.parse::<usize>() {
            if (1..=rows.len()).contains(&number) {
                return Ok(Some(rows[number - 1].get_object_id("_id")?));
            }
        }
        println!("Такого пункта нет.");
    }
}

pub fn direction(db: &Database) -> Result<ObjectId> {
    Ok(choose_id(db, "Выберите направление:", "directions", "name", "name", false)?
        .expect("выбор обязателен"))
}

pub fn group(db: &Database) -> Result<ObjectId> {
    Ok(
        choose_id(db, "Выберите группу:", "student_groups", "group_number", "group_number", false)?
            .expect("выбор обязателен"),
    )
}

pub fn subject(db: &Database) -> Result<ObjectId> {
    Ok(choose_id(db, "Выберите предмет:", "subjects", "name", "name", false)?
        .expect("выбор обязателен"))
}

/// Для преподавателей label составляется из трёх полей, поэтому используем
/// отдельную функцию вместо общей choose_id (там нет готового текстового поля).
pub fn teacher(db: &Database, allow_all: bool) -> Result<Option<ObjectId>> {
    let coll = db.collection::<mongodb::bson::Document>("teachers");
    let options = FindOptions::builder()
        .sort(doc! { "surname": 1, "first_name": 1, "patronymic": 1 })
        .build();
    let cursor = coll.find(doc! {}, options)?;
    let rows: Vec<mongodb::bson::Document> = cursor.collect::<Result<_, _>>()?;

    if rows.is_empty() {
        bail!("в базе нет преподавателей");
    }

    println!("\nВыберите преподавателя:");
    if allow_all {
        println!("0. Все");
    }
    for (index, row) in rows.iter().enumerate() {
        let surname = row.get_str("surname").unwrap_or("?");
        let first_name = row.get_str("first_name").unwrap_or("?");
        let patronymic = row.get_str("patronymic").unwrap_or("");
        println!("{}. {surname} {first_name} {patronymic}", index + 1);
    }

    loop {
        let raw = read_line("Введите номер: ")?;
        if allow_all && raw == "0" {
            return Ok(None);
        }
        if let Ok(number) = raw.parse::<usize>() {
            if (1..=rows.len()).contains(&number) {
                return Ok(Some(rows[number - 1].get_object_id("_id")?));
            }
        }
        println!("Такого пункта нет.");
    }
}

/// Поиск студента по части ФИО или email вместо вывода полного списка —
/// то же решение, что и в PostgreSQL-версии клиента (задача 19).
pub fn student_search(db: &Database) -> Result<Option<ObjectId>> {
    let coll = db.collection::<mongodb::bson::Document>("students");

    loop {
        let text = read_line("\nВведите часть ФИО или email студента (0 — назад): ")?;
        if text == "0" {
            return Ok(None);
        }
        if text.is_empty() {
            println!("Введите хотя бы один символ.");
            continue;
        }

        let regex = mongodb::bson::Regex {
            pattern: regex_escape(&text),
            options: "i".to_string(),
        };
        let filter = doc! {
            "$or": [
                { "surname": { "$regex": regex.clone() } },
                { "first_name": { "$regex": regex.clone() } },
                { "email": { "$regex": regex } }
            ]
        };

        let options = FindOptions::builder()
            .sort(doc! { "surname": 1, "first_name": 1 })
            .limit(15)
            .build();
        let cursor = coll.find(filter, options)?;
        let rows: Vec<mongodb::bson::Document> = cursor.collect::<Result<_, _>>()?;

        if rows.is_empty() {
            println!("Студенты не найдены.");
            continue;
        }

        println!("\nНайденные студенты:");
        for (index, row) in rows.iter().enumerate() {
            let surname = row.get_str("surname").unwrap_or("?");
            let first_name = row.get_str("first_name").unwrap_or("?");
            let patronymic = row.get_str("patronymic").unwrap_or("");
            let email = row.get_str("email").unwrap_or("?");
            println!("{}. {surname} {first_name} {patronymic} — {email}", index + 1);
        }
        println!("0. Назад");

        loop {
            let choice = read_line("Введите номер: ")?;
            if choice == "0" {
                return Ok(None);
            }
            if let Ok(number) = choice.parse::<usize>() {
                if (1..=rows.len()).contains(&number) {
                    return Ok(Some(rows[number - 1].get_object_id("_id")?));
                }
            }
            println!("Такого пункта нет.");
        }
    }
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
