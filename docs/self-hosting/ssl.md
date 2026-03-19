# SSL / TLS Setup

HTTPS is required for production deployments. This guide covers two approaches: Let's Encrypt with certbot (recommended) and Caddy with automatic SSL.

## Table of Contents

- [Option 1: Let's Encrypt with Certbot](#option-1-lets-encrypt-with-certbot)
- [Option 2: Caddy (Automatic SSL)](#option-2-caddy-automatic-ssl)
- [Verifying SSL](#verifying-ssl)

## Option 1: Let's Encrypt with Certbot

### Prerequisites

- Domain with DNS A record pointing to your server
- Ports 80 and 443 open in your firewall
- nginx installed and running

### Install Certbot

```bash
sudo apt install -y certbot python3-certbot-nginx
```

### Obtain Certificate

```bash
sudo certbot --nginx -d yourdomain.com --non-interactive --agree-tos -m admin@yourdomain.com
```

Certbot will:
1. Verify domain ownership via HTTP challenge
2. Obtain the certificate
3. Modify your nginx config to use it
4. Set up automatic renewal

### Docker Deployments

For Docker-based deployments, obtain the certificate on the host before starting the stack:

```bash
# Stop nginx if running (certbot needs port 80)
sudo systemctl stop nginx

# Obtain certificate in standalone mode
sudo certbot certonly --standalone -d yourdomain.com --non-interactive --agree-tos -m admin@yourdomain.com

# Update tinyboards.hjson to enable TLS
# Edit tinyboards.hjson and set:
#   tls_enabled: true
#   frontend: {
#     use_https: true
#   }

# Regenerate .env and restart
./configure.sh && docker compose up -d
```

Then update `docker-compose.yml` to mount the Let's Encrypt directory:

```yaml
nginx:
  volumes:
    - /etc/letsencrypt:/etc/letsencrypt:ro
    # ... other volumes
```

### Automatic Renewal

Certbot installs a systemd timer that renews certificates automatically. Verify it's active:

```bash
sudo systemctl status certbot.timer
```

To test renewal:

```bash
sudo certbot renew --dry-run
```

### Renewal with Docker

When using Docker, the nginx container needs to reload after certificate renewal. Create a deploy hook:

```bash
sudo mkdir -p /etc/letsencrypt/renewal-hooks/deploy
```

Create `/etc/letsencrypt/renewal-hooks/deploy/reload-nginx.sh`:

```bash
#!/bin/bash
cd /path/to/tinyboards-rewrite
docker compose exec nginx nginx -s reload
```

```bash
sudo chmod +x /etc/letsencrypt/renewal-hooks/deploy/reload-nginx.sh
```

### Manual Renewal

```bash
sudo certbot renew
sudo systemctl reload nginx
```

## Option 2: Caddy (Automatic SSL)

Caddy provides automatic SSL with zero configuration — it obtains and renews certificates from Let's Encrypt automatically.

### Install Caddy

```bash
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

### Caddyfile

Create `/etc/caddy/Caddyfile`:

```
yourdomain.com {
    # API → backend
    handle /api/* {
        reverse_proxy localhost:8536
    }

    # Media files
    handle /media/* {
        root * /app/media
        file_server
    }

    # Everything else → frontend
    handle {
        reverse_proxy localhost:3000
    }

    # Upload size limit
    request_body {
        max_size 50MB
    }

    # Security headers
    header {
        X-Content-Type-Options "nosniff"
        X-Frame-Options "SAMEORIGIN"
        Referrer-Policy "strict-origin-when-cross-origin"
    }
}
```

### Replace nginx with Caddy in Docker

If replacing nginx in docker-compose.yml, remove the nginx service and add:

```yaml
caddy:
  image: caddy:2-alpine
  restart: unless-stopped
  ports:
    - "80:80"
    - "443:443"
  volumes:
    - ./Caddyfile:/etc/caddy/Caddyfile:ro
    - caddy_data:/data
    - caddy_config:/config
    - media_data:/app/media:ro
  networks:
    - internal
    - external
```

Add to volumes:

```yaml
volumes:
  caddy_data:
  caddy_config:
```

### Start Caddy

```bash
sudo systemctl enable caddy
sudo systemctl start caddy
```

Caddy will automatically obtain and renew SSL certificates. No cron jobs or renewal hooks needed.

## Verifying SSL

After setup, verify your SSL configuration:

```bash
# Check certificate
curl -vI https://yourdomain.com 2>&1 | grep -E "subject:|expire date:|issuer:"

# Check SSL grade (external)
# Visit: https://www.ssllabs.com/ssltest/analyze.html?d=yourdomain.com
```

Update `tinyboards.hjson` to enable HTTPS in the application:

```hjson
tls_enabled: true
frontend: {
  use_https: true
}
```

Then regenerate the environment and restart:

```bash
./configure.sh && docker compose up -d
```
