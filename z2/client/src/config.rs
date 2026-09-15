use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct MongoConfig {
    pub uri: String,
    pub dbname: String,
}

impl MongoConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            uri: env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            dbname: env::var("MONGO_DB").unwrap_or_else(|_| "faculty_db".to_string()),
        })
    }
}
