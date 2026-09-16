// Демонстрационный клиент распределённой БД «объект КИИ ↔ центр».
// Подключения можно переопределить переменными окружения:
//   RBD_CENTER — центр, RBD_SITE — объект.

mod center;
mod site;
mod ui;

use std::env;
use ui::mask_url;

const DEFAULT_CENTER: &str = "postgres://center:center_pass@127.0.0.1:5432/rbd_center";
const DEFAULT_SITE: &str = "postgres://site:site_pass@127.0.0.1:5433/rbd_site";

fn main() {
    let center_url = env::var("RBD_CENTER").unwrap_or_else(|_| DEFAULT_CENTER.into());
    let site_url = env::var("RBD_SITE").unwrap_or_else(|_| DEFAULT_SITE.into());

    println!("Демонстрационный клиент распределённой БД «объект КИИ ↔ центр»");
    println!("Центр:  {}", mask_url(&center_url));
    println!("Объект: {}", mask_url(&site_url));

    loop {
        println!("\n=== Главное меню ===");
        println!("1) Режим ЦЕНТРА (справочники, журнал, квитирование)");
        println!("2) Режим ОБЪЕКТА (очередь аномалий, самовосстановление)");
        println!("0) Выход");
        match ui::prompt(">").as_str() {
            "1" => center::center_mode(&center_url),
            "2" => site::site_mode(&site_url),
            "0" => break,
            _ => {}
        }
    }
}
