# Backup and Restore

Regular backups are essential for any self-hosted service. TinyBoards has two components to back up: the PostgreSQL database and uploaded media files.

## Table of Contents

- [What to Back Up](#what-to-back-up)
- [Database Backup](#database-backup)
- [Media Backup](#media-backup)
- [Automated Backup Script](#automated-backup-script)
- [Restore Procedure](#restore-procedure)
- [Offsite Backups](#offsite-backups)

## What to Back Up

| Component | Location (Docker) | Location (From Source) | Priority |
|-----------|-------------------|----------------------|----------|
| Database  | `postgres_data` volume | PostgreSQL data directory | Critical |
| Media     | `media_data` volume | `/opt/tinyboards/media` | High |
| Config    | `.env`, nginx configs | `.env`, nginx configs | High |

The database contains all user accounts, posts, comments, settings, and metadata. Media contains uploaded images and files. Both should be backed up regularly.

## Database Backup

### Manual Backup (Docker)

```bash
docker compose exec postgres pg_dump \
  -U tinyboards \
  -d tinyboards \
  --format=custom \
  --compress=9 \
  > backup_$(date +%Y%m%d_%H%M%S).dump
```

### Manual Backup (From Source)

```bash
pg_dump \
  -U tinyboards \
  -d tinyboards \
  --format=custom \
  --compress=9 \
  > backup_$(date +%Y%m%d_%H%M%S).dump
```

### Cron-Based Automated Backup

Create `/opt/tinyboards/scripts/backup.sh`:

```bash
#!/bin/bash
set -euo pipefail

# Configuration
BACKUP_DIR="/opt/backups/tinyboards"
RETENTION_DAYS=30
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/db_${TIMESTAMP}.dump"

# Create backup directory
mkdir -p "${BACKUP_DIR}"

# Database backup
docker compose -f /path/to/tinyboards-rewrite/docker-compose.yml \
  exec -T postgres pg_dump \
  -U tinyboards \
  -d tinyboards \
  --format=custom \
  --compress=9 \
  > "${BACKUP_FILE}"

# Verify the backup file is not empty
if [ ! -s "${BACKUP_FILE}" ]; then
  echo "ERROR: Backup file is empty" >&2
  rm -f "${BACKUP_FILE}"
  exit 1
fi

# Remove backups older than retention period
find "${BACKUP_DIR}" -name "db_*.dump" -mtime "+${RETENTION_DAYS}" -delete

echo "Backup complete: ${BACKUP_FILE} ($(du -h "${BACKUP_FILE}" | cut -f1))"
```

```bash
chmod +x /opt/tinyboards/scripts/backup.sh
```

Add to crontab (`crontab -e`):

```cron
# TinyBoards database backup — daily at 2:00 AM
0 2 * * * /opt/tinyboards/scripts/backup.sh >> /var/log/tinyboards-backup.log 2>&1
```

### From-Source Cron Template

If not using Docker, modify the pg_dump command:

```bash
# Replace the docker compose exec line with:
pg_dump \
  -U tinyboards \
  -h localhost \
  -d tinyboards \
  --format=custom \
  --compress=9 \
  > "${BACKUP_FILE}"
```

You may need a `.pgpass` file for non-interactive authentication:

```bash
echo "localhost:5432:tinyboards:tinyboards:your_password" > ~/.pgpass
chmod 600 ~/.pgpass
```

## Media Backup

### Sync Media to a Backup Location

```bash
rsync -av --delete /path/to/media/ /opt/backups/tinyboards/media/
```

### Docker Media Backup

```bash
# Find the media volume mount point
docker volume inspect tinyboards_media_data --format '{{ .Mountpoint }}'

# Sync to backup
rsync -av --delete "$(docker volume inspect tinyboards_media_data --format '{{ .Mountpoint }}')/" \
  /opt/backups/tinyboards/media/
```

## Automated Backup Script

A complete script that backs up both database and media:

```bash
#!/bin/bash
set -euo pipefail

BACKUP_DIR="/opt/backups/tinyboards"
RETENTION_DAYS=30
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
COMPOSE_DIR="/path/to/tinyboards-rewrite"

mkdir -p "${BACKUP_DIR}"

# 1. Database backup
echo "[$(date)] Starting database backup..."
docker compose -f "${COMPOSE_DIR}/docker-compose.yml" \
  exec -T postgres pg_dump \
  -U tinyboards \
  -d tinyboards \
  --format=custom \
  --compress=9 \
  > "${BACKUP_DIR}/db_${TIMESTAMP}.dump"

# 2. Media backup
echo "[$(date)] Starting media backup..."
MEDIA_MOUNT=$(docker volume inspect tinyboards_media_data --format '{{ .Mountpoint }}')
tar -czf "${BACKUP_DIR}/media_${TIMESTAMP}.tar.gz" -C "${MEDIA_MOUNT}" .

# 3. Config backup
echo "[$(date)] Backing up configuration..."
tar -czf "${BACKUP_DIR}/config_${TIMESTAMP}.tar.gz" \
  -C "${COMPOSE_DIR}" \
  .env docker-compose.yml config/

# 4. Clean old backups
find "${BACKUP_DIR}" -name "db_*.dump" -mtime "+${RETENTION_DAYS}" -delete
find "${BACKUP_DIR}" -name "media_*.tar.gz" -mtime "+${RETENTION_DAYS}" -delete
find "${BACKUP_DIR}" -name "config_*.tar.gz" -mtime "+${RETENTION_DAYS}" -delete

echo "[$(date)] Backup complete."
ls -lh "${BACKUP_DIR}/"*_${TIMESTAMP}*
```

## Restore Procedure

### 1. Stop the Application

```bash
# Docker
docker compose down

# From source
sudo systemctl stop tinyboards-backend tinyboards-frontend
```

### 2. Restore the Database

```bash
# Docker: start only postgres
docker compose up -d postgres
sleep 5

# Drop and recreate the database
docker compose exec postgres psql -U tinyboards -c "DROP DATABASE IF EXISTS tinyboards;"
docker compose exec postgres psql -U tinyboards -c "CREATE DATABASE tinyboards OWNER tinyboards;"

# Restore from backup
docker compose exec -T postgres pg_restore \
  -U tinyboards \
  -d tinyboards \
  --no-owner \
  --clean \
  --if-exists \
  < /opt/backups/tinyboards/db_20260301_020000.dump
```

For from-source installs:

```bash
sudo -u postgres psql -c "DROP DATABASE IF EXISTS tinyboards;"
sudo -u postgres psql -c "CREATE DATABASE tinyboards OWNER tinyboards;"

pg_restore \
  -U tinyboards \
  -h localhost \
  -d tinyboards \
  --no-owner \
  --clean \
  --if-exists \
  /opt/backups/tinyboards/db_20260301_020000.dump
```

### 3. Restore Media Files

```bash
# Docker
MEDIA_MOUNT=$(docker volume inspect tinyboards_media_data --format '{{ .Mountpoint }}')
sudo tar -xzf /opt/backups/tinyboards/media_20260301_020000.tar.gz -C "${MEDIA_MOUNT}"

# From source
tar -xzf /opt/backups/tinyboards/media_20260301_020000.tar.gz -C /opt/tinyboards/media/
```

### 4. Restart the Application

```bash
# Docker
docker compose up -d

# From source
sudo systemctl start tinyboards-backend tinyboards-frontend
```

### 5. Verify

```bash
# Check health
curl -s http://localhost:8536/
# Should return: ok

# Check a few pages in the browser
```

## Offsite Backups

For disaster recovery, store backups offsite. Add one of these to the backup script:

### rsync to Remote Server

```bash
rsync -avz "${BACKUP_DIR}/" user@backup-server:/backups/tinyboards/
```

### S3-Compatible Storage

```bash
aws s3 sync "${BACKUP_DIR}/" s3://your-backup-bucket/tinyboards/ \
  --storage-class STANDARD_IA
```

### Backblaze B2

```bash
b2 sync "${BACKUP_DIR}/" b2://your-bucket/tinyboards/
```
