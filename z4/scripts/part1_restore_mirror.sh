#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DUMP="$ROOT/z4/part1/dumps/faculty_snapshot.dump"
test -f "$DUMP" || { echo "Сначала запусти part1_bootstrap.sh"; exit 1; }
until podman exec z4-mirror pg_isready -U postgres >/dev/null; do sleep 1; done
podman exec z4-mirror createdb -U postgres faculty_db 2>/dev/null || true
podman exec -i z4-mirror pg_restore -U postgres -d faculty_db --clean --if-exists < "$DUMP"
echo "Зеркальная копия восстановлена в z4-mirror"
