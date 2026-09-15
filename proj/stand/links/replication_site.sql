-- Логическая репликация: подписка ОБЪЕКТА на справочники центра.
-- Асинхронная: при обрыве канала подписчик отстаёт, при возврате догоняет сам.
-- Таблицы ci и remediation_policy на объекте уже созданы (site/init).

DROP SUBSCRIPTION IF EXISTS sub_ref;
CREATE SUBSCRIPTION sub_ref
    CONNECTION 'host=rbd_center port=5432 dbname=rbd_center user=repl_user password=repl_pass connect_timeout=5'
    PUBLICATION pub_ref
    WITH (copy_data = true);
