#!/usr/bin/env bash
# Возврат канала: объект снова в сети rbd_wan, репликация догоняет.
set -euo pipefail

podman network connect rbd_wan rbd_site

echo "== Канал восстановлен =="
sleep 3
echo "Состояние подписки на объекте:"
podman exec rbd_site psql -U site -d rbd_site -c \
    "SELECT subname, received_lsn, latest_end_lsn FROM pg_stat_subscription;"
echo "FDW работает, очередь объекта видна из центра:"
podman exec rbd_center psql -U center -d rbd_center -c \
    "SELECT count(*) AS events_on_site FROM site_anomaly_event;"
