//! Читаемые многострочные SQL-константы.
//!
//! Каждый запрос вынесен отдельно от кода, который его выполняет,
//! чтобы SQL можно было проверять и изменять независимо от Rust-логики.

// --- Центр: конфигурационные единицы (ci) ---

pub const SELECT_CI_LIST: &str = "
    SELECT
        ci_id,
        ci_type,
        criticality,
        coalesce(parent_ci_id::text, '-')
    FROM ci
    ORDER BY ci_id
";

pub const INSERT_CI: &str = "
    INSERT INTO ci (site_id, ci_type, criticality)
    VALUES (1, $1, $2)
";

pub const UPDATE_CI_CRITICALITY: &str = "
    UPDATE ci
    SET criticality = $2
    WHERE ci_id = $1
";

pub const DELETE_CI: &str = "
    DELETE FROM ci
    WHERE ci_id = $1
";

// --- Центр: политики самовосстановления ---

pub const SELECT_POLICY_LIST: &str = "
    SELECT
        policy_id,
        ci_type,
        anomaly_class,
        action_type,
        priority
    FROM remediation_policy
    ORDER BY policy_id
";

pub const INSERT_POLICY: &str = "
    INSERT INTO remediation_policy (ci_type, anomaly_class, action_type, priority)
    VALUES ($1, $2, $3, $4)
";

// --- Центр: объединённый журнал инцидентов ---

pub const SELECT_INCIDENT_JOURNAL: &str = "
    SELECT
        src,
        event_id::text,
        ci_id::text,
        to_char(ts, 'YYYY-MM-DD HH24:MI:SS TZ'),
        class,
        status
    FROM v_incident_all
    ORDER BY ts DESC
    LIMIT 30
";

// --- Центр: квитирование событий объекта через FDW ---

pub const SELECT_PENDING_SITE_EVENTS: &str = "
    SELECT
        event_id,
        ci_id,
        anomaly_class,
        to_char(detected_at, 'YYYY-MM-DD HH24:MI:SS TZ')
    FROM site_anomaly_event
    WHERE sync_status = 'pending'
    ORDER BY event_id
    LIMIT 25
";

pub const INSERT_INCIDENT_FROM_PENDING_EVENT: &str = "
    INSERT INTO incident (event_id, ci_id, severity, status)
    SELECT
        e.event_id,
        e.ci_id,
        CASE c.criticality
            WHEN 'high' THEN 'critical'
            WHEN 'medium' THEN 'major'
            ELSE 'minor'
        END,
        'acked'
    FROM site_anomaly_event e
    JOIN ci c ON c.ci_id = e.ci_id
    WHERE e.event_id = $1
      AND e.sync_status = 'pending'
";

pub const MARK_SITE_EVENT_ACKED: &str = "
    UPDATE site_anomaly_event
    SET sync_status = 'acked'
    WHERE event_id = $1
";

pub const MARK_SITE_LOCAL_INCIDENT_ACKED: &str = "
    UPDATE site_local_incident
    SET status = 'acked', updated_at = now()
    WHERE event_id = $1
";

// --- Объект: проверка канала связи с центром через FDW ---

pub const SELECT_CENTER_CI_COUNT: &str = "
    SELECT count(*) FROM center_ci
";

// --- Объект: очередь аномалий ---

pub const INSERT_ANOMALY_EVENT: &str = "
    INSERT INTO anomaly_event (ci_id, metric, value, anomaly_class)
    VALUES ($1, $2, $3, $4)
    RETURNING event_id
";

pub const SELECT_MATCHING_POLICY: &str = "
    SELECT
        p.action_type,
        p.policy_id,
        ci.criticality
    FROM remediation_policy p
    JOIN ci ON ci.ci_type = p.ci_type
    WHERE ci.ci_id = $1
      AND p.anomaly_class = $2
";

pub const INSERT_LOCAL_INCIDENT: &str = "
    INSERT INTO local_incident (event_id)
    VALUES ($1)
    RETURNING local_incident_id
";

pub const INSERT_REMEDIATION_ACTION: &str = "
    INSERT INTO remediation_action (local_incident_id, policy_id, action_type)
    VALUES ($1, $2, $3)
";

pub const SELECT_ANOMALY_QUEUE: &str = "
    SELECT
        event_id,
        ci_id,
        metric,
        value::text,
        anomaly_class,
        sync_status,
        to_char(detected_at, 'YYYY-MM-DD HH24:MI:SS TZ')
    FROM anomaly_event
    ORDER BY event_id DESC
    LIMIT 20
";

// --- Объект: действия самовосстановления ---

pub const SELECT_OPEN_REMEDIATION_ACTIONS: &str = "
    SELECT
        a.action_id,
        a.action_type,
        a.local_incident_id,
        l.event_id
    FROM remediation_action a
    JOIN local_incident l ON l.local_incident_id = a.local_incident_id
    WHERE a.result IS NULL
";

pub const UPDATE_REMEDIATION_ACTION_RESULT: &str = "
    UPDATE remediation_action
    SET result = $1, finished_at = now()
    WHERE action_id = $2
";

pub const UPDATE_LOCAL_INCIDENT_STATUS_BY_ACTION: &str = "
    UPDATE local_incident
    SET status = $1, updated_at = now()
    WHERE local_incident_id = (
        SELECT local_incident_id
        FROM remediation_action
        WHERE action_id = $2
    )
";

pub const SELECT_REMEDIATION_HISTORY: &str = "
    SELECT
        a.action_id,
        a.action_type,
        coalesce(a.result, '(выполняется)'),
        to_char(a.started_at, 'YYYY-MM-DD HH24:MI:SS TZ'),
        l.event_id,
        l.status
    FROM remediation_action a
    JOIN local_incident l ON l.local_incident_id = a.local_incident_id
    ORDER BY a.action_id DESC
    LIMIT 20
";
