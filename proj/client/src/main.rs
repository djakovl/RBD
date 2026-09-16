//! Точка входа: только запуск приложения и главное меню.

use rbd_client::config::Endpoints;
use rbd_client::ui::prompt;
use rbd_client::{center, site};

fn main() {
    let endpoints = Endpoints::from_env();
    println!("Демонстрационный клиент распределённой БД «объект КИИ ↔ центр»");
    println!("Центр:  {}", endpoints.masked_center());
    println!("Объект: {}", endpoints.masked_site());

    loop {
        println!("\n=== Главное меню ===");
        println!("1) Режим ЦЕНТРА (справочники, журнал, квитирование)");
        println!("2) Режим ОБЪЕКТА (очередь аномалий, самовосстановление)");
        println!("0) Выход");

        match prompt("> ").as_str() {
            "1" => center::run(&endpoints.center),
            "2" => site::run(&endpoints.site),
            "0" => break,
            _ => println!("Неизвестная команда, попробуйте снова."),
        }
    }
}
