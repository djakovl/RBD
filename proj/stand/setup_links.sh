#!/usr/bin/env bash
# Настройка связей между узлами: FDW в обе стороны + логическая репликация.
# Запускать после `podman compose up -d` (можно повторно — идемпотентно).
set -euo pipefail
cd "$(dirname "$0")"

echo "Ждём готовности обеих нод..."
until podman exec rbd_center pg_isready -U center -d rbd_center >/dev/null 2>&1; do sleep 1; done
until podman exec rbd_site pg_isready -U site -d rbd_site >/dev/null 2>&1; do sleep 1; done

echo "[1/4] FDW на центре (шлюз к очереди объекта)..."
podman exec -i rbd_center psql -U center -d rbd_center -v ON_ERROR_STOP=1 -q < links/fdw_center.sql

echo "[2/4] FDW на объекте (шлюз к справочникам центра)..."
podman exec -i rbd_site psql -U site -d rbd_site -v ON_ERROR_STOP=1 -q < links/fdw_site.sql

echo "[3/4] Публикация справочников на центре..."
podman exec -i rbd_center psql -U center -d rbd_center -v ON_ERROR_STOP=1 -q < links/replication_center.sql

echo "[4/4] Подписка на объекте..."
podman exec -i rbd_site psql -U site -d rbd_site -v ON_ERROR_STOP=1 -q < links/replication_site.sql

echo "Готово. Проверка: реплика справочников на объекте (строк в ci):"
podman exec rbd_site psql -U site -d rbd_site -c "SELECT count(*) AS ci_rows FROM ci;"
echo "FDW из центра (очередь объекта):"
podman exec rbd_center psql -U center -d rbd_center -c "SELECT count(*) AS events_on_site FROM site_anomaly_event;"
