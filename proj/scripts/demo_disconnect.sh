#!/usr/bin/env bash
# Обрыв канала объект <-> центр: объект отвязывается от сети rbd_wan.
# Клиенты продолжают работать с узлами через center_net / site_net.
set -euo pipefail

podman network disconnect rbd_wan rbd_site

echo "== Канал оборван =="
echo "Центр больше не видит очередь объекта (FDW):"
podman exec rbd_center psql -U center -d rbd_center -c \
    "SELECT count(*) AS events_on_site FROM site_anomaly_event;" \
    || echo "-> ошибка обращения к внешней таблице: канал недоступен"
echo "Очередь на объекте (копится локально, sync_status = pending):"
podman exec rbd_site psql -U site -d rbd_site -c \
    "SELECT count(*) AS pending FROM anomaly_event WHERE sync_status = 'pending';"
