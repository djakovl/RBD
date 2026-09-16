use anyhow::Result;
use faculty_client::{config::DbConfig, db, selectors, tasks};

fn main() {
    if let Err(error) = run() {
        eprintln!("\nОшибка: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let settings = DbConfig::from_env()?;
    let mut client = db::connect(&settings)?;
    let (database, user) = db::connection_info(&mut client)?;

    println!("Подключено к БД: {database}; пользователь: {user}");

    loop {
        println!("\nДемонстрационный клиент факультета\n");
        for (code, name) in tasks::MENU {
            println!("{code:>2}. {name}");
        }
        println!(" 0. Выход");

        let choice = selectors::read_line("\nВыберите задачу: ")?;
        if choice == "0" {
            println!("До свидания.");
            break;
        }

        if let Err(error) = tasks::run(&mut client, &choice) {
            eprintln!("Ошибка выполнения задачи: {error:#}");
        }
    }
    Ok(())
}
