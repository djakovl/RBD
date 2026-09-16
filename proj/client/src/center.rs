//! Режим ЦЕНТРА: CRUD справочников, журнал инцидентов, квитирование
//! событий, полученных с объекта через FDW.

use crate::db::{self, ClientError, ClientResult};
use crate::models::{ConfigurationItem, IncidentJournalEntry, PendingAnomalyEvent, RemediationPolicy};
use crate::queries;
use crate::selectors::{read_criticality, read_id_i32, read_non_empty, read_priority_or_default};
use crate::table::print_table_or_message;
use crate::ui::{pause, prompt, run_operation};
use postgres::Client;

/// Запускает интерактивное меню режима центра на переданном подключении.
pub fn run(url: &str) {
    let mut client = match db::connect(url) {
        Ok(client) => client,
        Err(error) => {
            println!("[ЦЕНТР НЕДОСТУПЕН] {error}");
            pause();
            return;
        }
    };
    println!("[центр] подключено");

    loop {
        print_menu();
        match prompt("> ").as_str() {
            "1" => run_operation(&mut client, list_configuration_items),
            "2" => run_operation(&mut client, add_configuration_item),
            "3" => run_operation(&mut client, update_configuration_item_criticality),
            "4" => run_operation(&mut client, delete_configuration_item),
            "5" => run_operation(&mut client, list_remediation_policies),
            "6" => run_operation(&mut client, add_remediation_policy),
            "7" => run_operation(&mut client, show_incident_journal),
            "8" => run_operation(&mut client, acknowledge_pending_event),
            "0" => break,
            _ => println!("Неизвестная команда, попробуйте снова."),
        }
    }
}

fn print_menu() {
    println!("\n--- ЦЕНТР: справочники, журнал, квитирование ---");
    println!("1) Список оборудования (ci)");
    println!("2) Добавить КЕ");
    println!("3) Изменить критичность КЕ (UPDATE)");
    println!("4) Удалить КЕ (DELETE)");
    println!("5) Список политик самовосстановления");
    println!("6) Добавить политику");
    println!("7) Журнал v_incident_all (журнал + очередь объекта)");
    println!("8) Квитировать события из очереди объекта (FDW)");
    println!("0) Назад");
}

fn list_configuration_items(client: &mut Client) -> ClientResult<()> {
    println!("[центр] чтение локального справочника");
    let rows = client.query(queries::SELECT_CI_LIST, &[])?;
    let table_rows = rows
        .iter()
        .map(|row| ConfigurationItem::from_row(row).into_table_row())
        .collect();
    print_table_or_message(&["id", "тип", "критичность", "parent"], table_rows, "Оборудование не найдено.");
    Ok(())
}

fn add_configuration_item(client: &mut Client) -> ClientResult<()> {
    let ci_type = read_non_empty("тип");
    let criticality = read_criticality()?;
    client.execute(queries::INSERT_CI, &[&ci_type, &criticality])?;
    println!("[центр] КЕ создана; на объект уедет логической репликацией");
    Ok(())
}

fn update_configuration_item_criticality(client: &mut Client) -> ClientResult<()> {
    let ci_id = read_id_i32("ci_id")?;
    let criticality = read_criticality()?;
    let updated = client.execute(queries::UPDATE_CI_CRITICALITY, &[&ci_id, &criticality])?;
    println!("[центр] обновлено строк: {updated}");
    Ok(())
}

fn delete_configuration_item(client: &mut Client) -> ClientResult<()> {
    let ci_id = read_id_i32("ci_id")?;
    let deleted = client.execute(queries::DELETE_CI, &[&ci_id])?;
    println!("[центр] удалено строк: {deleted}");
    Ok(())
}

fn list_remediation_policies(client: &mut Client) -> ClientResult<()> {
    println!("[центр] чтение локального справочника политик");
    let rows = client.query(queries::SELECT_POLICY_LIST, &[])?;
    let table_rows = rows
        .iter()
        .map(|row| RemediationPolicy::from_row(row).into_table_row())
        .collect();
    print_table_or_message(
        &["policy", "тип КЕ", "класс аномалии", "действие", "приоритет"],
        table_rows,
        "Политики не найдены.",
    );
    Ok(())
}

fn add_remediation_policy(client: &mut Client) -> ClientResult<()> {
    let ci_type = read_non_empty("ci_type");
    let anomaly_class = read_non_empty("класс");
    let action_type = read_non_empty("действие");
    let priority = read_priority_or_default(1);
    client.execute(
        queries::INSERT_POLICY,
        &[&ci_type, &anomaly_class, &action_type, &priority],
    )?;
    println!("[центр] политика создана");
    Ok(())
}

fn show_incident_journal(client: &mut Client) -> ClientResult<()> {
    println!("[центр] v_incident_all: журнал центра + очередь объекта через FDW");
    let rows = client.query(queries::SELECT_INCIDENT_JOURNAL, &[])?;
    let table_rows = rows
        .iter()
        .map(|row| IncidentJournalEntry::from_row(row).into_table_row())
        .collect();
    print_table_or_message(
        &["источник", "event", "ci", "время", "класс", "статус"],
        table_rows,
        "Журнал пуст.",
    );
    Ok(())
}

fn acknowledge_pending_event(client: &mut Client) -> ClientResult<()> {
    let rows = client.query(queries::SELECT_PENDING_SITE_EVENTS, &[])?;
    if rows.is_empty() {
        println!("[центр] pending-событий нет");
        return Ok(());
    }
    let table_rows = rows
        .iter()
        .map(|row| PendingAnomalyEvent::from_row(row).into_table_row())
        .collect();
    print_table_or_message(&["event", "ci", "класс", "время"], table_rows, "Событий нет.");

    let raw_id = prompt("event_id (Enter — отмена):");
    if raw_id.is_empty() {
        return Ok(());
    }
    let event_id: i64 = raw_id
        .parse()
        .map_err(|_| ClientError::InvalidInput(format!("«{raw_id}» не является числом")))?;

    let inserted = client.execute(queries::INSERT_INCIDENT_FROM_PENDING_EVENT, &[&event_id])?;
    if inserted == 0 {
        println!("[центр] событие не найдено");
        return Ok(());
    }
    client.execute(queries::MARK_SITE_EVENT_ACKED, &[&event_id])?;
    client.execute(queries::MARK_SITE_LOCAL_INCIDENT_ACKED, &[&event_id])?;
    println!("[центр] создан incident; на объект отправлен статус pending → acked через FDW");
    Ok(())
}
