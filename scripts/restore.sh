#!/usr/bin/env bash
set -euo pipefail

# Restores a tinyboards database from a custom-format compressed backup.
# WARNING: This drops and recreates the database. All existing data will be lost.
#
# Usage: ./restore.sh <backup_file>
# Example: ./restore.sh tinyboards_20260319_020000.dump.gz
#
# The backup file can be a full path or a filename in /var/backups/tinyboards/.
#
# Environment variables:
#   POSTGRES_HOST      Database host (default: localhost)
#   POSTGRES_PORT      Database port (default: 5432)
#   POSTGRES_DB        Database name (default: tinyboards)
#   POSTGRES_USER      Database user (default: tinyboards)
#   POSTGRES_PASSWORD  Database password
#   DATABASE_URL       Full connection string (overrides individual vars)

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

# Resolve connection parameters
PG_HOST="${POSTGRES_HOST:-localhost}"
PG_PORT="${POSTGRES_PORT:-5432}"
PG_DB="${POSTGRES_DB:-tinyboards}"
PG_USER="${POSTGRES_USER:-tinyboards}"
PG_PASS="${POSTGRES_PASSWORD:-}"

if [ -z "${DATABASE_URL:-}" ] && [ -z "$PG_PASS" ]; then
    echo "Error: Neither DATABASE_URL nor POSTGRES_PASSWORD is set."
    exit 1
fi

if [ -n "${DATABASE_URL:-}" ]; then
    PG_DB=$(echo "$DATABASE_URL" | sed -E 's|.*/([^?]+).*|\1|')
    DB_ADMIN_URL=$(echo "$DATABASE_URL" | sed -E "s|/[^/?]+(\?.*)?$|/postgres|")
else
    export PGPASSWORD="$PG_PASS"
    DB_ADMIN_URL="postgresql://${PG_USER}:${PG_PASS}@${PG_HOST}:${PG_PORT}/postgres"
fi

if [ $# -lt 1 ]; then
    echo "Usage: $0 <backup_file>"
    echo "Example: $0 tinyboards_20260319_020000.dump.gz"
    echo ""
    echo "Available backups:"
    ls -lh /var/backups/tinyboards/tinyboards_*.dump.gz 2>/dev/null || echo "  (none found)"
    exit 1
fi

BACKUP_FILE="$1"

# If given a filename without a path, look in the default backup directory
if [ ! -f "$BACKUP_FILE" ]; then
    BACKUP_FILE="/var/backups/tinyboards/${BACKUP_FILE}"
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "Error: Backup file not found: $1"
    exit 1
fi

echo ""
echo "============================================"
echo "  DESTRUCTIVE OPERATION — DATABASE RESTORE"
echo "============================================"
echo ""
echo "  Backup file : $(basename "$BACKUP_FILE")"
echo "  Database    : ${PG_DB}"
echo "  Host        : ${PG_HOST}:${PG_PORT}"
echo ""
echo "  WARNING: This will DROP the existing database"
echo "  and replace it with the backup contents."
echo "  ALL CURRENT DATA WILL BE LOST."
echo ""
echo "============================================"
echo ""
read -rp "Type 'yes' to confirm: " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "Restore cancelled."
    exit 0
fi

echo ""
echo "Terminating existing connections to ${PG_DB}..."
psql "$DB_ADMIN_URL" -q -c \
    "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${PG_DB}' AND pid <> pg_backend_pid();" \
    2>/dev/null || true

echo "Dropping database ${PG_DB}..."
psql "$DB_ADMIN_URL" -q -c "DROP DATABASE IF EXISTS \"${PG_DB}\";"

echo "Creating database ${PG_DB}..."
psql "$DB_ADMIN_URL" -q -c "CREATE DATABASE \"${PG_DB}\";"

echo "Restoring from backup..."
# Decompress and restore using pg_restore for custom-format dumps
gunzip -c "$BACKUP_FILE" | pg_restore -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" --no-owner --no-privileges 2>&1 || true

echo ""
echo "Verifying restore..."
if [ -n "${DATABASE_URL:-}" ]; then
    USER_COUNT=$(psql "$DATABASE_URL" -t -A -c "SELECT count(*) FROM users;" 2>/dev/null || echo "?")
else
    USER_COUNT=$(psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$PG_DB" -t -A -c "SELECT count(*) FROM users;" 2>/dev/null || echo "?")
fi

echo "Restore complete. Database ${PG_DB} has been restored from:"
echo "  $(basename "$BACKUP_FILE")"
echo "  Users in database: ${USER_COUNT}"
