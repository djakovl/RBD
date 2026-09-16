-- Логическая репликация: публикация справочников на ЦЕНТРЕ.
-- repl_user имеет право REPLICATION: оно обязательно, чтобы подписчик
-- на объекте мог читать изменения из WAL центра.

DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'repl_user') THEN
        CREATE ROLE repl_user LOGIN REPLICATION PASSWORD 'repl_pass';
    END IF;
END
$$;

-- Нужен и при повторном запуске: пользователь мог быть создан старой версией
-- скрипта без этого права.
ALTER ROLE repl_user WITH REPLICATION;

GRANT SELECT ON TABLE ci, remediation_policy TO repl_user;

DROP PUBLICATION IF EXISTS pub_ref;
CREATE PUBLICATION pub_ref FOR TABLE ci, remediation_policy;
