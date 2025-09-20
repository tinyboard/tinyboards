#!/bin/bash

# TinyBoards Production Deployment Script
# This script automates the deployment process with safety checks
# Supports both backend and frontend deployment from separate repositories
# Frontend repo should be located at ../tinyboards-fe relative to the backend

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$DOCKER_DIR")"
FRONTEND_ROOT="$(dirname "$PROJECT_ROOT")/tinyboards-fe"
ENV_FILE="$DOCKER_DIR/.env"
COMPOSE_FILE="$DOCKER_DIR/docker-compose.prod.yml"
BACKUP_DIR="/opt/backups/tinyboards"

# Frontend configuration
FE_BUILD_DIR="$FRONTEND_ROOT/.output"
FE_DOCKER_DIR="$FRONTEND_ROOT/docker"
FE_DOCKERFILE="$FE_DOCKER_DIR/Dockerfile"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_requirements() {
    log_info "Checking requirements..."

    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi

    # Check if Docker Compose is installed
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed"
        exit 1
    fi

    # Check if running as root or with sudo
    if [[ $EUID -ne 0 ]] && ! groups $USER | grep -q docker; then
        log_error "Please run as root or add user to docker group"
        exit 1
    fi

    # Check if Node.js is installed (for frontend builds)
    if ! command -v node &> /dev/null; then
        log_error "Node.js is not installed (required for frontend builds)"
        exit 1
    fi

    # Check if npm is installed
    if ! command -v npm &> /dev/null; then
        log_error "npm is not installed (required for frontend builds)"
        exit 1
    fi

    # Check if frontend directory exists
    if [[ ! -d "$FRONTEND_ROOT" ]]; then
        log_error "Frontend directory not found: $FRONTEND_ROOT"
        log_info "Please ensure tinyboards-fe is cloned alongside tinyboards"
        exit 1
    fi

    log_success "Requirements check passed"
}

check_environment() {
    log_info "Checking environment configuration..."

    if [[ ! -f "$ENV_FILE" ]]; then
        log_error "Environment file not found: $ENV_FILE"
        log_info "Please copy .env.prod.example to .env and configure it"
        exit 1
    fi

    # Source environment file
    set -a
    source "$ENV_FILE"
    set +a

    # Check critical environment variables
    critical_vars=(
        "DOMAIN"
        "POSTGRES_PASSWORD"
        "SALT_SUFFIX"
        "ADMIN_PASSWORD"
    )

    for var in "${critical_vars[@]}"; do
        if [[ -z "${!var:-}" ]]; then
            log_error "Critical environment variable $var is not set"
            exit 1
        fi

        # Check for default values that should be changed
        case $var in
            "POSTGRES_PASSWORD")
                if [[ "${!var}" == *"change_this"* ]]; then
                    log_error "Please change the default $var in $ENV_FILE"
                    exit 1
                fi
                ;;
            "SALT_SUFFIX")
                if [[ "${!var}" == *"change_in_production"* ]] || [[ "${!var}" == *"your_"* ]]; then
                    log_error "Please change the default $var in $ENV_FILE"
                    exit 1
                fi
                ;;
            "ADMIN_PASSWORD")
                if [[ ${#!var} -lt 12 ]]; then
                    log_error "Admin password must be at least 12 characters long"
                    exit 1
                fi
                ;;
        esac
    done

    log_success "Environment configuration check passed"
}

check_ssl_certificates() {
    log_info "Checking SSL certificates..."

    if [[ "${TLS_ENABLED:-false}" == "true" ]]; then
        cert_path="/etc/letsencrypt/live/${DOMAIN}/fullchain.pem"
        key_path="/etc/letsencrypt/live/${DOMAIN}/privkey.pem"

        if [[ ! -f "$cert_path" ]] || [[ ! -f "$key_path" ]]; then
            log_warning "SSL certificates not found"
            log_info "Would you like to generate Let's Encrypt certificates? (y/n)"
            read -r response
            if [[ "$response" == "y" ]]; then
                generate_ssl_certificates
            else
                log_error "SSL is enabled but certificates not found"
                exit 1
            fi
        else
            log_success "SSL certificates found"
        fi
    else
        log_warning "TLS is disabled - not recommended for production"
    fi
}

generate_ssl_certificates() {
    log_info "Generating SSL certificates with Let's Encrypt..."

    if ! command -v certbot &> /dev/null; then
        log_info "Installing certbot..."
        apt-get update
        apt-get install -y certbot
    fi

    # Stop nginx if running to free port 80
    docker-compose -f "$COMPOSE_FILE" stop nginx 2>/dev/null || true

    # Generate certificate
    certbot certonly --standalone \
        --email "${LETSENCRYPT_EMAIL}" \
        --agree-tos \
        --no-eff-email \
        -d "${DOMAIN}"

    log_success "SSL certificates generated"
}

create_backup() {
    log_info "Creating backup before deployment..."

    local backup_name="tinyboards-backup-$(date +%Y%m%d-%H%M%S)"
    local backup_path="$BACKUP_DIR/$backup_name"

    mkdir -p "$backup_path"

    # Backup database if running
    if docker-compose -f "$COMPOSE_FILE" ps postgres | grep -q "Up"; then
        log_info "Backing up database..."
        docker-compose -f "$COMPOSE_FILE" exec -T postgres \
            pg_dump -U "${POSTGRES_USER}" "${POSTGRES_DB}" \
            | gzip > "$backup_path/database.sql.gz"
    fi

    # Backup media files
    if [[ -d "/opt/tinyboards/media" ]]; then
        log_info "Backing up media files..."
        tar -czf "$backup_path/media.tar.gz" -C "/opt/tinyboards" media/
    fi

    # Backup frontend logs if they exist
    if [[ -d "/opt/tinyboards/frontend_logs" ]]; then
        log_info "Backing up frontend logs..."
        tar -czf "$backup_path/frontend_logs.tar.gz" -C "/opt/tinyboards" frontend_logs/
    fi

    # Backup configuration
    cp "$ENV_FILE" "$backup_path/env"

    # Backup frontend configuration if it exists
    if [[ -f "$FRONTEND_ROOT/.env" ]]; then
        cp "$FRONTEND_ROOT/.env" "$backup_path/frontend_env"
    fi

    echo "$backup_path" > "$BACKUP_DIR/latest-backup"
    log_success "Backup created: $backup_path"
}

check_frontend_environment() {
    log_info "Checking frontend environment..."

    if [[ ! -f "$FRONTEND_ROOT/package.json" ]]; then
        log_error "Frontend package.json not found"
        exit 1
    fi

    # Check if .env exists in frontend
    if [[ ! -f "$FRONTEND_ROOT/.env" ]]; then
        log_warning "Frontend .env file not found"
        if [[ -f "$FRONTEND_ROOT/.env.example" ]]; then
            log_info "Copying .env.example to .env"
            cp "$FRONTEND_ROOT/.env.example" "$FRONTEND_ROOT/.env"
        else
            log_error "No .env.example found in frontend directory"
            exit 1
        fi
    fi

    log_success "Frontend environment check passed"
}

build_frontend() {
    log_info "Building frontend application..."

    cd "$FRONTEND_ROOT"

    # Install dependencies
    log_info "Installing frontend dependencies..."
    npm ci --production=false

    # Set production environment
    export NODE_ENV=production

    # Build the application
    log_info "Building frontend for production..."
    npm run build

    # Verify build output
    if [[ ! -d "$FE_BUILD_DIR" ]]; then
        log_error "Frontend build failed - output directory not found"
        exit 1
    fi

    log_success "Frontend build completed"
    cd "$DOCKER_DIR"
}

build_frontend_image() {
    log_info "Building frontend Docker image..."

    cd "$FRONTEND_ROOT"

    # Check if Dockerfile exists
    if [[ ! -f "$FE_DOCKERFILE" ]]; then
        log_info "Creating production Dockerfile for frontend..."
        create_frontend_dockerfile
    fi

    # Build the Docker image with production tag
    docker build -f "$FE_DOCKERFILE" -t tinyboards-fe:latest .

    # Tag as kronusdev/tinyboards-fe:latest for compatibility with compose file
    docker tag tinyboards-fe:latest kronusdev/tinyboards-fe:latest

    log_success "Frontend Docker image built"
    cd "$DOCKER_DIR"
}

create_frontend_dockerfile() {
    log_info "Creating optimized Dockerfile for frontend..."

    mkdir -p "$FE_DOCKER_DIR"

    cat > "$FE_DOCKERFILE" << 'EOF'
# Multi-stage build for Nuxt.js frontend
FROM node:18-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production && npm cache clean --force

# Copy source code
COPY . .

# Build the application
RUN npm run build

# Production stage
FROM node:18-alpine AS runtime

WORKDIR /app

# Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nuxt -u 1001

# Copy built application
COPY --from=builder --chown=nuxt:nodejs /app/.output /app/.output

# Switch to non-root user
USER nuxt

# Expose port
EXPOSE 3000

# Set environment
ENV NODE_ENV=production
ENV NUXT_HOST=0.0.0.0
ENV NUXT_PORT=3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Start the application
CMD ["node", ".output/server/index.mjs"]
EOF

    log_success "Frontend Dockerfile created"
}

deploy() {
    log_info "Starting deployment..."

    cd "$DOCKER_DIR"

    # Build frontend first
    check_frontend_environment
    build_frontend
    build_frontend_image

    # Pull latest backend images (excluding frontend since we build it locally)
    log_info "Pulling latest backend images..."
    docker-compose -f "$COMPOSE_FILE" pull tinyboards postgres redis nginx 2>/dev/null || true

    # Stop services gracefully
    log_info "Stopping services..."
    docker-compose -f "$COMPOSE_FILE" down --timeout 30

    # Start services
    log_info "Starting services..."
    docker-compose -f "$COMPOSE_FILE" up -d

    # Wait for services to be healthy
    log_info "Waiting for services to be healthy..."
    sleep 30

    # Check health
    check_deployment_health

    log_success "Deployment completed successfully"
}

check_deployment_health() {
    log_info "Checking deployment health..."

    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if curl -sf "http://localhost:8536/api/v1/site" > /dev/null 2>&1; then
            log_success "Backend is healthy"
            break
        fi

        if [[ $attempt -eq $max_attempts ]]; then
            log_error "Backend health check failed"
            show_logs
            exit 1
        fi

        log_info "Attempt $attempt/$max_attempts - waiting for backend..."
        sleep 10
        ((attempt++))
    done

    # Check frontend
    attempt=1
    while [[ $attempt -le $max_attempts ]]; do
        if curl -sf "http://localhost:3000/health" > /dev/null 2>&1; then
            log_success "Frontend is healthy"
            break
        fi

        if [[ $attempt -eq $max_attempts ]]; then
            log_error "Frontend health check failed"
            show_logs
            exit 1
        fi

        log_info "Attempt $attempt/$max_attempts - waiting for frontend..."
        sleep 10
        ((attempt++))
    done
}

show_logs() {
    log_info "Recent logs:"
    docker-compose -f "$COMPOSE_FILE" logs --tail=50
}

rollback() {
    log_warning "Rolling back to previous backup..."

    if [[ ! -f "$BACKUP_DIR/latest-backup" ]]; then
        log_error "No backup found for rollback"
        exit 1
    fi

    local backup_path=$(cat "$BACKUP_DIR/latest-backup")

    if [[ ! -d "$backup_path" ]]; then
        log_error "Backup directory not found: $backup_path"
        exit 1
    fi

    # Stop current services
    docker-compose -f "$COMPOSE_FILE" down

    # Restore database
    if [[ -f "$backup_path/database.sql.gz" ]]; then
        log_info "Restoring database..."
        # Start only postgres for restore
        docker-compose -f "$COMPOSE_FILE" up -d postgres
        sleep 10

        gunzip -c "$backup_path/database.sql.gz" | \
        docker-compose -f "$COMPOSE_FILE" exec -T postgres \
            psql -U "${POSTGRES_USER}" -d "${POSTGRES_DB}"
    fi

    # Restore media files
    if [[ -f "$backup_path/media.tar.gz" ]]; then
        log_info "Restoring media files..."
        tar -xzf "$backup_path/media.tar.gz" -C "/opt/tinyboards/"
    fi

    # Start all services
    docker-compose -f "$COMPOSE_FILE" up -d

    log_success "Rollback completed"
}

cleanup() {
    log_info "Cleaning up old images and volumes..."
    docker system prune -f
    docker volume prune -f

    # Clean up frontend build artifacts
    if [[ -d "$FRONTEND_ROOT" ]]; then
        log_info "Cleaning up frontend build artifacts..."
        cd "$FRONTEND_ROOT"
        rm -rf .output .nuxt node_modules/.cache dist 2>/dev/null || true
        npm cache clean --force 2>/dev/null || true
        cd "$DOCKER_DIR"
    fi

    log_success "Cleanup completed"
}

show_status() {
    log_info "Current deployment status:"
    docker-compose -f "$COMPOSE_FILE" ps

    log_info "Service health:"
    docker-compose -f "$COMPOSE_FILE" exec nginx nginx -t 2>/dev/null && log_success "Nginx: OK" || log_error "Nginx: Error"

    if curl -sf "http://localhost:8536/api/v1/site" > /dev/null 2>&1; then
        log_success "Backend: OK"
    else
        log_error "Backend: Error"
    fi

    if curl -sf "http://localhost:3000/health" > /dev/null 2>&1; then
        log_success "Frontend: OK"
    else
        log_error "Frontend: Error"
    fi
}

deploy_frontend_only() {
    log_info "Starting frontend-only deployment..."

    # Build and deploy only frontend
    check_frontend_environment
    build_frontend
    build_frontend_image

    # Restart only frontend service
    log_info "Restarting frontend service..."
    docker-compose -f "$COMPOSE_FILE" up -d --no-deps tinyboards-fe

    # Wait for frontend to be healthy
    log_info "Waiting for frontend to be healthy..."
    sleep 15

    # Check frontend health only
    local max_attempts=30
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if curl -sf "http://localhost:3000/health" > /dev/null 2>&1; then
            log_success "Frontend is healthy"
            break
        fi

        if [[ $attempt -eq $max_attempts ]]; then
            log_error "Frontend health check failed"
            docker-compose -f "$COMPOSE_FILE" logs tinyboards-fe
            exit 1
        fi

        log_info "Attempt $attempt/$max_attempts - waiting for frontend..."
        sleep 5
        ((attempt++))
    done

    log_success "Frontend deployment completed successfully"
}

show_usage() {
    echo "Usage: $0 {deploy|deploy-fe|rollback|status|backup|cleanup|logs}"
    echo ""
    echo "Commands:"
    echo "  deploy     - Deploy the full application (backend + frontend)"
    echo "  deploy-fe  - Deploy only the frontend application"
    echo "  rollback   - Rollback to the previous backup"
    echo "  status     - Show current deployment status"
    echo "  backup     - Create a backup of current data"
    echo "  cleanup    - Clean up old Docker images and volumes"
    echo "  logs       - Show recent logs"
}

# Main script logic
case "${1:-}" in
    "deploy")
        check_requirements
        check_environment
        check_ssl_certificates
        create_backup
        deploy
        ;;
    "deploy-fe")
        check_requirements
        deploy_frontend_only
        ;;
    "rollback")
        check_requirements
        rollback
        ;;
    "status")
        show_status
        ;;
    "backup")
        check_requirements
        check_environment
        create_backup
        ;;
    "cleanup")
        check_requirements
        cleanup
        ;;
    "logs")
        show_logs
        ;;
    *)
        show_usage
        exit 1
        ;;
esac