#!/bin/bash
# tinyboards backup cleanup script
#
# Deletes backup files older than N days.
# Safe to run after the backup cron job.
#
# Usage:
#   ./cleanup-backups.sh
#
# Environment variables:
#   BACKUP_DIR             — Directory containing backups (default: ./backups/)
#   BACKUP_RETENTION_DAYS  — Days to keep backups (default: 7)

set -euo pipefail

BACKUP_DIR="${BACKUP_DIR:-./backups}"
BACKUP_RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-7}"

if [ ! -d "$BACKUP_DIR" ]; then
    echo "[$(date)] Backup directory does not exist: $BACKUP_DIR"
    exit 0
fi

# Find and delete backup files older than retention period
DELETED=$(find "$BACKUP_DIR" -name "tinyboards_backup_*.dump.gz" -type f -mtime +"$BACKUP_RETENTION_DAYS" -print -delete | wc -l)

echo "[$(date)] Deleted $DELETED backup(s) older than $BACKUP_RETENTION_DAYS days from $BACKUP_DIR"
