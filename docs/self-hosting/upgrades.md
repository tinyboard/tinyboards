# Upgrades

This guide covers how to upgrade your TinyBoards instance between versions.

## Table of Contents

- [Before Upgrading](#before-upgrading)
- [Docker Upgrade Procedure](#docker-upgrade-procedure)
- [From-Source Upgrade Procedure](#from-source-upgrade-procedure)
- [Database Migrations](#database-migrations)
- [Rolling Back](#rolling-back)

## Before Upgrading

1. **Read the release notes.** Check for breaking changes, new required environment variables, or manual migration steps.

2. **Back up your database and media.** See [backup-restore.md](backup-restore.md).

   ```bash
   # Quick pre-upgrade backup
   docker compose exec -T postgres pg_dump \
     -U tinyboards -d tinyboards \
     --format=custom --compress=9 \
     > pre-upgrade-$(date +%Y%m%d).dump
   ```

3. **Check your environment variables.** New versions may introduce new required variables. Compare your `.env` with the latest `.env.example`:

   ```bash
   diff .env .env.example
   ```

## Docker Upgrade Procedure

### Pull and Rebuild

```bash
cd /path/to/tinyboards-rewrite

# Pull latest code
git fetch origin main
git pull origin main

# Check for new env vars
diff .env .env.example

# Rebuild containers
docker compose build

# Restart with new images
docker compose up -d
```

The backend runs database migrations automatically on startup. Watch the logs to confirm:

```bash
docker compose logs -f backend
```

### Using Pre-Built Images

If using pre-built images from a container registry:

```bash
# Pull latest images
docker compose pull

# Restart
docker compose up -d
```

## From-Source Upgrade Procedure

### 1. Pull Latest Code

```bash
cd /opt/tinyboards
git fetch origin main
git pull origin main
```

### 2. Rebuild Backend

```bash
cd backend
cargo build --release
```

### 3. Rebuild Frontend

```bash
cd ../frontend
pnpm install
pnpm build
```

### 4. Apply Migrations

Migrations run automatically when the backend starts. Alternatively:

```bash
cd /opt/tinyboards/backend
cargo run --release -- --run-migrations
```

### 5. Restart Services

```bash
sudo systemctl restart tinyboards-backend
sudo systemctl restart tinyboards-frontend
```

### 6. Verify

```bash
sudo systemctl status tinyboards-backend
sudo systemctl status tinyboards-frontend
curl -s http://localhost:8536/
```

## Database Migrations

TinyBoards uses numbered SQL migration files in the `migrations/` directory:

```
migrations/
├── 2026-03-03-000001_foundation/
│   ├── up.sql
│   └── down.sql
├── 2026-03-03-000002_core_tables/
│   ├── up.sql
│   └── down.sql
├── ...
└── 2026-03-03-000014_auth_sessions/
    ├── up.sql
    └── down.sql
```

Each migration has an `up.sql` (apply) and `down.sql` (revert). Migrations are applied in order and tracked in the database so they only run once.

### Checking Migration Status

```bash
# Docker
docker compose exec postgres psql -U tinyboards -d tinyboards \
  -c "SELECT * FROM __diesel_schema_migrations ORDER BY run_on DESC LIMIT 5;"

# From source
psql -U tinyboards -d tinyboards \
  -c "SELECT * FROM __diesel_schema_migrations ORDER BY run_on DESC LIMIT 5;"
```

## Rolling Back

If an upgrade causes issues:

### 1. Stop the Application

```bash
docker compose down
```

### 2. Restore from Pre-Upgrade Backup

```bash
docker compose up -d postgres
sleep 5

docker compose exec postgres psql -U tinyboards -c "DROP DATABASE IF EXISTS tinyboards;"
docker compose exec postgres psql -U tinyboards -c "CREATE DATABASE tinyboards OWNER tinyboards;"

docker compose exec -T postgres pg_restore \
  -U tinyboards -d tinyboards \
  --no-owner --clean --if-exists \
  < pre-upgrade-20260301.dump
```

### 3. Check Out the Previous Version

```bash
git checkout v1.2.3   # replace with the version you were on
docker compose build
docker compose up -d
```

### 4. Verify

```bash
docker compose ps
curl -s http://localhost:8536/
```

> Always take a backup before upgrading. Rolling back database migrations manually is risky and may result in data loss.
