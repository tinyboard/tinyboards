# Docker Compose Deployment

This guide walks through the `docker-compose.yml` that ships with TinyBoards.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Services Overview](#services-overview)
- [Step-by-Step Setup](#step-by-step-setup)
- [Service Details](#service-details)
- [Networks](#networks)
- [Volumes](#volumes)
- [Common Operations](#common-operations)

## Prerequisites

- Docker 20.10+ and Docker Compose v2
- A domain name with an A record pointed to your server
- Ports 80 and 443 available

## Services Overview

The compose file defines four services:

| Service    | Image                 | Internal Port | Purpose                  |
|------------|-----------------------|---------------|--------------------------|
| `postgres` | `postgres:16-alpine`  | 5432          | Database                 |
| `backend`  | Built from source     | 8536          | Rust API server          |
| `frontend` | Built from source     | 3000          | Nuxt 3 SSR server        |
| `nginx`    | `nginx:1.25-alpine`   | 80, 443       | Reverse proxy, SSL, media |

## Step-by-Step Setup

### 1. Clone and configure

```bash
git clone https://github.com/tinyboard/tinyboards-rewrite.git
cd tinyboards-rewrite
cp tinyboards.example.hjson tinyboards.hjson
```

### 2. Edit tinyboards.hjson

At minimum, set these values:

```hjson
{
  domain: boards.example.com

  database: {
    password: your_secure_password_here
  }

  security: {
    // Generate with: openssl rand -base64 48
    jwt_secret: your_jwt_secret_here
    // Generate with: openssl rand -hex 32
    salt_suffix: your_salt_here
  }

  admin: {
    username: admin
    password: your_admin_password
    email: "admin@example.com"
  }
}
```

Then generate the `.env` that docker-compose needs:

```bash
./configure.sh
```

See [env-vars.md](env-vars.md) for the complete configuration reference.

### 3. Set up SSL certificates

If using Let's Encrypt, see [ssl.md](ssl.md). For initial testing, you can generate self-signed certs:

```bash
mkdir -p certs
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout certs/privkey.pem \
  -out certs/fullchain.pem \
  -subj "/CN=localhost"
```

### 4. Start the stack

```bash
docker compose up -d
```

### 5. Check health

```bash
docker compose ps
docker compose logs -f
```

All services should show `healthy` status within 60 seconds.

## Service Details

### PostgreSQL

```yaml
postgres:
  image: postgres:16-alpine
  restart: unless-stopped
  environment:
    POSTGRES_USER: ${POSTGRES_USER}
    POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    POSTGRES_DB: ${POSTGRES_DB}
  volumes:
    - postgres_data:/var/lib/postgresql/data
    - ./config/postgresql.conf:/etc/postgresql/postgresql.conf:ro
  command: postgres -c config_file=/etc/postgresql/postgresql.conf
  ports:
    - "127.0.0.1:5432:5432"   # localhost only — not exposed to internet
```

Key points:
- Port bound to `127.0.0.1` only — never exposed externally.
- Custom `postgresql.conf` mounted for tuning. The production compose file includes inline tuning parameters for shared_buffers, WAL, autovacuum, etc.
- A healthcheck runs `pg_isready` every 10 seconds. Other services wait for this.
- Data persisted in the `postgres_data` volume.

### Backend

```yaml
backend:
  build:
    context: .
    dockerfile: backend.Dockerfile
  depends_on:
    postgres:
      condition: service_healthy
  environment:
    TB_CONFIG_LOCATION: /app/tinyboards.hjson
    RUST_LOG: info
    TLS_ENABLED: "false"
  volumes:
    - ./tinyboards.hjson:/app/tinyboards.hjson:ro
    - media_data:/app/media
  expose:
    - "8536"
```

Key points:
- Uses `expose` (not `ports`) — only accessible from within the Docker network.
- Waits for PostgreSQL to be healthy before starting.
- All configuration is read from `tinyboards.hjson`, mounted read-only into the container.
- The only env vars are `TB_CONFIG_LOCATION` (path to the config file), `RUST_LOG` (log level), and `TLS_ENABLED`.
- Media files stored in the `media_data` volume.
- Runs database migrations automatically on startup.
- Healthcheck at `http://localhost:8536/` returns "ok".

### Frontend

```yaml
frontend:
  build:
    context: .
    dockerfile: frontend.Dockerfile
  depends_on:
    backend:
      condition: service_healthy
  environment:
    TB_CONFIG_LOCATION: /app/tinyboards.hjson
  volumes:
    - ./tinyboards.hjson:/app/tinyboards.hjson:ro
```

Key points:
- The frontend reads `tinyboards.hjson` directly to discover the backend URL, domain, and other settings.
- Contacts the backend over the internal Docker network using the service name `backend`.
- Not exposed to the host — only nginx routes traffic to it.

### Nginx

```yaml
nginx:
  image: nginx:1.25-alpine
  ports:
    - "80:80"
    - "443:443"
  volumes:
    - ./config/nginx.conf:/etc/nginx/nginx.conf:ro
    - ./config/nginx_site.conf:/etc/nginx/conf.d/default.conf:ro
    - ./certs/fullchain.pem:/etc/nginx/certs/fullchain.pem:ro
    - ./certs/privkey.pem:/etc/nginx/certs/privkey.pem:ro
    - media_data:/app/media:ro
```

Key points:
- The only service with host-exposed ports.
- SSL certs mounted read-only.
- Media volume mounted read-only for direct file serving (bypasses backend for media requests).
- See [nginx.md](nginx.md) for the full configuration reference.

## Networks

```yaml
networks:
  internal:
    driver: bridge
    internal: true    # No internet access
  external:
    driver: bridge    # Internet access for nginx
```

- `internal` — Used by postgres, backend, frontend, and nginx. The `internal: true` flag blocks outbound internet access from these containers.
- `external` — Used by nginx only, allowing it to receive inbound connections.

## Volumes

| Volume          | Purpose                       | Backup Priority |
|-----------------|-------------------------------|-----------------|
| `postgres_data` | Database files                | Critical        |
| `media_data`    | Uploaded images, videos, etc. | High            |
| `nginx_logs`    | Nginx access/error logs       | Low             |

See [backup-restore.md](backup-restore.md) for backup procedures.

## Common Operations

### View logs

```bash
# All services
docker compose logs -f

# Single service
docker compose logs -f backend
```

### Restart a service

```bash
docker compose restart backend
```

### Rebuild after code changes

```bash
docker compose build backend frontend
docker compose up -d
```

### Stop everything

```bash
docker compose down
```

### Stop and remove volumes (destroys data)

```bash
docker compose down -v
```

### Run a database migration manually

```bash
docker compose exec backend /app/tinyboards --run-migrations
```

### Open a psql shell

```bash
docker compose exec postgres psql -U tinyboards -d tinyboards
```
