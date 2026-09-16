// Общие функции интерфейса: ввод, пауза, маскировка строки подключения,
// обёртка запуска операции с перехватом ошибок (отказ узла / обрыв FDW
// не роняют клиент, а выводятся сообщением).

use postgres::Client;
use std::io::{self, Write};

pub fn prompt(msg: &str) -> String {
    if !msg.is_empty() {
        print!("{} ", msg);
        io::stdout().flush().unwrap();
    }
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}

pub fn pause() {
    prompt("\n[Enter — продолжить]");
}

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

pub fn op<F>(c: &mut Client, f: F)
where
    F: Fn(&mut Client) -> Result<(), postgres::Error>,
{
    if let Err(e) = f(c) {
        println!("[ошибка операции] {}", e);
    }
    pause();
}
