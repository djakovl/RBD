use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub password: String,
}

impl DbConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let port = env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .context("DB_PORT должен быть числом от 1 до 65535")?;

        Ok(Self {
            host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port,
            dbname: env::var("DB_NAME").unwrap_or_else(|_| "faculty_db".to_string()),
            user: env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: env::var("DB_PASSWORD").unwrap_or_default(),
        })
    }
}
