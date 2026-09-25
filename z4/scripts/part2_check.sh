#!/usr/bin/env bash
set -euo pipefail
podman exec z4-university psql -U postgres -d faculty_db -c "SELECT tablename FROM pg_tables WHERE schemaname='faculty' ORDER BY tablename;"
podman exec z4-personal psql -U postgres -d personal_db -c "SELECT tablename FROM pg_tables WHERE schemaname='faculty' ORDER BY tablename;"
