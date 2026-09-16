//! Вывод табличных данных в терминал в виде ASCII-таблиц (`tabled`).

use tabled::builder::Builder;
use tabled::settings::Style;

/// Строит и печатает ASCII-таблицу по заголовкам и строкам данных.
/// Каждая строка должна содержать столько же элементов, сколько заголовков.
pub fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    let mut builder = Builder::default();
    builder.push_record(headers.iter().copied());
    for row in rows {
        builder.push_record(row);
    }
    let mut table = builder.build();
    table.with(Style::ascii());
    println!("{table}");
}

/// Печатает сообщение, если данных для таблицы нет, вместо пустой таблицы.
pub fn print_table_or_message(headers: &[&str], rows: Vec<Vec<String>>, empty_message: &str) {
    if rows.is_empty() {
        println!("{empty_message}");
    } else {
        print_table(headers, rows);
    }
}
