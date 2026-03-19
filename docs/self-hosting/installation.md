# From-Source Installation

This guide covers building and running TinyBoards directly on Ubuntu 22.04 / Debian 12 without Docker.

## Table of Contents

- [System Dependencies](#system-dependencies)
- [PostgreSQL Setup](#postgresql-setup)
- [Backend Build](#backend-build)
- [Frontend Build](#frontend-build)
- [Nginx Setup](#nginx-setup)
- [Systemd Services](#systemd-services)
- [First Run](#first-run)

## System Dependencies

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Build essentials
sudo apt install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  libpq-dev \
  curl \
  git \
  nginx \
  certbot \
  python3-certbot-nginx

# Install Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Install Node.js 20 via NodeSource
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# Install pnpm (used by the frontend)
npm install -g pnpm
```

## PostgreSQL Setup

### Install PostgreSQL 16

```bash
# Add PostgreSQL APT repository
sudo sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add -
sudo apt update
sudo apt install -y postgresql-16

# Required extensions
sudo apt install -y postgresql-16-pg-trgm
```

### Create Database and User

```bash
sudo -u postgres psql <<SQL
CREATE USER tinyboards WITH PASSWORD 'your_secure_password_here';
CREATE DATABASE tinyboards OWNER tinyboards;
GRANT ALL PRIVILEGES ON DATABASE tinyboards TO tinyboards;
\c tinyboards
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
SQL
```

### Tuning (Optional)

Edit `/etc/postgresql/16/main/postgresql.conf`:

```ini
# Connection settings
max_connections = 100

# Memory (adjust for your server)
shared_buffers = 128MB
effective_cache_size = 512MB
work_mem = 4MB
maintenance_work_mem = 64MB

# WAL
wal_buffers = 32MB
checkpoint_completion_target = 0.9

# Query planner
random_page_cost = 1.1
effective_io_concurrency = 200
```

Restart PostgreSQL after changes:

```bash
sudo systemctl restart postgresql
```

## Backend Build

### Clone and Build

```bash
git clone https://github.com/tinyboard/tinyboards-rewrite.git /opt/tinyboards
cd /opt/tinyboards

# Build the backend in release mode
cd backend
cargo build --release
```

The binary will be at `backend/target/release/tinyboards_server` (the exact name depends on the crate configuration).

### Configure

Copy the configuration template:

```bash
cp tinyboards.example.hjson tinyboards.hjson
```

Edit `tinyboards.hjson` and set the following values:

```hjson
database: {
  host: "localhost"
  password: "your_secure_password_here"
}
hostname: "yourdomain.com"
salt_suffix: "your_random_salt_here"
media: {
  media_path: "/opt/tinyboards/media"
}
storage: {
  fs: {
    root: "/opt/tinyboards/media"
  }
}
```

Set the environment variable so the backend can find the config file:

```bash
export TB_CONFIG_LOCATION=/opt/tinyboards/tinyboards.hjson
```

### Run Migrations

Migrations run automatically when the backend starts. To verify the database connection:

```bash
cd /opt/tinyboards
source .env
cargo run --release -- --run-migrations
```

### Create Media Directory

```bash
sudo mkdir -p /opt/tinyboards/media
sudo chown -R $USER:$USER /opt/tinyboards/media
```

## Frontend Build

```bash
cd /opt/tinyboards/frontend
pnpm install
pnpm build
```

### Configure Frontend Environment

The frontend now reads from the same `tinyboards.hjson` configuration file as the backend. No separate frontend `.env` is needed. Just ensure `TB_CONFIG_LOCATION` is set:

```bash
export TB_CONFIG_LOCATION=/opt/tinyboards/tinyboards.hjson
```

For bare-metal installations where the frontend connects to the backend on localhost, also set `frontend.internal_api_host` in `tinyboards.hjson`:

```hjson
frontend: {
  internal_api_host: "http://localhost:8536"
}
```

## Nginx Setup

See [nginx.md](nginx.md) for the full configuration. A minimal setup:

```bash
sudo cp /opt/tinyboards/config/nginx_site.conf /etc/nginx/sites-available/tinyboards
sudo ln -s /etc/nginx/sites-available/tinyboards /etc/nginx/sites-enabled/
sudo rm /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl reload nginx
```

## Systemd Services

### Backend Service

Create `/etc/systemd/system/tinyboards-backend.service`:

```ini
[Unit]
Description=TinyBoards Backend
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=tinyboards
Group=tinyboards
WorkingDirectory=/opt/tinyboards/backend
ExecStart=/opt/tinyboards/backend/target/release/tinyboards_server
Environment=TB_CONFIG_LOCATION=/opt/tinyboards/tinyboards.hjson
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/tinyboards/media

[Install]
WantedBy=multi-user.target
```

### Frontend Service

Create `/etc/systemd/system/tinyboards-frontend.service`:

```ini
[Unit]
Description=TinyBoards Frontend
After=network.target tinyboards-backend.service
Wants=tinyboards-backend.service

[Service]
Type=simple
User=tinyboards
Group=tinyboards
WorkingDirectory=/opt/tinyboards/frontend
ExecStart=/usr/bin/node .output/server/index.mjs
Environment=NUXT_HOST=127.0.0.1
Environment=NUXT_PORT=3000
Environment=TB_CONFIG_LOCATION=/opt/tinyboards/tinyboards.hjson
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true

[Install]
WantedBy=multi-user.target
```

### Enable and Start

```bash
# Create a system user for tinyboards
sudo useradd -r -s /usr/sbin/nologin -d /opt/tinyboards tinyboards
sudo chown -R tinyboards:tinyboards /opt/tinyboards

# Enable and start services
sudo systemctl daemon-reload
sudo systemctl enable tinyboards-backend tinyboards-frontend
sudo systemctl start tinyboards-backend
sudo systemctl start tinyboards-frontend

# Check status
sudo systemctl status tinyboards-backend
sudo systemctl status tinyboards-frontend
```

## First Run

1. Start all services (PostgreSQL, backend, frontend, nginx).
2. Open `https://yourdomain.com` in your browser.
3. The initial setup wizard will prompt you to configure the site name, registration mode, and admin account (if not already set via the `setup` section in `tinyboards.hjson`).
4. After setup, navigate to `/admin` to configure site settings.

### Verify Services

```bash
# Backend health check
curl -s http://localhost:8536/
# Should return: ok

# Frontend health check
curl -s http://localhost:3000/
# Should return HTML

# Full stack through nginx
curl -s https://yourdomain.com/
```
