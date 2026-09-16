use crate::db::{ClientError, ClientResult};
use crate::ui::prompt;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Запрашивает у пользователя целое число (id) и возвращает ошибку
/// понятную вызывающему коду, если ввод нельзя разобрать.
pub fn read_id(label: &str) -> ClientResult<i64> {
    let raw = prompt(&format!("{label}:"));
    raw.parse::<i64>()
        .map_err(|_| ClientError::InvalidInput(format!("«{raw}» не является числом")))
}

/// Вариант `read_id` для 32-битных идентификаторов (ci_id, policy_id).
pub fn read_id_i32(label: &str) -> ClientResult<i32> {
    let raw = prompt(&format!("{label}:"));
    raw.parse::<i32>()
        .map_err(|_| ClientError::InvalidInput(format!("«{raw}» не является числом")))
}

/// Запрашивает непустую строку; повторяет запрос, пока значение не будет введено.
pub fn read_non_empty(label: &str) -> String {
    loop {
        let value = prompt(&format!("{label}:"));
        if !value.is_empty() {
            return value;
        }
        println!("Значение не может быть пустым, повторите ввод.");
    }
}

/// Запрашивает значение метрики как точное десятичное число.
///
/// Столбец `anomaly_event.value` в БД имеет тип `NUMERIC`. Crate `postgres`
/// не реализует сериализацию `f64` в `NUMERIC` (только в `DOUBLE PRECISION`),
/// поэтому значение читается и передаётся как `rust_decimal::Decimal` —
/// он поддерживает `NUMERIC` через фичу `db-postgres` без потери точности.
pub fn read_metric_value() -> ClientResult<Decimal> {
    let raw = prompt("значение:");
    Decimal::from_str(&raw)
        .map_err(|_| ClientError::InvalidInput(format!("«{raw}» не является числом")))
}

/// Запрашивает критичность и проверяет, что введено одно из допустимых значений.
pub fn read_criticality() -> ClientResult<String> {
    let value = prompt("критичность (high/medium/low):");
    match value.as_str() {
        "high" | "medium" | "low" => Ok(value),
        _ => Err(ClientError::InvalidInput(
            "критичность должна быть high, medium или low".to_owned(),
        )),
    }
}

/// Запрашивает результат действия самовосстановления и проверяет значение.
pub fn read_remediation_result() -> ClientResult<String> {
    let value = prompt("результат (success/failed):");
    match value.as_str() {
        "success" | "failed" => Ok(value),
        _ => Err(ClientError::InvalidInput(
            "результат должен быть success или failed".to_owned(),
        )),
    }
}

/// Запрашивает приоритет политики, по умолчанию возвращает 1 при пустом/некорректном вводе.
pub fn read_priority_or_default(default: i32) -> i32 {
    prompt("приоритет:").parse::<i32>().unwrap_or(default)
}