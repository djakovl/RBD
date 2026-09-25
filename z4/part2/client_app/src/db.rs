use crate::config::DbConfig;
use anyhow::{Context, Result};
use postgres::{Client, Config, NoTls};

pub fn connect(settings: &DbConfig) -> Result<Client> {
    let mut config = Config::new();
    config
        .host(&settings.host)
        .port(settings.port)
        .dbname(&settings.dbname)
        .user(&settings.user);

    if !settings.password.is_empty() {
        config.password(&settings.password);
    }

    config.connect(NoTls).with_context(|| {
        format!(
            "не удалось подключиться к PostgreSQL {}:{}/{} пользователем {}",
            settings.host, settings.port, settings.dbname, settings.user
        )
    })
}

pub fn connection_info(client: &mut Client) -> Result<(String, String)> {
    let row = client.query_one("SELECT current_database()::text, current_user::text", &[])?;
    Ok((row.get(0), row.get(1)))
}
