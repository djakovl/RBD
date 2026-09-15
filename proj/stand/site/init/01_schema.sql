-- Узел объекта КИИ: оперативные данные и реплика справочников.

-- Очередь событий (свой фрагмент, целиком в центр не копируется).
CREATE TABLE anomaly_event (
    event_id      BIGSERIAL PRIMARY KEY,
    ci_id         INT NOT NULL,
    detected_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    metric        TEXT NOT NULL,
    value         NUMERIC(12,3) NOT NULL,
    anomaly_class TEXT NOT NULL,
    sync_status   TEXT NOT NULL DEFAULT 'pending'
                  CHECK (sync_status IN ('pending', 'acked', 'rejected'))
);

-- Локальный инцидент (живёт на объекте до переноса в журнал центра).
CREATE TABLE local_incident (
    local_incident_id BIGSERIAL PRIMARY KEY,
    event_id   BIGINT NOT NULL REFERENCES anomaly_event(event_id),
    status     TEXT NOT NULL DEFAULT 'detected'
               CHECK (status IN ('detected', 'acked', 'in_remediation', 'resolved', 'escalated', 'rejected')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Действие самовосстановления: выполняется и фиксируется на объекте.
CREATE TABLE remediation_action (
    action_id   BIGSERIAL PRIMARY KEY,
    local_incident_id BIGINT NOT NULL REFERENCES local_incident(local_incident_id),
    policy_id   INT,
    action_type TEXT NOT NULL,
    started_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ,
    result      TEXT CHECK (result IN ('pending', 'success', 'failed'))
);

-- Реплика справочников: заполняется логической репликацией из центра
-- (см. links/replication_site.sql). Имена совпадают с таблицами центра —
-- так подписка находит таблицы на своей стороне.
CREATE TABLE ci (
    ci_id        INT PRIMARY KEY,
    site_id      INT NOT NULL,
    ci_type      TEXT NOT NULL,
    criticality  TEXT NOT NULL,
    parent_ci_id INT
);

CREATE TABLE remediation_policy (
    policy_id     INT PRIMARY KEY,
    ci_type       TEXT NOT NULL,
    anomaly_class TEXT NOT NULL,
    action_type   TEXT NOT NULL,
    priority      INT NOT NULL DEFAULT 1
);
