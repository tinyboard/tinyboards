# Self-Hosting tinyboards

A step-by-step guide for deploying tinyboards on your own server.

---

## Prerequisites

### Hardware

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 1 vCPU | 2 vCPU |
| RAM | 1 GB | 2 GB |
| Disk | 10 GB | 20 GB+ (depends on media uploads) |

Tested on: Hetzner CX11/CX21, DigitalOcean Basic $6-12/mo, Oracle Cloud Free Tier.

### Software (Docker method — recommended)

- Docker Engine 24+
- Docker Compose v2 (the `docker compose` plugin, not standalone `docker-compose`)

### Software (manual method)

- Rust (edition 2021 toolchain — install via [rustup](https://rustup.rs/))
- Node.js 20+
- PostgreSQL 16+
- nginx or Caddy (reverse proxy)
- git

---

## Quick Start (Docker — recommended)

Pre-built images are published to Docker Hub after every release. Your server
only needs Docker — no git clone, no compilers, no Node.js.

```bash
# 1. Create a directory for your instance
mkdir tinyboards && cd tinyboards

# 2. Download the compose file and config template
curl -LO https://raw.githubusercontent.com/tinyboard/tinyboards/main/docker-compose.yml
curl -LO https://raw.githubusercontent.com/tinyboard/tinyboards/main/tinyboards.example.hjson
cp tinyboards.example.hjson tinyboards.hjson
```

You will also need the nginx config, postgres tuning file, and the configuration script:

```bash
mkdir -p nginx deploy/postgres
curl -L -o nginx/default.conf \
  https://raw.githubusercontent.com/tinyboard/tinyboards/main/nginx/default.conf
curl -L -o deploy/postgres/postgresql-1gb.conf \
  https://raw.githubusercontent.com/tinyboard/tinyboards/main/deploy/postgres/postgresql-1gb.conf
curl -LO https://raw.githubusercontent.com/tinyboard/tinyboards/main/configure.sh
chmod +x configure.sh
```

**Edit `tinyboards.hjson` before continuing.** At minimum, you must set these fields:

```hjson
# REQUIRED — your domain (no protocol)
hostname: forum.example.com

# REQUIRED — strong database password
database: {
  password: "<generate with: openssl rand -base64 32>"
}

# REQUIRED — password hashing salt
salt_suffix: "<generate with: openssl rand -hex 32>"

# REQUIRED — initial admin account (change the password!)
setup: {
  admin_username: admin
  admin_password: "<your strong admin password>"
}
```

Optionally pin a specific release instead of `latest`:

```hjson
# In tinyboards.hjson — defaults to "latest" if omitted
docker: {
  image_tag: "v1.0.0"
}
```

Then generate the `.env` file and start the stack:

```bash
# 3. Generate .env from tinyboards.hjson
./configure.sh

# 4. Pull images and start all services
docker compose pull
docker compose up -d

# 5. Wait for health checks to pass (about 30-60 seconds)
docker compose ps

# 6. Open your browser
# http://localhost (or http://your-domain if DNS is configured)
```

All four services (postgres, backend, frontend, nginx) should show as "healthy". If not, check the [Troubleshooting](#troubleshooting) section.

---

## Manual Installation (without Docker)

### 1. Install dependencies

```bash
# PostgreSQL 16
sudo apt install postgresql-16 postgresql-client-16

# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Node.js 20 (via NodeSource)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install nodejs

# nginx
sudo apt install nginx
```

### 2. Create the tinyboards user and directory

```bash
sudo useradd -r -s /usr/sbin/nologin -d /opt/tinyboards tinyboards
sudo mkdir -p /opt/tinyboards/media
sudo chown -R tinyboards:tinyboards /opt/tinyboards
```

### 3. Build the backend

```bash
cd /opt/tinyboards
sudo -u tinyboards git clone https://github.com/tinyboard/tinyboards.git .
cd backend
cargo build --release

# The binary is at: target/release/tinyboards_server
sudo cp target/release/tinyboards_server /opt/tinyboards/tinyboards-server
```

### 4. Build the frontend

```bash
cd /opt/tinyboards/frontend
npm ci
npx nuxi build

# The output is at: .output/server/index.mjs
```

### 5. Set up the database

```bash
# Create database and user
sudo -u postgres psql <<SQL
CREATE USER tinyboards WITH PASSWORD 'your_strong_password';
CREATE DATABASE tinyboards OWNER tinyboards;
\c tinyboards
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
SQL
```

Migrations run automatically when the backend starts for the first time.

### 6. Configure the application

```bash
sudo cp /opt/tinyboards/tinyboards.example.hjson /opt/tinyboards/tinyboards.hjson
sudo chown tinyboards:tinyboards /opt/tinyboards/tinyboards.hjson
sudo chmod 600 /opt/tinyboards/tinyboards.hjson
```

Edit `/opt/tinyboards/tinyboards.hjson` with the required values (see Quick Start above). For bare-metal installs, also update these settings:

```hjson
# Point to your local PostgreSQL instead of the Docker service name
database: {
  host: "localhost"
}

# Media path (must be writable by the tinyboards user)
media: {
  media_path: "/opt/tinyboards/media"
}
```

The backend reads `tinyboards.hjson` directly via the `TB_CONFIG_LOCATION` environment variable. Set this in the systemd service file or your shell:

```bash
TB_CONFIG_LOCATION=/opt/tinyboards/tinyboards.hjson
```

The frontend also reads `tinyboards.hjson` for its configuration.

### 7. Install systemd service files

```bash
sudo cp deploy/systemd/tinyboards-backend.service /etc/systemd/system/
sudo cp deploy/systemd/tinyboards-frontend.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now tinyboards-backend tinyboards-frontend
```

### 8. Configure nginx

```bash
sudo cp nginx/default.conf /etc/nginx/sites-available/tinyboards
sudo ln -s /etc/nginx/sites-available/tinyboards /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t && sudo systemctl reload nginx
```

---

## Configuration Reference

All configuration is in `tinyboards.hjson`. See the comments in `tinyboards.example.hjson` for documentation of every option.

The `.env` file is auto-generated by `configure.sh` and only contains the few values needed by docker-compose (PostgreSQL container credentials, domain, logging). **Do not edit `.env` directly** — edit `tinyboards.hjson` and re-run `./configure.sh` instead.

---

## Storage Configuration (local, S3, Wasabi, Backblaze B2)

Storage is configured in the `storage` section of `tinyboards.hjson`.

### Local filesystem (default)

Files are stored at the path specified by `media.media_path` in `tinyboards.hjson`. In Docker, this is a named volume (`media_data`) mounted at `/app/media`. For manual installs, set it to a writable directory (e.g., `/opt/tinyboards/media`).

No additional configuration is needed.

### AWS S3

```hjson
storage: {
  backend: "s3"
  s3: {
    bucket: "my-tinyboards-media"
    region: "us-east-1"
    access_key_id: "AKIAIOSFODNN7EXAMPLE"
    secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
  }
}
```

### Wasabi

```hjson
storage: {
  backend: "s3"
  s3: {
    bucket: "my-tinyboards-media"
    region: "us-west-1"
    access_key_id: "your-wasabi-access-key"
    secret_access_key: "your-wasabi-secret-key"
    endpoint: "https://s3.us-west-1.wasabisys.com"
  }
}
```

> **Important:** The `endpoint` must match the bucket's region. A bucket created in `us-west-1` must use `https://s3.us-west-1.wasabisys.com`. This is the most common configuration mistake with Wasabi — using the wrong regional endpoint will result in 403 errors.

### Backblaze B2

```hjson
storage: {
  backend: "s3"
  s3: {
    bucket: "my-tinyboards-media"
    region: "us-west-002"
    access_key_id: "your-b2-application-key-id"
    secret_access_key: "your-b2-application-key"
    endpoint: "https://s3.us-west-002.backblazeb2.com"
  }
}
```

### Cloudflare R2

```hjson
storage: {
  backend: "s3"
  s3: {
    bucket: "my-tinyboards-media"
    region: "auto"
    access_key_id: "your-r2-access-key-id"
    secret_access_key: "your-r2-secret-access-key"
    endpoint: "https://<account-id>.r2.cloudflarestorage.com"
  }
}
```

---

## Reverse Proxy (nginx, Caddy)

### nginx (HTTP)

The included `nginx/default.conf` handles routing between the frontend and backend:

- `/api/v2/*` → backend (Rust, port 8536)
- `/api/*` → frontend BFF (Nuxt server routes, port 3000)
- `/media/*` → backend (media serving)
- `/` → frontend (Nuxt SSR, port 3000)

For Docker deployments, nginx is included as a service. For manual installs:

```bash
sudo cp nginx/default.conf /etc/nginx/sites-available/tinyboards
sudo ln -s /etc/nginx/sites-available/tinyboards /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

### nginx (HTTPS)

Use `nginx/ssl.conf` instead. See the [SSL/TLS](#ssltls) section.

### Caddy (automatic HTTPS)

Caddy handles SSL automatically. Create a `Caddyfile`:

```
forum.example.com {
    # Backend API
    handle /api/v2/* {
        reverse_proxy backend:8536
    }

    # Media files
    handle /media/* {
        reverse_proxy backend:8536
    }

    # Frontend BFF routes
    handle /api/* {
        reverse_proxy frontend:3000
    }

    # Frontend (Nuxt SSR)
    handle {
        reverse_proxy frontend:3000
    }
}
```

If using Caddy outside Docker, replace `backend:8536` with `127.0.0.1:8536` and `frontend:3000` with `127.0.0.1:3000`.

---

## SSL/TLS

### Option 1: Let's Encrypt with certbot (recommended)

```bash
# Install certbot
sudo apt install certbot

# Obtain certificate (nginx must be running on port 80)
sudo certbot certonly --webroot -w /var/www/certbot -d forum.example.com

# Copy SSL config
sed 's|/live/DOMAIN/|/live/forum.example.com/|g' nginx/ssl.conf > nginx/ssl-active.conf
```

For Docker, update `docker-compose.yml` to mount the SSL config and certificates:

```yaml
nginx:
  volumes:
    - ./nginx/ssl-active.conf:/etc/nginx/conf.d/default.conf:ro
    - /etc/letsencrypt:/etc/letsencrypt:ro
    - /var/www/certbot:/var/www/certbot:ro
```

Then update `tinyboards.hjson`:

```hjson
# In tinyboards.hjson:
tls_enabled: true
frontend: {
  use_https: true
}
```

Then regenerate the `.env` and restart: `./configure.sh && docker compose up -d`

### Option 2: Caddy (automatic)

Caddy handles certificate provisioning and renewal automatically. No additional configuration needed beyond the Caddyfile shown above.

### Option 3: Automated setup script

```bash
sudo ./deploy/scripts/setup-ssl.sh forum.example.com admin@example.com
```

This installs certbot, obtains a certificate, and generates `nginx/ssl-active.conf`.

---

## Database Backups

### Creating backups

```bash
# Using the included script
./deploy/scripts/backup.sh

# Or with an explicit database URL
./deploy/scripts/backup.sh "postgresql://tinyboards:password@localhost:5432/tinyboards"
```

Backups are saved to `./backups/` as compressed pg_dump files (e.g., `tinyboards_backup_2026-03-11_03-00.dump.gz`).

### Automating with cron

```cron
# Run backup daily at 3am, clean up old backups at 3:05am
0 3 * * * /opt/tinyboards/deploy/scripts/backup.sh >> /var/log/tinyboards-backup.log 2>&1
5 3 * * * /opt/tinyboards/deploy/scripts/cleanup-backups.sh >> /var/log/tinyboards-backup.log 2>&1
```

Set `BACKUP_RETENTION_DAYS` to control how long backups are kept (default: 7 days).

### Restoring from backup

```bash
# Interactive (asks for confirmation)
./deploy/scripts/restore.sh backups/tinyboards_backup_2026-03-11_03-00.dump.gz

# Non-interactive (for scripts)
./deploy/scripts/restore.sh --yes backups/tinyboards_backup_2026-03-11_03-00.dump.gz
```

> **Warning:** Restoring replaces the entire database. Back up your current data first.

---

## PostgreSQL Tuning

Three pre-tuned configuration files are provided in `deploy/postgres/`:

| File | Target | RAM | Connections |
|------|--------|-----|-------------|
| `postgresql-1gb.conf` | 1 GB VPS | 256 MB shared_buffers | 25 |
| `postgresql-2gb.conf` | 2 GB VPS | 512 MB shared_buffers | 50 |
| `postgresql-4gb.conf` | 4 GB VPS | 1 GB shared_buffers | 100 |

### Docker

The `docker-compose.yml` mounts the 1 GB config by default. To use a different tier:

```yaml
postgres:
  volumes:
    - ./deploy/postgres/postgresql-2gb.conf:/etc/postgresql/postgresql.conf:ro
```

Then restart: `docker compose restart postgres`

### Manual install

Append the config values to your `postgresql.conf`, or use `include`:

```
# At the end of /etc/postgresql/16/main/postgresql.conf
include '/opt/tinyboards/deploy/postgres/postgresql-2gb.conf'
```

Then restart PostgreSQL: `sudo systemctl restart postgresql`

Each setting has a comment explaining what it does and why the value was chosen. Adjust `random_page_cost` and `effective_io_concurrency` if running on spinning disk instead of SSD.

---

## Upgrading

### Docker

```bash
# 1. Pull the latest images
docker compose pull

# 2. Recreate containers with the new images
docker compose up -d

# 3. Check that all services are healthy
docker compose ps
```

To upgrade to a specific version, set `docker.image_tag` in `tinyboards.hjson` and re-run `./configure.sh`, then repeat the steps above.

Database migrations run automatically when the backend starts. There is no separate migration step.

For manual installs:

```bash
# Rebuild backend
cd backend && cargo build --release
sudo cp target/release/tinyboards_server /opt/tinyboards/tinyboards-server

# Rebuild frontend
cd frontend && npm ci && npx nuxi build

# Restart services
sudo systemctl restart tinyboards-backend tinyboards-frontend
```

---

## Troubleshooting

### Backend fails to start: "could not establish connection"

The backend cannot reach PostgreSQL. In Docker, this usually means the database isn't ready yet.

**Fix:** Check that the postgres service is healthy:

```bash
docker compose ps postgres
docker compose logs postgres
```

The `docker-compose.yml` uses `depends_on: condition: service_healthy` so this should resolve automatically. If it persists, check that `database.password` in `tinyboards.hjson` matches `POSTGRES_PASSWORD` in the generated `.env` (re-run `./configure.sh`).

### 502 Bad Gateway

Nginx is running but can't reach the backend or frontend.

**Diagnose:**

```bash
# Check which service is down
docker compose ps

# Check service logs
docker compose logs backend
docker compose logs frontend
```

Common causes:
- Backend crashed on startup (check logs for panics)
- Frontend build failed (check logs for Node.js errors)
- Wrong internal URLs in environment variables

### Login not working / cookies not being set

This is almost always a domain mismatch. The `hostname` field in `tinyboards.hjson` must match the domain in your browser's address bar exactly.

**Fix:**

```hjson
# In tinyboards.hjson:
hostname: forum.example.com    # Must match your actual domain
```

If using HTTPS, also set:

```hjson
# In tinyboards.hjson:
frontend: {
  use_https: true
}
```

Then re-run `./configure.sh && docker compose up -d`.

Cookies are set with `httpOnly` and `Secure` flags. If `use_https` is `true` but you're accessing via HTTP, cookies will not be set.

### Images not loading

**Local storage:** Check that `media.media_path` in `tinyboards.hjson` is correct and the directory is writable by the tinyboards user:

```bash
ls -la /app/media/          # Docker
ls -la /opt/tinyboards/media/  # Manual install
```

**S3 storage:** Verify your S3 credentials and bucket permissions. Check backend logs for storage errors:

```bash
docker compose logs backend | grep -i "storage\|s3\|upload"
```

### "relation does not exist" errors

Migrations didn't run. The backend runs migrations automatically on startup, but if it crashed before completing them:

```bash
# Check backend logs for migration errors
docker compose logs backend | grep -i "migration"

# Restart the backend to retry migrations
docker compose restart backend
```

For manual installs, migrations run when the backend binary starts. If it fails, check that the `DATABASE_URL` is correct and the database exists.
