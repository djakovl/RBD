use postgres::Client;
use std::io::{self, Write};
use tabled::{builder::Builder, settings::Style};

pub fn prompt(msg: &str) -> String {
    if !msg.is_empty() { print!("{} ", msg); io::stdout().flush().unwrap(); }
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

pub fn pause() { prompt("\n[Enter — продолжить]"); }

pub fn mask_url(u: &str) -> String {
    if let Some(p) = u.find("://") {
        let rest = &u[p + 3..];
        if let Some(at) = rest.find('@') {
            if let Some(colon) = rest[..at].find(':') {
                return format!("{}{}:***{}", &u[..p + 3], &rest[..colon], &rest[at..]);
            }
        }
    }
    u.to_string()
}

pub fn op<F>(c: &mut Client, f: F) where F: Fn(&mut Client) -> Result<(), postgres::Error> {
    if let Err(e) = f(c) { println!("[ошибка операции] {}", e); }
    pause();
}

pub fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    let mut builder = Builder::default();
    builder.push_record(headers.iter().copied());
    for row in rows { builder.push_record(row); }
    let mut table = builder.build();
    table.with(Style::ascii());
    println!("{table}");
}
