-- FDW на ЦЕНТРЕ: шлюз к очереди объекта + единое представление.
-- Применяется скриптом setup_links.sh (после старта обоих узлов).

CREATE EXTENSION IF NOT EXISTS postgres_fdw;

DROP SERVER IF EXISTS site_srv CASCADE;
CREATE SERVER site_srv FOREIGN DATA WRAPPER postgres_fdw
    OPTIONS (host 'rbd_site', port '5432', dbname 'rbd_site', connect_timeout '5');

CREATE USER MAPPING FOR CURRENT_USER SERVER site_srv
    OPTIONS (user 'site', password 'site_pass');

DROP FOREIGN TABLE IF EXISTS site_anomaly_event CASCADE;
CREATE FOREIGN TABLE site_anomaly_event (
    event_id      BIGINT,
    ci_id         INT,
    detected_at   TIMESTAMPTZ,
    metric        TEXT,
    value         NUMERIC(12,3),
    anomaly_class TEXT,
    sync_status   TEXT
) SERVER site_srv OPTIONS (schema_name 'public', table_name 'anomaly_event');

DROP FOREIGN TABLE IF EXISTS site_local_incident CASCADE;
CREATE FOREIGN TABLE site_local_incident (
    local_incident_id BIGINT,
    event_id   BIGINT,
    status     TEXT,
    updated_at TIMESTAMPTZ
) SERVER site_srv OPTIONS (schema_name 'public', table_name 'local_incident');

DROP FOREIGN TABLE IF EXISTS site_remediation_action CASCADE;
CREATE FOREIGN TABLE site_remediation_action (
    action_id         BIGINT,
    local_incident_id BIGINT,
    policy_id   INT,
    action_type TEXT,
    started_at  TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    result      TEXT
) SERVER site_srv OPTIONS (schema_name 'public', table_name 'remediation_action');

-- Единое представление для клиента центра: журнал + внешняя очередь объекта.
-- Скрывает от клиента, где физически лежат данные (распределённость прозрачна).
CREATE OR REPLACE VIEW v_incident_all AS
SELECT 'journal' AS src, event_id, ci_id, opened_at AS ts, severity AS class, status
FROM incident
UNION ALL
SELECT 'queue', event_id, ci_id, detected_at, anomaly_class, sync_status
FROM site_anomaly_event;
