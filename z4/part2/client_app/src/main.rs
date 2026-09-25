use anyhow::{Context, Result};
use chrono::NaiveDate;
use faculty_client::{config::DbConfig, db, selectors, tasks};
use postgres::{Client, NoTls};
use std::env;

fn personal_client() -> Result<Client> {
    let host=env::var("PERSONAL_HOST").unwrap_or_else(|_|"127.0.0.1".into());
    let port=env::var("PERSONAL_PORT").unwrap_or_else(|_|"54324".into()).parse()?;
    let dbname=env::var("PERSONAL_DB").unwrap_or_else(|_|"personal_db".into());
    let user=env::var("PERSONAL_USER").unwrap_or_else(|_|"postgres".into());
    let password=env::var("PERSONAL_PASSWORD").unwrap_or_else(|_|"postgres".into());
    postgres::Config::new().host(&host).port(port).dbname(&dbname).user(&user).password(&password).connect(NoTls)
        .with_context(|| format!("нет подключения к персональной БД {host}:{port}/{dbname}"))
}
fn stage_students(university: &mut Client, personal: &mut Client) -> Result<()> {
    university.batch_execute("CREATE TEMP TABLE pg_temp.students (id bigint PRIMARY KEY,surname varchar(100),first_name varchar(100),patronymic varchar(100),birth_date date,email varchar(254)) ON COMMIT PRESERVE ROWS; TRUNCATE pg_temp.students;")?;
    let rows=personal.query("SELECT id,surname,first_name,patronymic,birth_date,email FROM faculty.students",&[])?;
    let mut tx=university.transaction()?;
    for r in rows { let id:i64=r.get(0); let surname:String=r.get(1); let first:String=r.get(2); let patronymic:Option<String>=r.get(3); let birth:NaiveDate=r.get(4); let email:String=r.get(5);
      tx.execute("INSERT INTO pg_temp.students VALUES($1,$2,$3,$4,$5,$6)",&[&id,&surname,&first,&patronymic,&birth,&email])?; }
    tx.commit()?;
    Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("Ошибка: {e:#}");std::process::exit(1)}}
fn run()->Result<()> {
 let cfg=DbConfig::from_env()?; let mut university=db::connect(&cfg)?; let mut personal=personal_client()?;
 stage_students(&mut university,&mut personal)?;
 println!("Часть 2: ПДн прочитаны приложением из personal и загружены во временную таблицу текущей сессии university. FDW не используется.");
 loop { println!("\nКлиент факультета — распределённая обработка (уровень приложения)\n"); for (c,n) in tasks::MENU {println!("{c:>2}. {n}")} println!(" 0. Выход"); let choice=selectors::read_line("\nВыберите задачу: ")?; if choice=="0"{break}; if let Err(e)=tasks::run(&mut university,&choice){eprintln!("Ошибка: {e:#}")} }
 Ok(())
}
