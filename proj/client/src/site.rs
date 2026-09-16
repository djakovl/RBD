//! Режим ОБЪЕКТА КИИ: очередь аномалий, автономная работа при разрыве
//! канала связи с центром, выполнение действий самовосстановления.

use crate::db::{self, ClientResult};
use crate::models::{AnomalyQueueEntry, MatchedPolicy, OpenRemediationAction, RemediationHistoryEntry};
use crate::queries;
use crate::selectors::{read_id, read_id_i32, read_metric_value, read_non_empty, read_remediation_result};
use crate::table::print_table_or_message;
use crate::ui::{pause, prompt, run_operation};
use postgres::Client;

/// Запускает интерактивное меню режима объекта на переданном подключении.
pub fn run(url: &str) {
    let mut client = match db::connect(url) {
        Ok(client) => client,
        Err(error) => {
            println!("[ОБЪЕКТ НЕДОСТУПЕН] {error}");
            pause();
            return;
        }
    };
    println!("[объект] подключено");

    loop {
        print_menu();
        match prompt("> ").as_str() {
            "1" => run_operation(&mut client, check_channel_to_center),
            "2" => run_operation(&mut client, register_anomaly),
            "3" => run_operation(&mut client, show_anomaly_queue),
            "4" => run_operation(&mut client, finish_remediation_action),
            "5" => run_operation(&mut client, show_remediation_history),
            "0" => break,
            _ => println!("Неизвестная команда, попробуйте снова."),
        }
    }
}

fn print_menu() {
    println!("\n--- ОБЪЕКТ КИИ: очередь аномалий, самовосстановление ---");
    println!("1) Статус канала до центра (FDW)");
    println!("2) Зарегистрировать аномалию");
    println!("3) Очередь событий");
    println!("4) Завершить действие самовосстановления");
    println!("5) Все действия");
    println!("0) Назад");
}

fn check_channel_to_center(client: &mut Client) -> ClientResult<()> {
    match client.query(queries::SELECT_CENTER_CI_COUNT, &[]) {
        Ok(rows) => {
            let count: i64 = rows[0].get(0);
            println!("[объект] канал ЕСТЬ: FDW читает {count} КЕ");
        }
        Err(_) => {
            println!("[объект] канала НЕТ: работа по локальной реплике");
        }
    }
    Ok(())
}

fn register_anomaly(client: &mut Client) -> ClientResult<()> {
    let ci_id = read_id_i32("ci_id")?;
    let metric = read_non_empty("метрика");
    let value = read_metric_value()?;
    let anomaly_class = read_non_empty("класс");

    let event_id: i64 = client
        .query_one(queries::INSERT_ANOMALY_EVENT, &[&ci_id, &metric, &value, &anomaly_class])?
        .get(0);
    println!("[объект] событие #{event_id} записано локально (pending)");

    apply_matching_policy_if_any(client, ci_id, &anomaly_class, event_id)
}

fn apply_matching_policy_if_any(
    client: &mut Client,
    ci_id: i32,
    anomaly_class: &str,
    event_id: i64,
) -> ClientResult<()> {
    let matched = client
        .query_opt(queries::SELECT_MATCHING_POLICY, &[&ci_id, &anomaly_class])?
        .map(|row| MatchedPolicy::from_row(&row));

    let Some(policy) = matched else {
        return Ok(());
    };

    let local_incident_id: i64 = client
        .query_one(queries::INSERT_LOCAL_INCIDENT, &[&event_id])?
        .get(0);
    client.execute(
        queries::INSERT_REMEDIATION_ACTION,
        &[&local_incident_id, &policy.policy_id, &policy.action_type],
    )?;
    println!(
        "[объект] реплика: критичность {}, действие «{}»",
        policy.criticality, policy.action_type
    );
    Ok(())
}

fn show_anomaly_queue(client: &mut Client) -> ClientResult<()> {
    let rows = client.query(queries::SELECT_ANOMALY_QUEUE, &[])?;
    let table_rows = rows
        .iter()
        .map(|row| AnomalyQueueEntry::from_row(row).into_table_row())
        .collect();
    print_table_or_message(
        &["event", "ci", "метрика", "значение", "класс", "статус", "время"],
        table_rows,
        "Очередь событий пуста.",
    );
    Ok(())
}

fn finish_remediation_action(client: &mut Client) -> ClientResult<()> {
    let rows = client.query(queries::SELECT_OPEN_REMEDIATION_ACTIONS, &[])?;
    if rows.is_empty() {
        println!("[объект] незавершённых действий нет");
        return Ok(());
    }
    let table_rows = rows
        .iter()
        .map(|row| OpenRemediationAction::from_row(row).into_table_row())
        .collect();
    print_table_or_message(&["action", "тип", "инцидент", "event"], table_rows, "Действий нет.");

    let action_id = read_id("action_id")?;
    let result = read_remediation_result()?;

    client.execute(queries::UPDATE_REMEDIATION_ACTION_RESULT, &[&result, &action_id])?;
    let incident_status = if result == "success" { "resolved" } else { "escalated" };
    client.execute(
        queries::UPDATE_LOCAL_INCIDENT_STATUS_BY_ACTION,
        &[&incident_status, &action_id],
    )?;
    println!("[объект] результат: {result}; инцидент → {incident_status}");
    Ok(())
}

fn show_remediation_history(client: &mut Client) -> ClientResult<()> {
    let rows = client.query(queries::SELECT_REMEDIATION_HISTORY, &[])?;
    let table_rows = rows
        .iter()
        .map(|row| RemediationHistoryEntry::from_row(row).into_table_row())
        .collect();
    print_table_or_message(
        &["action", "тип", "результат", "начало", "event", "инцидент"],
        table_rows,
        "История действий пуста.",
    );
    Ok(())
}
