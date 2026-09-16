use anyhow::{bail, Result};
use postgres::Client;
use std::io::{self, Write};

use crate::queries;

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

fn choose_id(client: &mut Client, title: &str, sql: &str, allow_all: bool) -> Result<Option<i32>> {
    let rows = client.query(sql, &[])?;
    if rows.is_empty() {
        bail!("справочник пуст: {title}");
    }

    println!("\n{title}");
    if allow_all { println!("0. Все"); }
    for (index, row) in rows.iter().enumerate() {
        let label: String = row.get("label");
        println!("{}. {label}", index + 1);
    }

    loop {
        let raw = read_line("Введите номер: ")?;
        if allow_all && raw == "0" { return Ok(None); }
        if let Ok(number) = raw.parse::<usize>() {
            if (1..=rows.len()).contains(&number) {
                return Ok(Some(rows[number - 1].get("id")));
            }
        }
        println!("Такого пункта нет.");
    }
}

pub fn direction(client: &mut Client) -> Result<i32> {
    Ok(choose_id(client, "Выберите направление:", queries::DIRECTIONS, false)?.expect("выбор обязателен"))
}

pub fn group(client: &mut Client) -> Result<i32> {
    Ok(choose_id(client, "Выберите группу:", queries::GROUPS, false)?.expect("выбор обязателен"))
}

pub fn subject(client: &mut Client) -> Result<i32> {
    Ok(choose_id(client, "Выберите предмет:", queries::SUBJECTS, false)?.expect("выбор обязателен"))
}

pub fn teacher(client: &mut Client, allow_all: bool) -> Result<Option<i32>> {
    choose_id(client, "Выберите преподавателя:", queries::TEACHERS, allow_all)
}

pub fn student_search(client: &mut Client) -> Result<Option<i64>> {
    loop {
        let text = read_line("\nВведите часть ФИО или email студента (0 — назад): ")?;
        if text == "0" { return Ok(None); }
        if text.is_empty() {
            println!("Введите хотя бы один символ.");
            continue;
        }

        let rows = client.query(
            "SELECT id, concat_ws(' ', surname, first_name, patronymic) || ' — ' || email AS label \
             FROM faculty.students \
             WHERE concat_ws(' ', surname, first_name, patronymic) ILIKE '%' || $1 || '%' \
                OR email ILIKE '%' || $1 || '%' \
             ORDER BY surname, first_name, patronymic LIMIT 15",
            &[&text],
        )?;
        if rows.is_empty() {
            println!("Студенты не найдены.");
            continue;
        }

        println!("\nНайденные студенты:");
        for (index, row) in rows.iter().enumerate() {
            let label: String = row.get("label");
            println!("{}. {label}", index + 1);
        }
        println!("0. Назад");

        loop {
            let choice = read_line("Введите номер: ")?;
            if choice == "0" { return Ok(None); }
            if let Ok(number) = choice.parse::<usize>() {
                if (1..=rows.len()).contains(&number) {
                    return Ok(Some(rows[number - 1].get("id")));
                }
            }
            println!("Такого пункта нет.");
        }
    }
}
