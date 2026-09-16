// Режим ЦЕНТРА: узел-владелец справочников (ci, remediation_policy),
// журнала incident и представления v_incident_all. Квитирование —
// распределённая операция: чтение очереди объекта и запись статуса через FDW.

use crate::ui::{op, pause, prompt};
use postgres::{Client, NoTls};

pub fn center_mode(url: &str) {
    let mut c = match Client::connect(url, NoTls) {
        Ok(c) => c,
        Err(e) => {
            println!("[ЦЕНТР НЕДОСТУПЕН] {}", e);
            println!("(модель отказа центра: объект продолжает работать сам по себе)");
            pause();
            return;
        }
    };
    println!("[центр] подключено");
    loop {
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
        match prompt(">").as_str() {
            "1" => op(&mut c, center_list_ci),
            "2" => op(&mut c, center_add_ci),
            "3" => op(&mut c, center_upd_ci),
            "4" => op(&mut c, center_del_ci),
            "5" => op(&mut c, center_list_pol),
            "6" => op(&mut c, center_add_pol),
            "7" => op(&mut c, center_journal),
            "8" => op(&mut c, center_ack),
            "0" => break,
            _ => {}
        }
    }
}

fn center_list_ci(c: &mut Client) -> Result<(), postgres::Error> {
    println!("[центр] чтение ЛОКАЛЬНОГО справочника (центр — узел-владелец)");
    let rows = c.query(
        "SELECT ci_id, ci_type, criticality, coalesce(parent_ci_id::text, '-') FROM ci ORDER BY ci_id",
        &[],
    )?;
    println!("{:<4} {:<18} {:<10} {:<7}", "id", "тип", "критичность", "parent");
    for r in rows {
        println!(
            "{:<4} {:<18} {:<10} {:<7}",
            r.get::<_, i32>(0),
            r.get::<_, String>(1),
            r.get::<_, String>(2),
            r.get::<_, String>(3)
        );
    }
    Ok(())
}

fn center_add_ci(c: &mut Client) -> Result<(), postgres::Error> {
    let ci_type = prompt("тип (controller/telemetry_server/network_gateway/sensor/test_bench):");
    let crit = prompt("критичность (high/medium/low):");
    let parent = prompt("parent ci_id (Enter — корневая):");
    if parent.is_empty() {
        c.execute(
            "INSERT INTO ci (site_id, ci_type, criticality) VALUES (1, $1, $2)",
            &[&ci_type, &crit],
        )?;
    } else {
        let p: i32 = match parent.parse() {
            Ok(v) => v,
            Err(_) => {
                println!("parent должен быть целым числом");
                return Ok(());
            }
        };
        c.execute(
            "INSERT INTO ci (site_id, ci_type, criticality, parent_ci_id) VALUES (1, $1, $2, $3)",
            &[&ci_type, &crit, &p],
        )?;
    }
    println!("[центр] КЕ создана ЛОКАЛЬНО; на объект уедет логической репликацией");
    Ok(())
}

fn center_upd_ci(c: &mut Client) -> Result<(), postgres::Error> {
    let id: i32 = match prompt("ci_id:").parse() {
        Ok(v) => v,
        Err(_) => {
            println!("целое число");
            return Ok(());
        }
    };
    let crit = prompt("новая критичность (high/medium/low):");
    let n = c.execute("UPDATE ci SET criticality = $2 WHERE ci_id = $1", &[&id, &crit])?;
    if n == 0 {
        println!("КЕ не найдена");
    } else {
        println!("[центр] обновлено; объект получит изменение через логическую репликацию (проверь в режиме объекта)");
    }
    Ok(())
}

fn center_del_ci(c: &mut Client) -> Result<(), postgres::Error> {
    let id: i32 = match prompt("ci_id:").parse() {
        Ok(v) => v,
        Err(_) => {
            println!("целое число");
            return Ok(());
        }
    };
    let n = c.execute("DELETE FROM ci WHERE ci_id = $1", &[&id])?;
    if n == 0 {
        println!("КЕ не найдена");
    } else {
        println!("[центр] удалено (если есть связанные инциденты — СУБД не даст, увидишь ошибку внешнего ключа)");
    }
    Ok(())
}

fn center_list_pol(c: &mut Client) -> Result<(), postgres::Error> {
    println!("[центр] чтение ЛОКАЛЬНОГО справочника политик");
    let rows = c.query(
        "SELECT policy_id, ci_type, anomaly_class, action_type, priority FROM remediation_policy ORDER BY policy_id",
        &[],
    )?;
    println!("{:<8} {:<18} {:<22} {:<18} {:<6}", "policy", "тип КЕ", "класс аномалии", "действие", "приор.");
    for r in rows {
        println!(
            "{:<8} {:<18} {:<22} {:<18} {:<6}",
            r.get::<_, i32>(0),
            r.get::<_, String>(1),
            r.get::<_, String>(2),
            r.get::<_, String>(3),
            r.get::<_, i32>(4)
        );
    }
    Ok(())
}

fn center_add_pol(c: &mut Client) -> Result<(), postgres::Error> {
    let ci_type = prompt("ci_type:");
    let class = prompt("класс аномалии:");
    let action = prompt("действие (restart/isolate/block/switch_to_backup/ignore):");
    let prio: i32 = match prompt("приоритет (1-3):").parse() {
        Ok(v) => v,
        Err(_) => 1,
    };
    c.execute(
        "INSERT INTO remediation_policy (ci_type, anomaly_class, action_type, priority) VALUES ($1, $2, $3, $4)",
        &[&ci_type, &class, &action, &prio],
    )?;
    println!("[центр] политика создана ЛОКАЛЬНО; приедет на объект логической репликацией");
    Ok(())
}

fn center_journal(c: &mut Client) -> Result<(), postgres::Error> {
    println!("[центр] единое представление v_incident_all: журнал (локально) + очередь объекта (через FDW)");
    let rows = c.query(
        "SELECT src, event_id::text, ci_id::text, ts::text, class, status FROM v_incident_all ORDER BY ts DESC LIMIT 30",
        &[],
    )?;
    println!("{:<8} {:>8} {:>5} {:<17} {:<22} {}", "src", "event", "ci", "время", "класс", "статус");
    for r in rows {
        println!(
            "{:<8} {:>8} {:>5} {:<17} {:<22} {}",
            r.get::<_, String>(0),
            r.get::<_, String>(1),
            r.get::<_, String>(2),
            r.get::<_, String>(3),
            r.get::<_, String>(4),
            r.get::<_, String>(5)
        );
    }
    println!("src=journal — записи журнала (центр), src=queue — события очереди (узел-объект, через FDW)");
    Ok(())
}

fn center_ack(c: &mut Client) -> Result<(), postgres::Error> {
    println!("[центр] читаю очередь ОБЪЕКТА через FDW (внешняя таблица site_anomaly_event)...");
    let rows = c.query(
        "SELECT event_id, ci_id, anomaly_class, to_char(detected_at, 'MM-DD HH24:MI:SS')
           FROM site_anomaly_event
          WHERE sync_status = 'pending'
          ORDER BY event_id LIMIT 25",
        &[],
    )?;
    if rows.is_empty() {
        println!("[центр] pending-событий нет (очередь пуста либо канал до объекта недоступен)");
        return Ok(());
    }
    println!("{:<8} {:<4} {:<22} {:<14}", "event", "ci", "класс", "время");
    for r in rows {
        println!(
            "{:<8} {:<4} {:<22} {:<14}",
            r.get::<_, i64>(0),
            r.get::<_, i32>(1),
            r.get::<_, String>(2),
            r.get::<_, String>(3)
        );
    }
    let id: i64 = match prompt("event_id для квитирования (Enter — отмена):").parse() {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };
    let n = c.execute(
        "INSERT INTO incident (event_id, ci_id, severity, status)
         SELECT e.event_id, e.ci_id,
                CASE c.criticality WHEN 'high' THEN 'critical' WHEN 'medium' THEN 'major' ELSE 'minor' END,
                'acked'
           FROM site_anomaly_event e
           JOIN ci c ON c.ci_id = e.ci_id
          WHERE e.event_id = $1 AND e.sync_status = 'pending'",
        &[&id],
    )?;
    if n == 0 {
        println!("[центр] событие не найдено среди pending (или его КЕ удалена)");
        return Ok(());
    }
    println!("[центр] инцидент создан в ЖУРНАЛЕ: распределённый запрос — JOIN локального справочника (ci) и внешней очереди (site_anomaly_event)");
    c.execute(
        "UPDATE site_anomaly_event SET sync_status = 'acked' WHERE event_id = $1",
        &[&id],
    )?;
    c.execute(
        "UPDATE site_local_incident SET status = 'acked', updated_at = now() WHERE event_id = $1",
        &[&id],
    )?;
    println!("[центр] статус события на объекте: pending → acked (ЗАПИСЬ ЧЕРЕЗ FDW)");
    Ok(())
}
