//! Структуры данных, соответствующие строкам результатов SELECT.
//!
//! Каждая структура сопоставлена с одним запросом из `queries.rs`
//! и знает, как собрать себя из строки `postgres::Row`.

use postgres::Row;

/// Конфигурационная единица оборудования (объект учёта в справочнике `ci`).
pub struct ConfigurationItem {
    pub ci_id: i32,
    pub ci_type: String,
    pub criticality: String,
    pub parent_ci_id: String,
}

impl ConfigurationItem {
    pub fn from_row(row: &Row) -> Self {
        ConfigurationItem {
            ci_id: row.get(0),
            ci_type: row.get(1),
            criticality: row.get(2),
            parent_ci_id: row.get(3),
        }
    }

    pub fn into_table_row(self) -> Vec<String> {
        vec![
            self.ci_id.to_string(),
            self.ci_type,
            self.criticality,
            self.parent_ci_id,
        ]
    }
}

/// Политика самовосстановления для класса аномалий на типе оборудования.
pub struct RemediationPolicy {
    pub policy_id: i32,
    pub ci_type: String,
    pub anomaly_class: String,
    pub action_type: String,
    pub priority: i32,
}

impl RemediationPolicy {
    pub fn from_row(row: &Row) -> Self {
        RemediationPolicy {
            policy_id: row.get(0),
            ci_type: row.get(1),
            anomaly_class: row.get(2),
            action_type: row.get(3),
            priority: row.get(4),
        }
    }

    pub fn into_table_row(self) -> Vec<String> {
        vec![
            self.policy_id.to_string(),
            self.ci_type,
            self.anomaly_class,
            self.action_type,
            self.priority.to_string(),
        ]
    }
}

/// Строка объединённого журнала инцидентов (центр + очередь объекта через FDW).
pub struct IncidentJournalEntry {
    pub source: String,
    pub event_id: String,
    pub ci_id: String,
    pub timestamp: String,
    pub anomaly_class: String,
    pub status: String,
}

impl IncidentJournalEntry {
    pub fn from_row(row: &Row) -> Self {
        IncidentJournalEntry {
            source: row.get(0),
            event_id: row.get(1),
            ci_id: row.get(2),
            timestamp: row.get(3),
            anomaly_class: row.get(4),
            status: row.get(5),
        }
    }

    pub fn into_table_row(self) -> Vec<String> {
        vec![
            self.source,
            self.event_id,
            self.ci_id,
            self.timestamp,
            self.anomaly_class,
            self.status,
        ]
    }
}

/// Событие аномалии объекта, ожидающее квитирования центром.
pub struct PendingAnomalyEvent {
    pub event_id: i64,
    pub ci_id: i32,
    pub anomaly_class: String,
    pub detected_at: String,
}

impl PendingAnomalyEvent {
    pub fn from_row(row: &Row) -> Self {
        PendingAnomalyEvent {
            event_id: row.get(0),
            ci_id: row.get(1),
            anomaly_class: row.get(2),
            detected_at: row.get(3),
        }
    }

    pub fn into_table_row(self) -> Vec<String> {
        vec![
            self.event_id.to_string(),
            self.ci_id.to_string(),
            self.anomaly_class,
            self.detected_at,
        ]
    }
}

/// Строка очереди аномалий объекта КИИ (локальная таблица `anomaly_event`).
pub struct AnomalyQueueEntry {
    pub event_id: i64,
    pub ci_id: i32,
    pub metric: String,
    pub value: String,
    pub anomaly_class: String,
    pub sync_status: String,
    pub detected_at: String,
}

impl AnomalyQueueEntry {
    pub fn from_row(row: &Row) -> Self {
        AnomalyQueueEntry {
            event_id: row.get(0),
            ci_id: row.get(1),
            metric: row.get(2),
            value: row.get(3),
            anomaly_class: row.get(4),
            sync_status: row.get(5),
            detected_at: row.get(6),
        }
    }

    pub fn into_table_row(self) -> Vec<String> {
        vec![
            self.event_id.to_string(),
            self.ci_id.to_string(),
            self.metric,
            self.value,
            self.anomaly_class,
            self.sync_status,
            self.detected_at,
        ]
    }
}

/// Незавершённое действие самовосстановления, ожидающее результата.
pub struct OpenRemediationAction {
    pub action_id: i64,
    pub action_type: String,
    pub local_incident_id: i64,
    pub event_id: i64,
}

impl OpenRemediationAction {
    pub fn from_row(row: &Row) -> Self {
        OpenRemediationAction {
            action_id: row.get(0),
            action_type: row.get(1),
            local_incident_id: row.get(2),
            event_id: row.get(3),
        }
    }

    pub fn into_table_row(self) -> Vec<String> {
        vec![
            self.action_id.to_string(),
            self.action_type,
            self.local_incident_id.to_string(),
            self.event_id.to_string(),
        ]
    }
}

/// Полная история действий самовосстановления (завершённых и текущих).
pub struct RemediationHistoryEntry {
    pub action_id: i64,
    pub action_type: String,
    pub result: String,
    pub started_at: String,
    pub event_id: i64,
    pub incident_status: String,
}

impl RemediationHistoryEntry {
    pub fn from_row(row: &Row) -> Self {
        RemediationHistoryEntry {
            action_id: row.get(0),
            action_type: row.get(1),
            result: row.get(2),
            started_at: row.get(3),
            event_id: row.get(4),
            incident_status: row.get(5),
        }
    }

    pub fn into_table_row(self) -> Vec<String> {
        vec![
            self.action_id.to_string(),
            self.action_type,
            self.result,
            self.started_at,
            self.event_id.to_string(),
            self.incident_status,
        ]
    }
}

/// Политика, найденная для конкретного типа оборудования и класса аномалии.
pub struct MatchedPolicy {
    pub action_type: String,
    pub policy_id: i32,
    pub criticality: String,
}

impl MatchedPolicy {
    pub fn from_row(row: &Row) -> Self {
        MatchedPolicy {
            action_type: row.get(0),
            policy_id: row.get(1),
            criticality: row.get(2),
        }
    }
}
