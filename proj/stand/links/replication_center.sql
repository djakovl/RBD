-- Логическая репликация: публикация справочников на ЦЕНТРЕ.
-- Пользователь подписчика — repl_user (пароль ниже, при смене поправить
-- и replication_site.sql).

DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'repl_user') THEN
        CREATE ROLE repl_user LOGIN PASSWORD 'repl_pass';
    END IF;
END
$$;

GRANT SELECT ON TABLE ci, remediation_policy TO repl_user;

DROP PUBLICATION IF EXISTS pub_ref;
CREATE PUBLICATION pub_ref FOR TABLE ci, remediation_policy;
