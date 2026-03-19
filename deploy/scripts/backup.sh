#!/bin/bash
# tinyboards database backup script
#
# Creates a compressed pg_dump backup in custom format.
# Safe to run from cron.
#
# Usage:
#   ./backup.sh                          # Uses DATABASE_URL from environment
#   ./backup.sh "postgresql://..."       # Uses the provided connection string
#
# Environment variables:
#   DATABASE_URL          — PostgreSQL connection string (required if no argument)
#   BACKUP_DIR            — Directory to store backups (default: ./backups/)
#
# Cron example — run daily at 3am, keep 7 days of backups:
#   0 3 * * * /opt/tinyboards/deploy/scripts/backup.sh >> /var/log/tinyboards-backup.log 2>&1
#   5 3 * * * /opt/tinyboards/deploy/scripts/cleanup-backups.sh >> /var/log/tinyboards-backup.log 2>&1

set -euo pipefail

DATABASE_URL="${1:-${DATABASE_URL:-}}"
BACKUP_DIR="${BACKUP_DIR:-./backups}"

if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: No database URL provided."
    echo "Usage: $0 [DATABASE_URL]"
    echo "Or set the DATABASE_URL environment variable."
    exit 1
fi

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

TIMESTAMP=$(date +"%Y-%m-%d_%H-%M")
BACKUP_FILE="${BACKUP_DIR}/tinyboards_backup_${TIMESTAMP}.dump"

echo "[$(date)] Starting backup..."

if pg_dump --format=custom --dbname="$DATABASE_URL" --file="$BACKUP_FILE"; then
    # Compress with gzip for additional space savings
    gzip "$BACKUP_FILE"
    FINAL_FILE="${BACKUP_FILE}.gz"
    SIZE=$(du -h "$FINAL_FILE" | cut -f1)
    echo "[$(date)] Backup complete: ${FINAL_FILE} (${SIZE})"
else
    echo "[$(date)] ERROR: Backup failed."
    # Clean up partial file if it exists
    rm -f "$BACKUP_FILE"
    exit 1
fi
