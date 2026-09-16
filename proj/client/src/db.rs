use postgres::{Client, NoTls};
use std::fmt;

/// Ошибки, которые может вернуть любая операция клиента: либо ошибка
/// драйвера postgres, либо ошибка ввода, допущенная пользователем.
#[derive(Debug)]
pub enum ClientError {
    Database(postgres::Error),
    InvalidInput(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Database(error) => {
                // as_db_error() достаёт структурированный ответ сервера:
                // SQLSTATE-код, основное сообщение и детали. Без него
                // Display у postgres::Error иногда сокращается до "db error".
                if let Some(db_error) = error.as_db_error() {
                    write!(
                        f,
                        "ошибка БД [{}]: {}{}",
                        db_error.code().code(),
                        db_error.message(),
                        db_error
                            .detail()
                            .map(|d| format!(" ({d})"))
                            .unwrap_or_default()
                    )
                } else {
                    write!(f, "ошибка БД: {error}")
                }
            }
            ClientError::InvalidInput(message) => write!(f, "некорректный ввод: {message}"),
        }
    }
}

impl From<postgres::Error> for ClientError {
    fn from(error: postgres::Error) -> Self {
        ClientError::Database(error)
    }
}

pub type ClientResult<T> = Result<T, ClientError>;

/// Пытается установить соединение с базой по переданному URL.
/// Возвращает готовый `Client` либо ошибку драйвера вызывающему коду.
pub fn connect(url: &str) -> Result<Client, postgres::Error> {
    Client::connect(url, NoTls)
}
