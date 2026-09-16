use anyhow::Result;
use postgres::{types::ToSql, Client};

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

fn show(client: &mut Client, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<()> {
    let rows = client.query(sql, params)?;
    table::print_rows(&rows);
    Ok(())
}

pub fn run(client: &mut Client, choice: &str) -> Result<()> {
    match choice {
        "1" => {
            let id = selectors::direction(client)?;
            show(client, queries::Q01, &[&id])
        }
        "2" => {
            let pattern = format!("{}%", selectors::read_letter("Первая буква фамилии: ")?);
            show(client, queries::Q02, &[&pattern])
        }
        "3" => {
            let month = selectors::read_i32("Номер месяца (1-12): ", 1, 12)?;
            show(client, queries::Q03, &[&month])
        }
        "4" => {
            let id = selectors::group(client)?;
            show(client, queries::Q04, &[&id])
        }
        "5" => show(client, queries::Q05, &[]),
        "6" => show(client, queries::Q06, &[]),
        "7" => {
            let id = selectors::direction(client)?;
            show(client, queries::Q07, &[&id])
        }
        "8" => {
            let subject = selectors::subject(client)?;
            let teacher = selectors::teacher(client, true)?;
            show(client, queries::Q08, &[&subject, &teacher])
        }
        "9" => show(client, queries::Q09, &[]),
        "10" => {
            let id = selectors::teacher(client, false)?.expect("выбор обязателен");
            show(client, queries::Q10, &[&id])
        }
        "11" => {
            let id = selectors::subject(client)?;
            show(client, queries::Q11, &[&id])
        }
        "12" => {
            let id = selectors::subject(client)?;
            show(client, queries::Q12, &[&id])
        }
        "13" => show(client, queries::Q13, &[]),
        "14" => {
            let id = selectors::direction(client)?;
            show(client, queries::Q14, &[&id])
        }
        "15" => {
            let count = selectors::read_i32("Минимум несданных экзаменов (1-20): ", 1, 20)? as i64;
            show(client, queries::Q15, &[&count])
        }
        "16" => {
            let id = selectors::subject(client)?;
            show(client, queries::Q16, &[&id])
        }
        "17" => {
            let id = selectors::subject(client)?;
            show(client, queries::Q17, &[&id])
        }
        "18" => {
            let id = selectors::teacher(client, false)?.expect("выбор обязателен");
            show(client, queries::Q18, &[&id])
        }
        "19" => {
            let Some(id) = selectors::student_search(client)? else {
                return Ok(());
            };
            show(client, queries::Q19, &[&id])
        }
        _ => {
            println!("Неизвестная задача.");
            Ok(())
        }
    }
}
