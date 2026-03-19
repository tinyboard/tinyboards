# Nginx Reverse Proxy

TinyBoards uses nginx as a reverse proxy to route requests, terminate SSL, and serve static media files.

## Table of Contents

- [How Routing Works](#how-routing-works)
- [HTTP Configuration](#http-configuration)
- [HTTPS Configuration](#https-configuration)
- [Key Directives](#key-directives)
- [Security Headers](#security-headers)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)

## How Routing Works

nginx routes requests based on path:

| Path Pattern | Destination | Purpose |
|-------------|-------------|---------|
| `/api/*` | `backend:8536` | GraphQL API and REST endpoints |
| `/media/*` | Local filesystem | Uploaded media (served directly by nginx) |
| Everything else | `frontend:3000` | Nuxt 3 SSR pages |

## HTTP Configuration

Use this for initial testing or if SSL is handled by an upstream load balancer.

```nginx
server {
    listen 80;
    server_name yourdomain.com;

    # Redirect to HTTPS in production — uncomment when SSL is ready
    # return 301 https://$server_name$request_uri;

    client_max_body_size 50M;

    # API requests → backend
    location /api/ {
        proxy_pass http://127.0.0.1:8536;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Media files → served directly from disk
    location /media/ {
        alias /app/media/;
        expires 30d;
        add_header Cache-Control "public, immutable";
        try_files $uri =404;
    }

    # Everything else → frontend
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## HTTPS Configuration

Production configuration with SSL termination.

```nginx
# Redirect HTTP → HTTPS
server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    # SSL certificates
    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;

    # SSL settings
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;
    ssl_session_tickets off;

    # OCSP stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    resolver 1.1.1.1 8.8.8.8 valid=300s;
    resolver_timeout 5s;

    # Upload size limit — must match MAX_FILE_SIZE_MB env var
    client_max_body_size 50M;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # API requests → backend
    location /api/ {
        proxy_pass http://127.0.0.1:8536;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts for GraphQL queries
        proxy_read_timeout 60s;
        proxy_send_timeout 60s;
    }

    # Media files → served directly from disk
    location /media/ {
        alias /app/media/;
        expires 30d;
        add_header Cache-Control "public, immutable";
        try_files $uri =404;
    }

    # Let's Encrypt challenge
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    # Everything else → frontend
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

> **Docker note:** When using docker-compose, replace `127.0.0.1:8536` with `backend:8536` and `127.0.0.1:3000` with `frontend:3000`. The media `alias` should be `/app/media/` (matching the volume mount).

## Key Directives

### client_max_body_size

Controls the maximum file upload size. This must match the `MAX_FILE_SIZE_MB` environment variable:

```nginx
client_max_body_size 50M;
```

### proxy_set_header

Essential headers for the backend to know the real client IP and protocol:

```nginx
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
proxy_set_header X-Forwarded-Proto $scheme;
```

### WebSocket support

The `Upgrade` and `Connection` headers are needed if you add WebSocket support in the future:

```nginx
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection "upgrade";
```

## Security Headers

The HTTPS config includes these security headers:

| Header | Value | Purpose |
|--------|-------|---------|
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | Force HTTPS for 1 year |
| `X-Content-Type-Options` | `nosniff` | Prevent MIME type sniffing |
| `X-Frame-Options` | `SAMEORIGIN` | Prevent clickjacking |
| `X-XSS-Protection` | `1; mode=block` | Enable browser XSS filter |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Limit referrer information |

## Performance Tuning

Add to the `http` block in `/etc/nginx/nginx.conf`:

```nginx
http {
    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 4;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;

    # Connection settings
    keepalive_timeout 65;
    keepalive_requests 100;

    # Buffer sizes
    proxy_buffer_size 128k;
    proxy_buffers 4 256k;
    proxy_busy_buffers_size 256k;
}
```

## Troubleshooting

### 502 Bad Gateway

The backend or frontend is not running or not reachable.

```bash
# Check if services are running
docker compose ps

# Check backend health
curl http://localhost:8536/

# Check frontend health
curl http://localhost:3000/
```

### 413 Request Entity Too Large

File upload exceeds `client_max_body_size`. Increase it in the nginx config and update `MAX_FILE_SIZE_MB` to match.

### Mixed Content Warnings

Ensure `NUXT_PUBLIC_USE_HTTPS=true` and `NUXT_PUBLIC_SITE_URL` uses `https://`.

### SSL Certificate Not Found

Check that the paths in your `.env` (`SSL_CERT_PATH`, `SSL_KEY_PATH`) point to valid certificate files. For Let's Encrypt, see [ssl.md](ssl.md).
