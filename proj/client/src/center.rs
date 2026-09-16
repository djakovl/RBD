use crate::ui::{op, pause, print_table, prompt};
use postgres::{Client, NoTls};

pub fn center_mode(url: &str) {
    let mut c = match Client::connect(url, NoTls) { Ok(c) => c, Err(e) => { println!("[ЦЕНТР НЕДОСТУПЕН] {}", e); pause(); return; } };
    println!("[центр] подключено");
    loop {
        println!("\n--- ЦЕНТР: справочники, журнал, квитирование ---\n1) Список оборудования (ci)\n2) Добавить КЕ\n3) Изменить критичность КЕ (UPDATE)\n4) Удалить КЕ (DELETE)\n5) Список политик самовосстановления\n6) Добавить политику\n7) Журнал v_incident_all (журнал + очередь объекта)\n8) Квитировать события из очереди объекта (FDW)\n0) Назад");
        match prompt(">").as_str() {
            "1" => op(&mut c, list_ci), "2" => op(&mut c, add_ci), "3" => op(&mut c, update_ci),
            "4" => op(&mut c, delete_ci), "5" => op(&mut c, list_policy), "6" => op(&mut c, add_policy),
            "7" => op(&mut c, journal), "8" => op(&mut c, acknowledge), "0" => break, _ => {}
        }
    }
}

fn list_ci(c: &mut Client) -> Result<(), postgres::Error> {
    println!("[центр] чтение локального справочника");
    let rs = c.query("SELECT ci_id,ci_type,criticality,coalesce(parent_ci_id::text,'-') FROM ci ORDER BY ci_id", &[])?;
    print_table(&["id","тип","критичность","parent"], rs.into_iter().map(|r| vec![r.get::<_,i32>(0).to_string(),r.get(1),r.get(2),r.get(3)]).collect()); Ok(())
}
fn add_ci(c: &mut Client) -> Result<(), postgres::Error> {
    let typ=prompt("тип:"); let crit=prompt("критичность (high/medium/low):");
    c.execute("INSERT INTO ci(site_id,ci_type,criticality) VALUES(1,$1,$2)",&[&typ,&crit])?;
    println!("[центр] КЕ создана; на объект уедет логической репликацией"); Ok(())
}
fn update_ci(c: &mut Client) -> Result<(), postgres::Error> {
    let id: i32=match prompt("ci_id:").parse(){Ok(x)=>x,Err(_)=>return Ok(())}; let crit=prompt("критичность:");
    println!("[центр] обновлено строк: {}",c.execute("UPDATE ci SET criticality=$2 WHERE ci_id=$1",&[&id,&crit])?); Ok(())
}
fn delete_ci(c: &mut Client) -> Result<(), postgres::Error> {
    let id: i32=match prompt("ci_id:").parse(){Ok(x)=>x,Err(_)=>return Ok(())};
    println!("[центр] удалено строк: {}",c.execute("DELETE FROM ci WHERE ci_id=$1",&[&id])?); Ok(())
}
fn list_policy(c: &mut Client) -> Result<(), postgres::Error> {
    println!("[центр] чтение локального справочника политик");
    let rs=c.query("SELECT policy_id,ci_type,anomaly_class,action_type,priority FROM remediation_policy ORDER BY policy_id",&[])?;
    print_table(&["policy","тип КЕ","класс аномалии","действие","приоритет"],rs.into_iter().map(|r|vec![r.get::<_,i32>(0).to_string(),r.get(1),r.get(2),r.get(3),r.get::<_,i32>(4).to_string()]).collect()); Ok(())
}
fn add_policy(c: &mut Client) -> Result<(), postgres::Error> {
    let typ=prompt("ci_type:");let class=prompt("класс:");let action=prompt("действие:");let prio:i32=prompt("приоритет:").parse().unwrap_or(1);
    c.execute("INSERT INTO remediation_policy(ci_type,anomaly_class,action_type,priority) VALUES($1,$2,$3,$4)",&[&typ,&class,&action,&prio])?; println!("[центр] политика создана"); Ok(())
}
fn journal(c: &mut Client) -> Result<(), postgres::Error> {
    println!("[центр] v_incident_all: журнал центра + очередь объекта через FDW");
    let rs=c.query("SELECT src,event_id::text,ci_id::text,to_char(ts,'YYYY-MM-DD HH24:MI:SS TZ'),class,status FROM v_incident_all ORDER BY ts DESC LIMIT 30",&[])?;
    print_table(&["источник","event","ci","время","класс","статус"],rs.into_iter().map(|r|vec![r.get(0),r.get(1),r.get(2),r.get(3),r.get(4),r.get(5)]).collect()); Ok(())
}
fn acknowledge(c: &mut Client) -> Result<(), postgres::Error> {
    let rs=c.query("SELECT event_id,ci_id,anomaly_class,to_char(detected_at,'YYYY-MM-DD HH24:MI:SS TZ') FROM site_anomaly_event WHERE sync_status='pending' ORDER BY event_id LIMIT 25",&[])?;
    if rs.is_empty(){println!("[центр] pending-событий нет");return Ok(())}
    print_table(&["event","ci","класс","время"],rs.iter().map(|r|vec![r.get::<_,i64>(0).to_string(),r.get::<_,i32>(1).to_string(),r.get(2),r.get(3)]).collect());
    let id:i64=match prompt("event_id (Enter — отмена):").parse(){Ok(x)=>x,Err(_)=>return Ok(())};
    let n=c.execute("INSERT INTO incident(event_id,ci_id,severity,status) SELECT e.event_id,e.ci_id,CASE c.criticality WHEN 'high' THEN 'critical' WHEN 'medium' THEN 'major' ELSE 'minor' END,'acked' FROM site_anomaly_event e JOIN ci c ON c.ci_id=e.ci_id WHERE e.event_id=$1 AND e.sync_status='pending'",&[&id])?;
    if n==0 {println!("[центр] событие не найдено");return Ok(())}
    c.execute("UPDATE site_anomaly_event SET sync_status='acked' WHERE event_id=$1",&[&id])?;
    c.execute("UPDATE site_local_incident SET status='acked',updated_at=now() WHERE event_id=$1",&[&id])?;
    println!("[центр] создан incident; на объект отправлен статус pending → acked через FDW"); Ok(())
}
