-- FDW на ОБЪЕКТЕ: шлюз к справочникам центра (онлайн-проверки).
-- При обрыве канала клиент объекта переключается на локальную реплику.
-- Применяется скриптом setup_links.sh.

CREATE EXTENSION IF NOT EXISTS postgres_fdw;

DROP SERVER IF EXISTS center_srv CASCADE;
CREATE SERVER center_srv FOREIGN DATA WRAPPER postgres_fdw
    OPTIONS (host 'rbd_center', port '5432', dbname 'rbd_center', connect_timeout '5');

CREATE USER MAPPING FOR CURRENT_USER SERVER center_srv
    OPTIONS (user 'center', password 'center_pass');

DROP FOREIGN TABLE IF EXISTS center_ci CASCADE;
CREATE FOREIGN TABLE center_ci (
    ci_id        INT,
    site_id      INT,
    ci_type      TEXT,
    criticality  TEXT,
    parent_ci_id INT
) SERVER center_srv OPTIONS (schema_name 'public', table_name 'ci');

DROP FOREIGN TABLE IF EXISTS center_policy CASCADE;
CREATE FOREIGN TABLE center_policy (
    policy_id     INT,
    ci_type       TEXT,
    anomaly_class TEXT,
    action_type   TEXT,
    priority      INT
) SERVER center_srv OPTIONS (schema_name 'public', table_name 'remediation_policy');
