-- Синтетические данные центра.

INSERT INTO site (name, region) VALUES ('Объект КИИ №1', 'ЮФО');

INSERT INTO ci (site_id, ci_type, criticality, parent_ci_id) VALUES
    (1, 'controller',       'high',   NULL),
    (1, 'telemetry_server', 'medium', NULL),
    (1, 'network_gateway',  'high',   NULL),
    (1, 'sensor',           'medium', 1),
    (1, 'test_bench',       'low',    2);

INSERT INTO remediation_policy (ci_type, anomaly_class, action_type, priority) VALUES
    ('controller',       'metric_threshold',       'switch_to_backup', 1),
    ('controller',       'component_unreachable', 'restart',          1),
    ('telemetry_server', 'metric_threshold',       'restart',          2),
    ('telemetry_server', 'suspicious_activity',    'isolate',          1),
    ('network_gateway',  'component_unreachable',  'switch_to_backup', 1),
    ('network_gateway',  'suspicious_activity',    'block',            1),
    ('sensor',           'metric_threshold',       'ignore',           3),
    ('test_bench',       'metric_threshold',       'ignore',           3);

INSERT INTO operator (name, role) VALUES
    ('Иванов И.',  'operator'),
    ('Петрова А.', 'analyst');
