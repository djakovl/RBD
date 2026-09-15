use crate::config::MongoConfig;
use anyhow::{Context, Result};
use mongodb::sync::{Client, Database};

pub fn connect(settings: &MongoConfig) -> Result<Database> {
    let client = Client::with_uri_str(&settings.uri)
        .with_context(|| format!("не удалось подключиться к MongoDB по адресу {}", settings.uri))?;
    Ok(client.database(&settings.dbname))
}

/// В mongodb 2.8 list_collection_names принимает Option<Document> как
/// фильтр и сразу возвращает Result<Vec<String>> — без промежуточного
/// builder-объекта с методом .run(), как это было бы в mongodb 3.x.
pub fn connection_info(db: &Database) -> Result<(String, Vec<String>)> {
    let name = db.name().to_string();
    let collections = db.list_collection_names(None)?;
    Ok((name, collections))
}
