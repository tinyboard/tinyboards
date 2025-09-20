#!/bin/bash

# TinyBoards Quick Start with Registry Images
# This script quickly sets up TinyBoards using pre-built DockerHub images

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$(dirname "$SCRIPT_DIR")"

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

show_usage() {
    cat << EOF
TinyBoards Quick Start with Registry Images

Usage: $0 [OPTIONS]

Options:
    -v, --version TAG   Image version to use (default: latest)
    -h, --help         Show this help message

This script will:
1. Pull the latest TinyBoards images from DockerHub
2. Set up environment configuration
3. Start all services with docker-compose
4. Verify deployment health

Prerequisites:
- Docker and Docker Compose installed
- Internet connection for pulling images

EOF
}

check_requirements() {
    log_info "Checking requirements..."

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose is not installed"
        exit 1
    fi

    if ! docker info &> /dev/null; then
        log_error "Cannot access Docker daemon. Please check Docker is running"
        exit 1
    fi

    log_success "Requirements check passed"
}

setup_environment() {
    local version="$1"

    log_info "Setting up environment configuration..."

    cd "$DOCKER_DIR"

    # Create .env file if it doesn't exist
    if [[ ! -f ".env" ]]; then
        if [[ -f ".env.prod.example" ]]; then
            log_info "Creating .env from .env.prod.example"
            cp .env.prod.example .env
        else
            log_warning "No .env.prod.example found, creating basic .env"
            cat > .env << EOF
# TinyBoards Configuration
DOMAIN=localhost
POSTGRES_USER=tinyboards
POSTGRES_PASSWORD=change_this_password_in_production
POSTGRES_DB=tinyboards
SALT_SUFFIX=change_this_salt_in_production
ADMIN_USERNAME=admin
ADMIN_PASSWORD=change_this_admin_password
SITE_NAME=TinyBoards Instance
RUST_LOG=info
NODE_ENV=production
EOF
        fi

        log_warning "Please edit .env file to configure your instance:"
        log_warning "- Change POSTGRES_PASSWORD"
        log_warning "- Change SALT_SUFFIX"
        log_warning "- Change ADMIN_PASSWORD"
        log_warning "- Set your DOMAIN"
    fi

    # Update image versions in docker-compose.registry.yml
    if [[ "$version" != "latest" ]]; then
        log_info "Updating image versions to $version"
        sed -i "s|kronusdev/tinyboards-be:.*|kronusdev/tinyboards-be:$version|g" docker-compose.registry.yml
        sed -i "s|kronusdev/tinyboards-fe:.*|kronusdev/tinyboards-fe:$version|g" docker-compose.registry.yml
    fi

    log_success "Environment setup completed"
}

pull_images() {
    local version="$1"

    log_info "Pulling TinyBoards images from DockerHub..."

    # Pull images
    docker pull "kronusdev/tinyboards-be:$version"
    docker pull "kronusdev/tinyboards-fe:$version"
    docker pull "postgres:15-alpine"
    docker pull "redis:7-alpine"
    docker pull "nginx:1.25-alpine"

    log_success "Images pulled successfully"
}

start_services() {
    log_info "Starting TinyBoards services..."

    cd "$DOCKER_DIR"

    # Start services
    docker-compose -f docker-compose.registry.yml up -d

    log_success "Services started"
}

wait_for_services() {
    log_info "Waiting for services to be healthy..."

    local max_attempts=60
    local attempt=1

    while [[ $attempt -le $max_attempts ]]; do
        if docker-compose -f docker-compose.registry.yml ps | grep -q "Up (healthy)"; then
            if curl -sf "http://localhost:8536/api/v1/site" > /dev/null 2>&1; then
                log_success "Backend is healthy"
                break
            fi
        fi

        if [[ $attempt -eq $max_attempts ]]; then
            log_error "Services failed to become healthy"
            log_info "Checking service status..."
            docker-compose -f docker-compose.registry.yml ps
            log_info "Recent logs:"
            docker-compose -f docker-compose.registry.yml logs --tail=20
            exit 1
        fi

        echo -n "."
        sleep 5
        ((attempt++))
    done

    echo ""
    log_success "All services are healthy"
}

show_status() {
    log_info "TinyBoards deployment status:"

    cd "$DOCKER_DIR"

    echo ""
    echo "Service Status:"
    docker-compose -f docker-compose.registry.yml ps

    echo ""
    echo "Access URLs:"
    echo "- Frontend: http://localhost (via nginx)"
    echo "- Backend API: http://localhost:8536"
    echo "- GraphQL Playground: http://localhost:8536/graphql"

    echo ""
    echo "Admin Access:"
    echo "- Check .env file for ADMIN_USERNAME and ADMIN_PASSWORD"
    echo "- First user to register becomes admin if no admin exists"

    echo ""
    echo "Management Commands:"
    echo "- View logs: docker-compose -f docker-compose.registry.yml logs -f"
    echo "- Stop services: docker-compose -f docker-compose.registry.yml down"
    echo "- Update images: docker-compose -f docker-compose.registry.yml pull && docker-compose -f docker-compose.registry.yml up -d"
}

# Default values
VERSION="latest"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
log_info "🚀 Starting TinyBoards Quick Setup with Registry Images"
log_info "Version: $VERSION"
echo ""

check_requirements
setup_environment "$VERSION"
pull_images "$VERSION"
start_services
wait_for_services
show_status

echo ""
log_success "🎉 TinyBoards is now running!"
log_info "Visit http://localhost to access your instance"