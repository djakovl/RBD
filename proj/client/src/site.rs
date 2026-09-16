// Режим ОБЪЕКТА КИИ: писатель оперативных данных (anomaly_event, local_incident,
// remediation_action). Работает автономно: канал до центра нужен только для
// онлайн-проверок через FDW; выбор действия самовосстановления идёт по
// локальной реплике справочников.

use crate::ui::{op, pause, prompt};
use postgres::{Client, NoTls};

pub fn site_mode(url: &str) {
    let mut c = match Client::connect(url, NoTls) {
        Ok(c) => c,
        Err(e) => {
            println!("[ОБЪЕКТ НЕДОСТУПЕН] {}", e);
            println!("(модель отказа объекта: теряется несинхронизированная очередь)");
            pause();
            return;
        }
    };
    println!("[объект] подключено");
    loop {
        println!("\n--- ОБЪЕКТ КИИ: очередь аномалий, самовосстановление ---");
        println!("1) Статус канала до центра (проверка FDW)");
        println!("2) Зарегистрировать аномалию (локально + выбор действия по реплике)");
        println!("3) Очередь событий");
        println!("4) Незавершённые действия (фиксация результата)");
        println!("5) Все действия самовосстановления");
        println!("0) Назад");
        match prompt(">").as_str() {
            "1" => op(&mut c, site_channel_status),
            "2" => op(&mut c, site_register),
            "3" => op(&mut c, site_queue),
            "4" => op(&mut c, site_finish),
            "5" => op(&mut c, site_actions),
            "0" => break,
            _ => {}
        }
    }
}

fn site_channel_status(c: &mut Client) -> Result<(), postgres::Error> {
    println!("[объект] проверка канала до центра: чтение center_ci через FDW...");
    match c.query("SELECT count(*) FROM center_ci", &[]) {
        Ok(rows) => {
            let n: i64 = rows[0].get(0);
            println!("[объект] канал до центра ЕСТЬ (онлайн): справочник читается через FDW, {} КЕ", n);
        }
        Err(_) => {
            println!("[объект] канала до центра НЕТ (offline): FDW недоступен, справочник берём из локальной РЕПЛИКИ");
        }
    }
    Ok(())
}

fn site_register(c: &mut Client) -> Result<(), postgres::Error> {
    println!("[объект] регистрация аномалии — канал до центра НЕ нужен, запись локальная");
    let ci_id: i32 = match prompt("ci_id КЕ:").parse() {
        Ok(v) => v,
        Err(_) => {
            println!("целое число");
            return Ok(());
        }
    };
    let metric = prompt("метрика (cpu_temp/latency_ms/...):");
    let value: f64 = match prompt("значение:").parse() {
        Ok(v) => v,
        Err(_) => {
            println!("число");
            return Ok(());
        }
    };
    let class = prompt("класс (metric_threshold/component_unreachable/suspicious_activity):");

    let row = c.query_one(
        "INSERT INTO anomaly_event (ci_id, metric, value, anomaly_class) VALUES ($1, $2, $3, $4) RETURNING event_id",
        &[&ci_id, &metric, &value, &class],
    )?;
    let event_id: i64 = row.get(0);
    println!("[объект] событие #{} записано ЛОКАЛЬНО в очередь (sync_status = pending)", event_id);

    let pol = c.query_opt(
        "SELECT p.action_type, p.policy_id, ci.criticality
           FROM remediation_policy p
           JOIN ci ON ci.ci_type = p.ci_type
          WHERE ci.ci_id = $1 AND p.anomaly_class = $2",
        &[&ci_id, &class],
    )?;
    match pol {
        Some(r) => {
            let action: String = r.get(0);
            let policy_id: i32 = r.get(1);
            let crit: String = r.get(2);
            let inc = c.query_one(
                "INSERT INTO local_incident (event_id) VALUES ($1) RETURNING local_incident_id",
                &[&event_id],
            )?;
            let liid: i64 = inc.get(0);
            c.execute(
                "INSERT INTO remediation_action (local_incident_id, policy_id, action_type) VALUES ($1, $2, $3)",
                &[&liid, &policy_id, &action],
            )?;
            if action == "ignore" {
                c.execute(
                    "UPDATE remediation_action SET result = 'success', finished_at = now() WHERE local_incident_id = $1",
                    &[&liid],
                )?;
                c.execute(
                    "UPDATE local_incident SET status = 'resolved', updated_at = now() WHERE local_incident_id = $1",
                    &[&liid],
                )?;
                println!("[объект] политика «ignore»: действие завершено сразу (success), инцидент resolved");
            } else {
                println!(
                    "[объект] по ЛОКАЛЬНОЙ РЕПЛИКЕ справочника (критичность {}) назначено действие «{}» (политика #{})",
                    crit, action, policy_id
                );
            }
        }
        None => println!("[объект] подходящей политики не найдено — событие ждёт в очереди"),
    }
    Ok(())
}

fn site_queue(c: &mut Client) -> Result<(), postgres::Error> {
    let row = c.query_one(
        "SELECT count(*) FILTER (WHERE sync_status = 'pending'), count(*) FROM anomaly_event",
        &[],
    )?;
    let pending: i64 = row.get(0);
    let total: i64 = row.get(1);
    println!(
        "[объект] очередь: {} ожидают квитирования центром, всего событий {}",
        pending, total
    );
    let rows = c.query(
        "SELECT event_id, ci_id, metric, value::text, anomaly_class, sync_status,
                to_char(detected_at, 'MM-DD HH24:MI:SS')
           FROM anomaly_event ORDER BY event_id DESC LIMIT 20",
        &[],
    )?;
    println!("{:<8} {:<4} {:<12} {:<10} {:<22} {:<9} {:<14}", "event", "ci", "метрика", "знач.", "класс", "статус", "время");
    for r in rows {
        println!(
            "{:<8} {:<4} {:<12} {:<10} {:<22} {:<9} {:<14}",
            r.get::<_, i64>(0),
            r.get::<_, i32>(1),
            r.get::<_, String>(2),
            r.get::<_, String>(3),
            r.get::<_, String>(4),
            r.get::<_, String>(5),
            r.get::<_, String>(6)
        );
    }
    Ok(())
}

fn site_finish(c: &mut Client) -> Result<(), postgres::Error> {
    let rows = c.query(
        "SELECT a.action_id, a.action_type, a.local_incident_id, l.event_id
           FROM remediation_action a
           JOIN local_incident l ON l.local_incident_id = a.local_incident_id
          WHERE a.result IS NULL
          ORDER BY a.action_id",
        &[],
    )?;
    if rows.is_empty() {
        println!("[объект] незавершённых действий нет");
        return Ok(());
    }
    println!("{:<8} {:<18} {:<10} {:<8}", "action", "тип", "инцидент", "event");
    for r in rows {
        println!(
            "{:<8} {:<18} {:<10} {:<8}",
            r.get::<_, i64>(0),
            r.get::<_, String>(1),
            r.get::<_, i64>(2),
            r.get::<_, i64>(3)
        );
    }
    let aid: i64 = match prompt("action_id:").parse() {
        Ok(v) => v,
        Err(_) => {
            println!("целое число");
            return Ok(());
        }
    };
    let res = prompt("результат (success/failed):");
    if res != "success" && res != "failed" {
        println!("результат должен быть success или failed");
        return Ok(());
    }
    let n = c.execute(
        "UPDATE remediation_action SET result = $1, finished_at = now() WHERE action_id = $2 AND result IS NULL",
        &[&res, &aid],
    )?;
    if n == 0 {
        println!("действие не найдено или уже закрыто");
        return Ok(());
    }
    let new_status = if res == "success" {
        "resolved".to_string()
    } else {
        "escalated".to_string()
    };
    c.execute(
        "UPDATE local_incident SET status = $1, updated_at = now()
          WHERE local_incident_id = (SELECT local_incident_id FROM remediation_action WHERE action_id = $2)",
        &[&new_status, &aid],
    )?;
    if res == "success" {
        println!("[объект] самовосстановление УСПЕШНО: инцидент → resolved");
    } else {
        println!("[объект] самовосстановление НЕ УДАЛОСЬ: инцидент → escalated, требуется оператор центра (центру видно через FDW)");
    }
    Ok(())
}

fn site_actions(c: &mut Client) -> Result<(), postgres::Error> {
    let rows = c.query(
        "SELECT a.action_id, a.action_type, coalesce(a.result, '(выполняется)'),
                to_char(a.started_at, 'MM-DD HH24:MI:SS'), l.event_id, l.status
           FROM remediation_action a
           JOIN local_incident l ON l.local_incident_id = a.local_incident_id
          ORDER BY a.action_id DESC LIMIT 20",
        &[],
    )?;
    if rows.is_empty() {
        println!("[объект] действий пока нет (зарегистрируй аномалию)");
        return Ok(());
    }
    println!("{:<8} {:<18} {:<16} {:<14} {:<8} {:<10}", "action", "тип", "результат", "начало", "event", "инцидент");
    for r in rows {
        println!(
            "{:<8} {:<18} {:<16} {:<14} {:<8} {:<10}",
            r.get::<_, i64>(0),
            r.get::<_, String>(1),
            r.get::<_, String>(2),
            r.get::<_, String>(3),
            r.get::<_, i64>(4),
            r.get::<_, String>(5)
        );
    }
    Ok(())
}
