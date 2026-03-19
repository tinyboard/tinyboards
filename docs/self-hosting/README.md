# Self-Hosting TinyBoards

TinyBoards is a self-hosted social media platform. This guide covers everything you need to deploy and maintain your own instance.

## Table of Contents

- [Requirements](#requirements)
- [Deployment Options](#deployment-options)
- [Quick Start (Docker)](#quick-start-docker)
- [Guides](#guides)

## Requirements

### Minimum Hardware

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU      | 1 vCPU  | 2 vCPU      |
| RAM      | 1 GB    | 2 GB        |
| Disk     | 10 GB   | 20 GB+      |

Disk usage scales with uploaded media. If you use S3-compatible object storage for media, the local disk requirement drops significantly.

### Software

- **Docker** 20.10+ and **Docker Compose** v2 (recommended deployment method)
- Or: **Rust** 1.75+, **Node.js** 20+, **PostgreSQL** 15+, **nginx** (from-source deployment)
- A registered domain name with DNS pointed to your server
- (Optional) SMTP credentials for email verification and password resets

### Supported Operating Systems

Docker deployment works on any Linux distribution. The from-source installation guide targets Ubuntu 22.04 / Debian 12, but any modern Linux will work with equivalent package names.

## Deployment Options

| Method | Difficulty | Best For |
|--------|-----------|----------|
| [Docker Compose](docker.md) | Easy | Most deployments |
| [From Source](installation.md) | Intermediate | Development, custom builds |

Docker Compose is the recommended approach for production. It handles PostgreSQL, the backend, frontend, and nginx in a single command.

## Quick Start (Docker)

```bash
# Clone the repository
git clone https://github.com/tinyboard/tinyboards-rewrite.git
cd tinyboards-rewrite

# Create your environment file
cp .env.example .env

# Generate secrets
sed -i "s/changeme_generate_with_openssl_rand_base64_48/$(openssl rand -base64 48)/" .env
sed -i "s/changeme_generate_with_openssl_rand_hex_32/$(openssl rand -hex 32)/" .env

# Set your domain
sed -i "s/DOMAIN=example.com/DOMAIN=yourdomain.com/" .env

# Set a strong admin password
sed -i "s/ADMIN_PASSWORD=changeme_strong_admin_password/ADMIN_PASSWORD=$(openssl rand -base64 24)/" .env

# Start the stack
docker compose up -d
```

After the stack starts, visit `https://yourdomain.com` to complete initial setup.

## Guides

| Guide | Description |
|-------|-------------|
| [Docker Compose Deployment](docker.md) | Full walkthrough of the docker-compose.yml |
| [From-Source Installation](installation.md) | Build and run on Ubuntu/Debian |
| [Environment Variables](env-vars.md) | Every configuration variable explained |
| [Nginx Reverse Proxy](nginx.md) | HTTP and HTTPS reverse proxy configs |
| [SSL / TLS](ssl.md) | Let's Encrypt with certbot; Caddy alternative |
| [Backup & Restore](backup-restore.md) | pg_dump cron template, restore procedure |
| [Upgrades](upgrades.md) | Migration procedure between versions |

## Architecture Overview

```
                    ┌─────────┐
  Internet ────────►│  nginx  │
                    │ :80/443 │
                    └────┬────┘
                         │
              ┌──────────┼──────────┐
              ▼                     ▼
        ┌──────────┐         ┌──────────┐
        │ frontend │         │ backend  │
        │  Nuxt 3  │────────►│ Actix-web│
        │  :3000   │ GraphQL │  :8536   │
        └──────────┘         └────┬─────┘
                                  │
                           ┌──────▼──────┐
                           │ PostgreSQL  │
                           │   :5432     │
                           └─────────────┘
```

- **nginx** terminates SSL and routes requests: `/api/*` and `/media/*` go to the backend; everything else goes to the frontend.
- **frontend** (Nuxt 3) renders pages and makes GraphQL requests to the backend.
- **backend** (Rust + Actix-web) serves the GraphQL API at `POST /api/v2/graphql` and media files at `GET /media/{filename}`.
- **PostgreSQL** stores all application data.

All services communicate over internal Docker networks. Only nginx exposes ports to the host.
