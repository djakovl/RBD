//! Базовые примитивы ввода-вывода: чтение строки, пауза, обёртка операций.

use crate::db::ClientError;
use postgres::Client;
use std::io::{self, Write};

/// Печатает подсказку без перевода строки и считывает одну строку ввода,
/// обрезая пробелы по краям.
pub fn prompt(message: &str) -> String {
    print!("{message} ");
    io::stdout().flush().expect("stdout недоступен");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("stdin недоступен");
    input.trim().to_owned()
}

/// Ожидает нажатия Enter перед возвратом в меню.
pub fn pause() {
    let _ = prompt("\n[Enter — продолжить]");
}

/// Выполняет операцию над подключением, печатает читаемое сообщение об ошибке
/// (как из БД, так и из-за некорректного ввода) и делает паузу перед возвратом в меню.
pub fn run_operation<F>(client: &mut Client, operation: F)
where
    F: Fn(&mut Client) -> Result<(), ClientError>,
{
    if let Err(error) = operation(client) {
        println!("[ошибка операции] {error}");
    }
    pause();
}
