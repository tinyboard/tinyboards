#!/bin/bash
# tinyboards SSL setup script
#
# Installs certbot, obtains a Let's Encrypt certificate via webroot,
# and configures nginx to use the SSL config.
#
# Prerequisites:
#   - nginx must be running and serving on port 80
#   - DNS must already point to this server
#   - Port 80 must be accessible from the internet
#
# Usage:
#   ./setup-ssl.sh yourdomain.com your@email.com

set -euo pipefail

DOMAIN="${1:-}"
EMAIL="${2:-}"

if [ -z "$DOMAIN" ] || [ -z "$EMAIL" ]; then
    echo "Usage: $0 <domain> <email>"
    echo "Example: $0 forum.example.com admin@example.com"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "[$(date)] Setting up SSL for $DOMAIN..."

# Install certbot if not present
if ! command -v certbot &> /dev/null; then
    echo "[$(date)] Installing certbot..."
    if command -v apt-get &> /dev/null; then
        apt-get update
        apt-get install -y certbot
    elif command -v dnf &> /dev/null; then
        dnf install -y certbot
    elif command -v yum &> /dev/null; then
        yum install -y certbot
    else
        echo "ERROR: Could not detect package manager. Install certbot manually."
        exit 1
    fi
fi

# Create webroot directory for ACME challenges
WEBROOT="/var/www/certbot"
mkdir -p "$WEBROOT"

# Obtain certificate using webroot validation
echo "[$(date)] Requesting certificate from Let's Encrypt..."
certbot certonly \
    --webroot \
    --webroot-path "$WEBROOT" \
    --domain "$DOMAIN" \
    --email "$EMAIL" \
    --agree-tos \
    --non-interactive

# Update ssl.conf with the actual domain
echo "[$(date)] Configuring nginx SSL..."
SSL_CONF="$PROJECT_DIR/nginx/ssl.conf"

if [ ! -f "$SSL_CONF" ]; then
    echo "ERROR: $SSL_CONF not found."
    exit 1
fi

# Replace DOMAIN placeholder with actual domain in a copy
sed "s|/live/DOMAIN/|/live/${DOMAIN}/|g" "$SSL_CONF" > "$PROJECT_DIR/nginx/ssl-active.conf"

echo "[$(date)] SSL configuration written to nginx/ssl-active.conf"
echo ""
echo "Next steps:"
echo "  1. Update docker-compose.yml to mount the SSL config:"
echo "       - ./nginx/ssl-active.conf:/etc/nginx/conf.d/default.conf:ro"
echo "  2. Mount the certbot webroot and certificates:"
echo "       - /var/www/certbot:/var/www/certbot:ro"
echo "       - /etc/letsencrypt:/etc/letsencrypt:ro"
echo "  3. Restart nginx:"
echo "       docker compose restart nginx"
echo ""
echo "Certificate renewal runs automatically via certbot's systemd timer."
echo "To manually renew: certbot renew && docker compose restart nginx"
