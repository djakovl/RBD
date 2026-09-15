#!/usr/bin/env bash
# Генератор синтетических событий мониторинга (имитация потока телеметрии объекта).
# Использование: ./feed_data.sh [интервал_в_секундах], стоп — Ctrl+C.
set -uo pipefail

INTERVAL="${1:-3}"
METRICS=(cpu_temp latency_ms packet_loss pressure_bar voltage)
CLASSES=(metric_threshold component_unreachable suspicious_activity)

echo "Генерация событий в rbd_site каждые ${INTERVAL}s (Ctrl+C — стоп)"
while true; do
    CI=$(( (RANDOM % 5) + 1 ))
    METRIC=${METRICS[$((RANDOM % ${#METRICS[@]}))]}
    CLASS=${CLASSES[$((RANDOM % ${#CLASSES[@]}))]}
    VALUE="$((RANDOM % 1000)).$((RANDOM % 100))"
    if podman exec rbd_site psql -U site -d rbd_site -q -c \
        "INSERT INTO anomaly_event (ci_id, metric, value, anomaly_class) VALUES (${CI}, '${METRIC}', ${VALUE}, '${CLASS}');"
    then
        echo "[feed] ci=${CI} ${METRIC}=${VALUE} (${CLASS})"
    else
        echo "[feed] узел объекта недоступен" >&2
    fi
    sleep "${INTERVAL}"
done
