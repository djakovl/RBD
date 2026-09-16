\set ON_ERROR_STOP on
SELECT 'CREATE DATABASE faculty_db'
WHERE NOT EXISTS (
    SELECT 1 FROM pg_database WHERE datname = 'faculty_db'
)\gexec
