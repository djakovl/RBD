use crate::ui::{op, pause, print_table, prompt};
use postgres::{Client, NoTls};

pub fn site_mode(url: &str) {
    let mut c=match Client::connect(url,NoTls){Ok(c)=>c,Err(e)=>{println!("[ОБЪЕКТ НЕДОСТУПЕН] {}",e);pause();return}};
    println!("[объект] подключено");
    loop {
        println!("\n--- ОБЪЕКТ КИИ: очередь аномалий, самовосстановление ---\n1) Статус канала до центра (FDW)\n2) Зарегистрировать аномалию\n3) Очередь событий\n4) Завершить действие самовосстановления\n5) Все действия\n0) Назад");
        match prompt(">").as_str(){"1"=>op(&mut c,channel),"2"=>op(&mut c,register),"3"=>op(&mut c,queue),"4"=>op(&mut c,finish),"5"=>op(&mut c,actions),"0"=>break,_=>{}}
    }
}
fn channel(c:&mut Client)->Result<(),postgres::Error>{match c.query("SELECT count(*) FROM center_ci",&[]){Ok(rs)=>println!("[объект] канал ЕСТЬ: FDW читает {} КЕ",rs[0].get::<_,i64>(0)),Err(_)=>println!("[объект] канала НЕТ: работа по локальной реплике")};Ok(())}
fn register(c:&mut Client)->Result<(),postgres::Error>{
    let ci:i32=match prompt("ci_id:").parse(){Ok(x)=>x,Err(_)=>return Ok(())};let metric=prompt("метрика:");let value:f64=match prompt("значение:").parse(){Ok(x)=>x,Err(_)=>return Ok(())};let class=prompt("класс:");
    let event:i64=c.query_one("INSERT INTO anomaly_event(ci_id,metric,value,anomaly_class) VALUES($1,$2,$3,$4) RETURNING event_id",&[&ci,&metric,&value,&class])?.get(0);println!("[объект] событие #{} записано локально (pending)",event);
    if let Some(r)=c.query_opt("SELECT p.action_type,p.policy_id,ci.criticality FROM remediation_policy p JOIN ci ON ci.ci_type=p.ci_type WHERE ci.ci_id=$1 AND p.anomaly_class=$2",&[&ci,&class])?{let action:String=r.get(0);let pid:i32=r.get(1);let crit:String=r.get(2);let lid:i64=c.query_one("INSERT INTO local_incident(event_id) VALUES($1) RETURNING local_incident_id",&[&event])?.get(0);c.execute("INSERT INTO remediation_action(local_incident_id,policy_id,action_type) VALUES($1,$2,$3)",&[&lid,&pid,&action])?;println!("[объект] реплика: критичность {}, действие «{}»",crit,action)}
    Ok(())
}
fn queue(c:&mut Client)->Result<(),postgres::Error>{let rs=c.query("SELECT event_id,ci_id,metric,value::text,anomaly_class,sync_status,to_char(detected_at,'YYYY-MM-DD HH24:MI:SS TZ') FROM anomaly_event ORDER BY event_id DESC LIMIT 20",&[])?;print_table(&["event","ci","метрика","значение","класс","статус","время"],rs.into_iter().map(|r|vec![r.get::<_,i64>(0).to_string(),r.get::<_,i32>(1).to_string(),r.get(2),r.get(3),r.get(4),r.get(5),r.get(6)]).collect());Ok(())}
fn finish(c:&mut Client)->Result<(),postgres::Error>{let rs=c.query("SELECT a.action_id,a.action_type,a.local_incident_id,l.event_id FROM remediation_action a JOIN local_incident l ON l.local_incident_id=a.local_incident_id WHERE a.result IS NULL",&[])?;if rs.is_empty(){println!("[объект] незавершённых действий нет");return Ok(())};print_table(&["action","тип","инцидент","event"],rs.iter().map(|r|vec![r.get::<_,i64>(0).to_string(),r.get(1),r.get::<_,i64>(2).to_string(),r.get::<_,i64>(3).to_string()]).collect());let id:i64=match prompt("action_id:").parse(){Ok(x)=>x,Err(_)=>return Ok(())};let result=prompt("результат (success/failed):");if result!="success"&&result!="failed"{return Ok(())};c.execute("UPDATE remediation_action SET result=$1,finished_at=now() WHERE action_id=$2",&[&result,&id])?;let status=if result=="success"{"resolved"}else{"escalated"};c.execute("UPDATE local_incident SET status=$1,updated_at=now() WHERE local_incident_id=(SELECT local_incident_id FROM remediation_action WHERE action_id=$2)",&[&status,&id])?;println!("[объект] результат: {}; инцидент → {}",result,status);Ok(())}
fn actions(c:&mut Client)->Result<(),postgres::Error>{let rs=c.query("SELECT a.action_id,a.action_type,coalesce(a.result,'(выполняется)'),to_char(a.started_at,'YYYY-MM-DD HH24:MI:SS TZ'),l.event_id,l.status FROM remediation_action a JOIN local_incident l ON l.local_incident_id=a.local_incident_id ORDER BY a.action_id DESC LIMIT 20",&[])?;print_table(&["action","тип","результат","начало","event","инцидент"],rs.into_iter().map(|r|vec![r.get::<_,i64>(0).to_string(),r.get(1),r.get(2),r.get(3),r.get::<_,i64>(4).to_string(),r.get(5)]).collect());Ok(())}
