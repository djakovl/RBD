use anyhow::Result;
use faculty_mongo_client::{config::MongoConfig, db, selectors, tasks};

fn main() {
    if let Err(error) = run() {
        eprintln!("\nОшибка: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let settings = MongoConfig::from_env()?;
    let database = db::connect(&settings)?;
    let (name, collections) = db::connection_info(&database)?;

    println!("Подключено к БД: {name}; коллекций: {}", collections.len());

    loop {
        println!("\nДемонстрационный клиент факультета (MongoDB)\n");
        for (code, title) in tasks::MENU {
            println!("{code:>2}. {title}");
        }
        println!(" 0. Выход");

        let choice = selectors::read_line("\nВыберите задачу: ")?;
        if choice == "0" {
            println!("До свидания.");
            break;
        }

        if let Err(error) = tasks::run(&database, &choice) {
            eprintln!("Ошибка выполнения задачи: {error:#}");
        }
    }
    Ok(())
}
