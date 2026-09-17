-- ЗЛОБНО --
\set ON_ERROR_STOP on
SET search_path TO faculty, public;

-- ---------------------------------------------------------------------
-- "Ломающийся" триггер: при UPDATE оценки обновляет ту же строку снова,
-- вызывая сам себя рекурсивно. Никакого условия выхода нет специально —
-- это демонстрация антипаттерна "триггер без защиты от самозапуска".
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.trg_broken_self_loop() RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    -- Намеренно НЕ проверяем, что значение уже такое же (нет условия останова).
    -- Каждый UPDATE вызывает этот же триггер повторно -> бесконечная рекурсия.
    UPDATE faculty.grades
    SET grade = NEW.grade
    WHERE id = NEW.id;

    RETURN NEW;
END;
$$;

COMMENT ON FUNCTION faculty.trg_broken_self_loop()
    IS 'ДЕМО ОШИБКИ: триггер без условия останова, вызывающий рекурсивный UPDATE той же строки (самозацикливание)';

DROP TRIGGER IF EXISTS trg_broken_self_loop ON faculty.grades;
CREATE TRIGGER trg_broken_self_loop
    AFTER UPDATE OF grade ON faculty.grades
    FOR EACH ROW
    EXECUTE FUNCTION faculty.trg_broken_self_loop();

-- По умолчанию триггер ВЫКЛЮЧЕН, чтобы не сломать обычную работу БД.
ALTER TABLE faculty.grades DISABLE TRIGGER trg_broken_self_loop;

-- ---------------------------------------------------------------------
-- Функции-обёртки для включения/выключения "ломающегося" триггера,
-- чтобы демо-клиент мог безопасно показать сбой и восстановить работу.
-- ---------------------------------------------------------------------
CREATE OR REPLACE FUNCTION faculty.enable_broken_trigger() RETURNS void
LANGUAGE sql
AS $$
    ALTER TABLE faculty.grades ENABLE TRIGGER trg_broken_self_loop;
$$;

CREATE OR REPLACE FUNCTION faculty.disable_broken_trigger() RETURNS void
LANGUAGE sql
AS $$
    ALTER TABLE faculty.grades DISABLE TRIGGER trg_broken_self_loop;
$$;

COMMENT ON FUNCTION faculty.enable_broken_trigger() IS 'ДЕМО: включает сломанный триггер самозацикливания перед показом ошибки';
COMMENT ON FUNCTION faculty.disable_broken_trigger() IS 'ДЕМО: выключает сломанный триггер после показа ошибки';
