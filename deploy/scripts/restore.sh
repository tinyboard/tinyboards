#!/bin/bash
# tinyboards database restore script
#
# Restores a pg_dump backup created by backup.sh.
#
# WARNING: This will DROP and recreate the target database.
# All existing data will be destroyed.
#
# Usage:
#   ./restore.sh backup_file.dump.gz                     # Uses DATABASE_URL from env
#   ./restore.sh backup_file.dump.gz "postgresql://..."  # Uses provided connection string
#   ./restore.sh --yes backup_file.dump.gz               # Skip confirmation prompt

set -euo pipefail

SKIP_CONFIRM=false

# Parse --yes flag
if [ "${1:-}" = "--yes" ]; then
    SKIP_CONFIRM=true
    shift
fi

BACKUP_FILE="${1:-}"
DATABASE_URL="${2:-${DATABASE_URL:-}}"

if [ -z "$BACKUP_FILE" ]; then
    echo "ERROR: No backup file specified."
    echo "Usage: $0 [--yes] <backup_file> [DATABASE_URL]"
    exit 1
fi

if [ ! -f "$BACKUP_FILE" ]; then
    echo "ERROR: Backup file not found: $BACKUP_FILE"
    exit 1
fi

if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: No database URL provided."
    echo "Usage: $0 [--yes] <backup_file> [DATABASE_URL]"
    echo "Or set the DATABASE_URL environment variable."
    exit 1
fi

# Confirmation prompt
if [ "$SKIP_CONFIRM" = false ]; then
    echo "============================================================"
    echo "  WARNING: This will DESTROY the existing database and"
    echo "  replace it with the contents of:"
    echo "    $BACKUP_FILE"
    echo "============================================================"
    echo ""
    read -r -p "Type 'yes' to continue: " CONFIRM
    if [ "$CONFIRM" != "yes" ]; then
        echo "Restore cancelled."
        exit 0
    fi
fi

echo "[$(date)] Starting restore from: $BACKUP_FILE"

# Determine if the file is gzipped
RESTORE_FILE="$BACKUP_FILE"
TEMP_FILE=""

if [[ "$BACKUP_FILE" == *.gz ]]; then
    echo "[$(date)] Decompressing backup..."
    TEMP_FILE=$(mktemp /tmp/tinyboards_restore_XXXXXX.dump)
    gunzip -c "$BACKUP_FILE" > "$TEMP_FILE"
    RESTORE_FILE="$TEMP_FILE"
fi

# Restore using pg_restore with --clean to drop existing objects first
# --if-exists prevents errors when objects don't exist yet
if pg_restore \
    --dbname="$DATABASE_URL" \
    --clean \
    --if-exists \
    --no-owner \
    --no-privileges \
    --single-transaction \
    "$RESTORE_FILE"; then
    echo "[$(date)] Restore complete."
else
    echo "[$(date)] ERROR: Restore failed. The database may be in an inconsistent state."
    # Clean up temp file
    if [ -n "$TEMP_FILE" ]; then
        rm -f "$TEMP_FILE"
    fi
    exit 1
fi

# Clean up temp file
if [ -n "$TEMP_FILE" ]; then
    rm -f "$TEMP_FILE"
fi

echo "[$(date)] Database restored successfully from: $BACKUP_FILE"
