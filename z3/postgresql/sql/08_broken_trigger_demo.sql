-- ЗЛОБНО --
\set ON_ERROR_STOP on
SET search_path TO faculty, public;


CREATE OR REPLACE FUNCTION faculty.trg_broken_self_loop() RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
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

ALTER TABLE faculty.grades DISABLE TRIGGER trg_broken_self_loop;





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
