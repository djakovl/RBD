-- Центр мониторинга: справочники, журнал инцидентов, витрина надёжности.

CREATE TABLE site (
    site_id  SERIAL PRIMARY KEY,
    name     TEXT NOT NULL,
    region   TEXT NOT NULL
);

CREATE TABLE ci (
    ci_id        SERIAL PRIMARY KEY,
    site_id      INT NOT NULL REFERENCES site(site_id),
    ci_type      TEXT NOT NULL,
    criticality  TEXT NOT NULL CHECK (criticality IN ('high', 'medium', 'low')),
    parent_ci_id INT REFERENCES ci(ci_id)
);

CREATE TABLE remediation_policy (
    policy_id     SERIAL PRIMARY KEY,
    ci_type       TEXT NOT NULL,
    anomaly_class TEXT NOT NULL,
    action_type   TEXT NOT NULL,
    priority      INT  NOT NULL DEFAULT 1,
    UNIQUE (ci_type, anomaly_class)
);

CREATE TABLE operator (
    operator_id SERIAL PRIMARY KEY,
    name        TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'operator'
);

-- event_id — идентификатор события с объекта (таблица на другом узле,
-- связь логическая: переносится через FDW, поэтому без FOREIGN KEY).
CREATE TABLE incident (
    incident_id SERIAL PRIMARY KEY,
    event_id    BIGINT NOT NULL UNIQUE,
    ci_id       INT NOT NULL REFERENCES ci(ci_id),
    severity    TEXT NOT NULL CHECK (severity IN ('critical', 'major', 'minor')),
    status      TEXT NOT NULL DEFAULT 'detected'
                CHECK (status IN ('detected', 'acked', 'in_remediation', 'resolved', 'escalated', 'rejected')),
    opened_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ
);

CREATE TABLE remediation_action_log (
    log_id     SERIAL PRIMARY KEY,
    incident_id INT NOT NULL REFERENCES incident(incident_id),
    action_type TEXT NOT NULL,
    result      TEXT CHECK (result IN ('pending', 'success', 'failed')),
    operator_id INT REFERENCES operator(operator_id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE reliability_metric (
    period            DATE NOT NULL,
    ci_id             INT NOT NULL REFERENCES ci(ci_id),
    incident_count    INT NOT NULL DEFAULT 0,
    mttr_minutes      NUMERIC(10,2),
    self_healing_rate NUMERIC(5,4),
    PRIMARY KEY (period, ci_id)
);
