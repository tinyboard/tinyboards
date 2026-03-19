#!/usr/bin/env bash
set -euo pipefail

# Creates a compressed custom-format database backup with retention policy.
# Designed to be dropped into cron for automated daily backups.
#
# Usage: ./backup.sh
# Cron:  0 2 * * * /opt/tinyboards/scripts/backup.sh
#
# Environment variables:
#   BACKUP_DIR         Output directory (default: /var/backups/tinyboards)
#   BACKUP_RETAIN_DAYS Days to keep old backups (default: 7)
#   POSTGRES_HOST      Database host (default: localhost)
#   POSTGRES_PORT      Database port (default: 5432)
#   POSTGRES_DB        Database name (default: tinyboards)
#   POSTGRES_USER      Database user (default: tinyboards)
#   POSTGRES_PASSWORD  Database password
#   DATABASE_URL       Full connection string (overrides individual vars)

BACKUP_DIR="${BACKUP_DIR:-/var/backups/tinyboards}"
BACKUP_RETAIN_DAYS="${BACKUP_RETAIN_DAYS:-7}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
FILENAME="tinyboards_${TIMESTAMP}.dump.gz"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Load .env if present
if [ -f "${PROJECT_DIR}/.env" ]; then
    # shellcheck disable=SC1091
    source "${PROJECT_DIR}/.env"
elif [ -f /opt/tinyboards/.env ]; then
    # shellcheck disable=SC1091
    source /opt/tinyboards/.env
fi

# Build connection string from individual vars if DATABASE_URL is not set
if [ -z "${DATABASE_URL:-}" ]; then
    PG_HOST="${POSTGRES_HOST:-localhost}"
    PG_PORT="${POSTGRES_PORT:-5432}"
    PG_DB="${POSTGRES_DB:-tinyboards}"
    PG_USER="${POSTGRES_USER:-tinyboards}"
    PG_PASS="${POSTGRES_PASSWORD:-}"

    if [ -z "$PG_PASS" ]; then
        echo "Error: Neither DATABASE_URL nor POSTGRES_PASSWORD is set."
        exit 1
    fi

    export PGPASSWORD="$PG_PASS"
    PG_ARGS="-h $PG_HOST -p $PG_PORT -U $PG_USER -d $PG_DB"
else
    PG_ARGS="$DATABASE_URL"
fi

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

echo "Starting backup..."

# Dump in custom format (binary, supports selective restore) and gzip
# shellcheck disable=SC2086
pg_dump --format=custom $PG_ARGS | gzip > "${BACKUP_DIR}/${FILENAME}"

FILESIZE=$(du -h "${BACKUP_DIR}/${FILENAME}" | cut -f1)
echo "Backup complete: ${FILENAME} (${FILESIZE})"

# Retention: delete backups older than BACKUP_RETAIN_DAYS
DELETED=$(find "$BACKUP_DIR" -name "tinyboards_*.dump.gz" -mtime "+${BACKUP_RETAIN_DAYS}" -print -delete | wc -l)

if [ "$DELETED" -gt 0 ]; then
    echo "Cleaned up ${DELETED} old backup(s) (retention: ${BACKUP_RETAIN_DAYS} days)."
fi
