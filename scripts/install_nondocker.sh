#!/usr/bin/env bash
set -euo pipefail

# Non-Docker installation script for tinyboards.
# Installs PostgreSQL 16, Rust toolchain, Node 20, builds both applications,
# installs systemd units, configures nginx, and runs database migrations.
#
# Tested on: Ubuntu 22.04 / 24.04, Debian 12
# Must be run as root or with sudo.
#
# Usage: sudo ./install_nondocker.sh

INSTALL_DIR="/opt/tinyboards"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

# Ensure running as root
if [ "$(id -u)" -ne 0 ]; then
    error "This script must be run as root (use sudo)."
    exit 1
fi

echo "============================================"
echo "  TinyBoards — Non-Docker Installation"
echo "============================================"
echo ""
echo "This will install and configure:"
echo "  - PostgreSQL 16"
echo "  - Rust toolchain (stable)"
echo "  - Node.js 20"
echo "  - nginx"
echo "  - tinyboards backend + frontend"
echo ""
echo "Install directory: ${INSTALL_DIR}"
echo ""
read -rp "Continue? [y/N] " CONFIRM
if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
    echo "Installation cancelled."
    exit 0
fi

# -------------------------------------------------------
# Step 1: System packages
# -------------------------------------------------------
info "Updating system packages..."
apt-get update
apt-get install -y --no-install-recommends \
    curl \
    wget \
    gnupg2 \
    lsb-release \
    ca-certificates \
    build-essential \
    pkg-config \
    libssl-dev \
    libpq-dev \
    git

# -------------------------------------------------------
# Step 2: PostgreSQL 16
# -------------------------------------------------------
info "Installing PostgreSQL 16..."
if ! command -v psql &>/dev/null; then
    sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
    wget -qO- https://www.postgresql.org/media/keys/ACCC4CF8.asc | gpg --dearmor -o /etc/apt/trusted.gpg.d/pgdg.gpg
    apt-get update
    apt-get install -y postgresql-16 postgresql-client-16
else
    info "PostgreSQL already installed, skipping."
fi

# Start PostgreSQL
systemctl enable postgresql
systemctl start postgresql

# -------------------------------------------------------
# Step 3: Create database and user
# -------------------------------------------------------
info "Configuring PostgreSQL database..."

# Check for .env or use defaults
if [ -f "${PROJECT_DIR}/.env" ]; then
    # shellcheck disable=SC1091
    source "${PROJECT_DIR}/.env"
fi

DB_USER="${POSTGRES_USER:-tinyboards}"
DB_PASS="${POSTGRES_PASSWORD:-changeme_strong_password_here}"
DB_NAME="${POSTGRES_DB:-tinyboards}"

# Create user and database if they don't exist
sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='${DB_USER}'" | \
    grep -q 1 || sudo -u postgres psql -c "CREATE ROLE ${DB_USER} WITH LOGIN PASSWORD '${DB_PASS}';"

sudo -u postgres psql -tc "SELECT 1 FROM pg_database WHERE datname='${DB_NAME}'" | \
    grep -q 1 || sudo -u postgres psql -c "CREATE DATABASE ${DB_NAME} OWNER ${DB_USER};"

# -------------------------------------------------------
# Step 4: PostgreSQL tuning
# -------------------------------------------------------
info "Applying PostgreSQL configuration..."

# Detect RAM and choose appropriate config
TOTAL_RAM_KB=$(grep MemTotal /proc/meminfo | awk '{print $2}')
TOTAL_RAM_GB=$((TOTAL_RAM_KB / 1024 / 1024))

if [ "$TOTAL_RAM_GB" -ge 4 ]; then
    PG_CONF="postgresql_4gb.conf"
elif [ "$TOTAL_RAM_GB" -ge 2 ]; then
    PG_CONF="postgresql_2gb.conf"
else
    PG_CONF="postgresql_1gb.conf"
fi

info "Detected ${TOTAL_RAM_GB}GB RAM, using ${PG_CONF}"

PG_CONF_DIR=$(pg_config --sysconfdir 2>/dev/null || echo "/etc/postgresql/16/main")
if [ -d "/etc/postgresql/16/main" ]; then
    PG_CONF_DIR="/etc/postgresql/16/main"
fi

cp "${PROJECT_DIR}/config/${PG_CONF}" "${PG_CONF_DIR}/conf.d/tinyboards.conf" 2>/dev/null || \
    cp "${PROJECT_DIR}/config/${PG_CONF}" "${PG_CONF_DIR}/tinyboards.conf"

systemctl restart postgresql

# -------------------------------------------------------
# Step 5: Rust toolchain
# -------------------------------------------------------
info "Installing Rust toolchain..."
if ! command -v rustc &>/dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
else
    info "Rust already installed: $(rustc --version)"
fi

# -------------------------------------------------------
# Step 6: Node.js 20
# -------------------------------------------------------
info "Installing Node.js 20..."
if ! command -v node &>/dev/null || ! node --version | grep -q "^v20"; then
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
    apt-get install -y nodejs
else
    info "Node.js 20 already installed: $(node --version)"
fi

# -------------------------------------------------------
# Step 7: Copy project files
# -------------------------------------------------------
info "Setting up project directory at ${INSTALL_DIR}..."
mkdir -p "${INSTALL_DIR}"
cp -r "${PROJECT_DIR}"/* "${INSTALL_DIR}/"

if [ -f "${PROJECT_DIR}/.env" ]; then
    cp "${PROJECT_DIR}/.env" "${INSTALL_DIR}/.env"
elif [ ! -f "${INSTALL_DIR}/.env" ]; then
    cp "${PROJECT_DIR}/.env.example" "${INSTALL_DIR}/.env"
    warn "Created .env from .env.example — edit ${INSTALL_DIR}/.env before starting!"
fi

# -------------------------------------------------------
# Step 8: Build the backend
# -------------------------------------------------------
info "Building the backend (this may take a while)..."
cd "${INSTALL_DIR}/backend"
# shellcheck disable=SC1091
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
cargo build --release --locked
cp target/release/tinyboards_server "${INSTALL_DIR}/tinyboards_server"
strip "${INSTALL_DIR}/tinyboards_server"

# -------------------------------------------------------
# Step 9: Build the frontend
# -------------------------------------------------------
info "Building the frontend..."
cd "${INSTALL_DIR}/frontend"
npm ci --frozen-lockfile
npx nuxi build

# -------------------------------------------------------
# Step 10: Create media directory
# -------------------------------------------------------
mkdir -p /app/media
chown www-data:www-data /app/media

# -------------------------------------------------------
# Step 11: Run database migrations
# -------------------------------------------------------
info "Running database migrations..."
export DATABASE_URL="postgresql://${DB_USER}:${DB_PASS}@localhost:5432/${DB_NAME}"
bash "${INSTALL_DIR}/scripts/run_migrations.sh"

# -------------------------------------------------------
# Step 12: Install systemd units
# -------------------------------------------------------
info "Installing systemd services..."
cp "${INSTALL_DIR}/config/tinyboards-backend.service" /etc/systemd/system/
cp "${INSTALL_DIR}/config/tinyboards-frontend.service" /etc/systemd/system/

systemctl daemon-reload
systemctl enable tinyboards-backend tinyboards-frontend

# -------------------------------------------------------
# Step 13: Configure nginx
# -------------------------------------------------------
info "Configuring nginx..."
if ! command -v nginx &>/dev/null; then
    apt-get install -y nginx
fi

# Install the main nginx config
cp "${INSTALL_DIR}/config/nginx.conf" /etc/nginx/nginx.conf

# Start with HTTP-only config (SSL can be added later)
cp "${INSTALL_DIR}/config/nginx_site_http_only.conf" /etc/nginx/conf.d/tinyboards.conf

# Remove default site if present
rm -f /etc/nginx/sites-enabled/default

# Update upstream addresses for non-Docker (localhost instead of container names)
sed -i 's/server backend:8536/server 127.0.0.1:8536/' /etc/nginx/conf.d/tinyboards.conf
sed -i 's/server frontend:3000/server 127.0.0.1:3000/' /etc/nginx/conf.d/tinyboards.conf

nginx -t
systemctl enable nginx
systemctl restart nginx

# -------------------------------------------------------
# Step 14: Start services
# -------------------------------------------------------
info "Starting tinyboards services..."
systemctl start tinyboards-backend
systemctl start tinyboards-frontend

echo ""
echo "============================================"
echo "  Installation complete!"
echo "============================================"
echo ""
echo "Services:"
echo "  - Backend:  systemctl status tinyboards-backend"
echo "  - Frontend: systemctl status tinyboards-frontend"
echo "  - Nginx:    systemctl status nginx"
echo "  - Database: systemctl status postgresql"
echo ""
echo "Next steps:"
echo "  1. Edit ${INSTALL_DIR}/.env with your domain and secrets"
echo "  2. Set up SSL with certbot (see docs/SELF_HOSTING.md)"
echo "  3. Switch to the SSL nginx config"
echo "  4. Restart services: systemctl restart tinyboards-backend tinyboards-frontend nginx"
echo ""
