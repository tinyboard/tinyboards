# TinyBoards Docker Setup

A comprehensive Docker-based deployment solution for TinyBoards, providing both development and production environments with full orchestration of all required services.

## 📋 Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Development Setup](#development-setup)
- [Production Deployment](#production-deployment)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)
- [Architecture](#architecture)
- [Advanced Usage](#advanced-usage)
- [Maintenance](#maintenance)

## 🌟 Overview

This Docker setup provides a complete TinyBoards deployment with the following components:

### Core Services
- **TinyBoards Backend** - Rust/Actix-based API server with GraphQL
- **TinyBoards Frontend** - Nuxt.js 3 web interface
- **PostgreSQL 14/15** - Primary database with optimized configuration
- **Nginx** - Reverse proxy with SSL termination and caching
- **Redis** - Session storage and caching (production only)

### Key Features
- 🔧 **Dual Environment Support** - Separate development and production configurations
- 🔒 **Security Hardened** - Container security best practices and capability dropping
- 🚀 **Production Optimized** - Health checks, resource limits, and monitoring
- 📈 **Scalable Architecture** - Microservices design with service isolation
- 🔄 **Auto-restart** - Automatic service recovery and dependency management
- 📊 **Monitoring Ready** - Structured logging and health endpoint integration

## 📋 Prerequisites

### System Requirements
- **Docker**: Version 20.10+ with Compose V2
- **System Memory**: Minimum 2GB RAM (4GB+ recommended for production)
- **Storage**: 10GB+ available disk space
- **Network**: Ports 80, 443, 3000, 5433, 8536 available

### Operating System Support
- ✅ **Linux** (Ubuntu 20.04+, Debian 11+, CentOS 8+)
- ✅ **macOS** (Docker Desktop)
- ✅ **Windows** (Docker Desktop with WSL2)

### Development Prerequisites
- **Rust**: Latest stable (for building from source)
- **Node.js**: v18+ (for frontend development)
- **Git**: For cloning repositories

## 🚀 Quick Start

### 1. Clone and Setup
```bash
# Clone the repository
git clone <repository-url>
cd tinyboards/docker

# Copy environment file
cp .env.example .env
```

### 2. Configure Environment
Edit `.env` with your basic settings:
```bash
# Basic Configuration
DOMAIN=localhost
POSTGRES_PASSWORD=your_secure_password

# For production, also set:
NUXT_PUBLIC_DOMAIN=your-domain.com
NUXT_PUBLIC_USE_HTTPS=true
```

### 3. Launch Services
```bash
# Development environment
docker compose -f docker-compose.dev.yml up -d

# Production environment
docker compose -f docker-compose.prod.yml up -d
```

### 4. Access Application
- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8536
- **GraphQL Playground**: http://localhost:8536/graphql
- **Admin Panel**: Log in with credentials from setup logs

## 🔧 Development Setup

### Environment Configuration
```bash
# Copy development environment
cp .env.example .env

# Edit with development settings
vim .env
```

#### Development .env Example
```bash
# Development Configuration
DOMAIN=tinyboards.test
NUXT_PUBLIC_DOMAIN=tinyboards.test
NUXT_PUBLIC_USE_HTTPS=false

# Database
POSTGRES_USER=tinyboards
POSTGRES_PASSWORD=tinyboards
POSTGRES_DB=tinyboards

# Site Setup
ADMIN_USERNAME=admin
ADMIN_PASSWORD=admin123456
SITE_NAME=TinyBoards Dev
```

### Development Commands
```bash
# Start development environment
docker compose -f docker-compose.dev.yml up -d

# View logs
docker compose -f docker-compose.dev.yml logs -f

# Rebuild after code changes
docker compose -f docker-compose.dev.yml build --no-cache
docker compose -f docker-compose.dev.yml up -d

# Stop services
docker compose -f docker-compose.dev.yml down

# Clean up (removes volumes)
docker compose -f docker-compose.dev.yml down -v
```

### Development Features
- **Hot Reload**: Frontend changes reflected immediately
- **Source Building**: Backend builds from local source code
- **Debug Logging**: `RUST_LOG=debug` enabled by default
- **Local Domains**: Uses `tinyboards.test` for testing
- **Port Exposure**: All services exposed for debugging

### Working with Source Code
```bash
# Backend development
cd ../  # Repository root
cargo run  # Direct Rust development

# Frontend development
cd ../../tinyboards-fe
npm run dev  # Direct Node.js development

# Database access
docker compose exec postgres psql -U tinyboards -d tinyboards
```

## 🏭 Production Deployment

### Environment Setup
```bash
# Copy production template
cp .env.prod.example .env

# Configure for your domain
vim .env
```

#### Production .env Configuration
```bash
# Domain Configuration
DOMAIN=your-domain.com
NUXT_PUBLIC_DOMAIN=your-domain.com
NUXT_PUBLIC_USE_HTTPS=true

# Secure Database Credentials
POSTGRES_USER=tinyboards
POSTGRES_PASSWORD=your_very_secure_password_here
POSTGRES_DB=tinyboards

# Security
SALT_SUFFIX=your_unique_salt_here_32_chars_min

# SSL Configuration
LETSENCRYPT_EMAIL=admin@your-domain.com

# Admin Setup (first run only)
ADMIN_USERNAME=admin
ADMIN_PASSWORD=your_secure_admin_password_here
SITE_NAME=Your TinyBoards Instance

# Performance
RUST_LOG=info
NODE_ENV=production
```

### SSL/TLS Setup

#### Option 1: Let's Encrypt (Recommended)
```bash
# Install certbot
sudo apt install certbot

# Get certificate
sudo certbot certonly --standalone -d your-domain.com

# Copy certificates to expected location
sudo cp /etc/letsencrypt/live/your-domain.com/fullchain.pem /etc/ssl/
sudo cp /etc/letsencrypt/live/your-domain.com/privkey.pem /etc/ssl/
```

#### Option 2: Custom Certificates
```bash
# Place your certificates
sudo cp your-certificate.pem /etc/ssl/fullchain.pem
sudo cp your-private-key.pem /etc/ssl/privkey.pem

# Set proper permissions
sudo chmod 644 /etc/ssl/fullchain.pem
sudo chmod 600 /etc/ssl/privkey.pem
```

### Production Deployment

#### Automated Deployment Script
The repository includes a comprehensive deployment script with frontend support:

```bash
# Full deployment (backend + frontend)
./scripts/deploy.sh deploy

# Frontend-only deployment
./scripts/deploy.sh deploy-fe

# Check deployment status
./scripts/deploy.sh status

# Create backup
./scripts/deploy.sh backup

# Rollback to previous version
./scripts/deploy.sh rollback

# Clean up old images and artifacts
./scripts/deploy.sh cleanup

# View recent logs
./scripts/deploy.sh logs
```

#### Manual Deployment
```bash
# Start production services
docker compose -f docker-compose.prod.yml up -d

# Monitor startup
docker compose -f docker-compose.prod.yml logs -f

# Verify health status
docker compose -f docker-compose.prod.yml ps
```

#### Frontend Integration
The deployment script automatically handles frontend deployment from the `../tinyboards-fe` repository:

**Prerequisites:**
- Frontend repository cloned at `../tinyboards-fe`
- Node.js and npm installed
- Frontend `.env` file configured

**Deployment Process:**
1. **Environment Check**: Validates frontend configuration
2. **Dependency Installation**: Runs `npm ci` for production
3. **Build Process**: Generates optimized production build
4. **Docker Image**: Creates containerized frontend
5. **Service Deployment**: Updates frontend service with zero downtime
6. **Health Verification**: Ensures frontend is responding correctly

**Frontend-Only Updates:**
```bash
# Quick frontend deployment for development iterations
./scripts/deploy.sh deploy-fe
```

### Production Features
- **Health Checks**: All services monitored with automatic restart
- **Resource Limits**: Memory and CPU limits prevent resource exhaustion
- **Security Hardening**: Dropped capabilities and security options
- **Optimized Images**: Production-tuned PostgreSQL and Redis configurations
- **Caching**: Nginx caching and Redis session storage
- **Monitoring**: Structured logging and performance metrics

## ⚙️ Configuration

### Environment Variables Reference

#### Database Configuration
```bash
POSTGRES_USER=tinyboards          # Database username
POSTGRES_PASSWORD=secure_password # Database password (CHANGE THIS!)
POSTGRES_DB=tinyboards           # Database name
```

#### Site Configuration
```bash
DOMAIN=your-domain.com           # Your site's domain
NUXT_PUBLIC_DOMAIN=your-domain.com # Frontend domain (usually same)
NUXT_PUBLIC_USE_HTTPS=true       # Enable HTTPS in frontend
```

#### Security Settings
```bash
SALT_SUFFIX=unique_salt_32chars   # Password hashing salt (IMPORTANT!)
ADMIN_USERNAME=admin             # Initial admin username
ADMIN_PASSWORD=secure_admin_pass # Initial admin password (10+ chars)
```

#### Email Configuration (Optional)
```bash
SMTP_SERVER=smtp.your-provider.com:587
SMTP_LOGIN=noreply@your-domain.com
SMTP_PASSWORD=your_email_password
SMTP_FROM_ADDRESS=noreply@your-domain.com
TLS_TYPE=starttls               # Options: none, tls, starttls
```

#### Performance Tuning
```bash
RUST_LOG=info                   # Logging level (debug, info, warn, error)
NODE_ENV=production             # Node.js environment
```

### TinyBoards Configuration File

The main application configuration is in `tinyboards.hjson`:

```hjson
{
  database: {
    user: "tinyboards"
    password: "your_password"
    host: "postgres"
    port: 5432
    database: "tinyboards"
    pool_size: 10
  }

  rate_limit: {
    message: 180              # Messages per minute
    post: 6                   # Posts per 10 minutes
    register: 3               # Registrations per hour
    image: 6                  # Image uploads per hour
    comment: 6                # Comments per 10 minutes
    search: 60                # Searches per 10 minutes
  }

  hostname: "your-domain.com"
  port: 8536
  tls_enabled: true
  environment: "prod"         # "dev" or "prod"
}
```

### Nginx Configuration

Nginx serves as a reverse proxy with the following routing:

- **Frontend Routes** (`/`) → TinyBoards Frontend (Port 3000)
- **API Routes** (`/api/*`) → TinyBoards Backend (Port 8536)
- **GraphQL** (`/graphql`) → TinyBoards Backend (Port 8536)
- **Media Files** (`/image/*`) → TinyBoards Backend (Port 8536)

#### Custom Nginx Configuration
```bash
# Edit nginx configuration
vim nginx/conf/nginx.conf

# Reload nginx after changes
docker compose exec nginx nginx -s reload
```

## 🔧 Troubleshooting

### Common Issues

#### 1. Services Won't Start
```bash
# Check service status
docker compose ps

# View service logs
docker compose logs service_name

# Common causes:
# - Port conflicts (change ports in the appropriate docker-compose file)
# - Permission issues (check file ownership)
# - Memory issues (increase Docker memory limit)
```

#### 2. Database Connection Issues
```bash
# Check PostgreSQL status
docker compose logs postgres

# Test database connection
docker compose exec postgres psql -U tinyboards -d tinyboards

# Common solutions:
# - Verify POSTGRES_PASSWORD in .env
# - Check if PostgreSQL finished initializing
# - Ensure database volume has proper permissions
```

#### 3. Frontend Can't Connect to Backend
```bash
# Check backend logs
docker compose logs tinyboards

# Verify CORS configuration in tinyboards.hjson
# Check if frontend environment variables match backend domain
```

#### 4. SSL/TLS Issues
```bash
# Verify certificate files exist
ls -la /etc/ssl/fullchain.pem /etc/ssl/privkey.pem

# Check nginx SSL configuration
docker compose exec nginx nginx -t

# View nginx logs
docker compose logs nginx
```

#### 5. Permission Denied Errors
```bash
# Fix volume permissions
sudo chown -R 999:999 volumes/postgres/
sudo chown -R 1000:1000 volumes/media/

# For nginx
sudo chown -R 101:101 volumes/nginx_cache/
```

### Debugging Commands

#### Service Inspection
```bash
# View all container logs
docker compose logs -f

# Inspect specific service
docker compose exec tinyboards bash

# Check service health
docker compose ps
docker stats
```

#### Database Debugging
```bash
# Connect to PostgreSQL
docker compose exec postgres psql -U tinyboards -d tinyboards

# View database logs
docker compose logs postgres

# Check database performance
docker compose exec postgres pg_stat_activity
```

#### Network Debugging
```bash
# Test service connectivity
docker compose exec tinyboards curl http://postgres:5432
docker compose exec nginx curl http://tinyboards:8536

# View network configuration
docker network ls
docker network inspect tinyboards_tinyboards
```

### Log Analysis

#### Important Log Locations
```bash
# Application logs
docker compose logs tinyboards

# Database logs
docker compose logs postgres

# Web server logs
docker compose logs nginx

# All services
docker compose logs
```

#### Log Filtering
```bash
# Error logs only
docker compose logs | grep ERROR

# Database connection issues
docker compose logs postgres | grep "connection"

# Authentication problems
docker compose logs tinyboards | grep "auth"
```

## 🏗️ Architecture

### Service Overview

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Client    │    │    Nginx    │    │ TinyBoards  │
│  (Browser)  │◄──►│   (Proxy)   │◄──►│  Frontend   │
└─────────────┘    └─────────────┘    └─────────────┘
                          │                   │
                          ▼                   ▼
                   ┌─────────────┐    ┌─────────────┐
                   │ TinyBoards  │    │ PostgreSQL  │
                   │   Backend   │◄──►│ (Database)  │
                   └─────────────┘    └─────────────┘
                          │
                          ▼
                   ┌─────────────┐
                   │    Redis    │
                   │  (Cache)    │
                   └─────────────┘
```

### Service Relationships

#### Frontend Service (tinyboards-fe)
- **Image**: `kronusdev/tinyboards-fe:latest`
- **Port**: 3000
- **Dependencies**: TinyBoards Backend
- **Function**: Vue.js/Nuxt.js web interface
- **Health Check**: HTTP request to `/health`

#### Backend Service (tinyboards)
- **Image**: `kronusdev/tinyboards-be:latest`
- **Port**: 8536
- **Dependencies**: PostgreSQL, Redis
- **Function**: Rust/Actix GraphQL API server
- **Health Check**: HTTP request to root endpoint

#### Database Service (postgres)
- **Image**: `postgres:15-alpine`
- **Port**: 5432 (exposed as 5433)
- **Function**: Primary data storage
- **Optimizations**: Connection tuning, query optimization
- **Health Check**: PostgreSQL readiness probe

#### Reverse Proxy (nginx)
- **Image**: `nginx:1.25-alpine`
- **Ports**: 80, 443
- **Function**: SSL termination, routing, caching
- **Features**: Gzip compression, static file serving

#### Cache Service (redis) - Production Only
- **Image**: `redis:7-alpine`
- **Function**: Session storage, application caching
- **Configuration**: LRU eviction, append-only persistence

### Networking

#### Network: `tinyboards`
- **Type**: Bridge network
- **Isolation**: Internal service communication
- **DNS**: Automatic service discovery

#### Port Mapping
- **80/443**: Nginx (public access)
- **3000**: Frontend (development only)
- **8536**: Backend (development only)
- **5433**: PostgreSQL (external access)

### Volume Management

#### Production Volumes
```yaml
postgres_data:     # Database storage
nginx_cache:       # Nginx cache data
redis_data:        # Redis persistence
media:            # User uploaded content
```

#### Development Volumes
```yaml
uploads:          # Development media uploads
source_code:      # Live code mounting for development
```

### Security Model

#### Container Security
- **Capability Dropping**: Removes unnecessary Linux capabilities
- **Non-privileged Users**: Services run as non-root
- **Read-only Mounts**: Configuration files mounted read-only
- **Security Options**: `no-new-privileges` flag enabled

#### Network Security
- **Internal Communication**: Services isolated in private network
- **Port Exposure**: Only necessary ports exposed to host
- **SSL/TLS**: End-to-end encryption for public traffic

## 🚀 Advanced Usage

### Scaling and High Availability

#### Horizontal Scaling
```bash
# Scale backend services
docker compose up -d --scale tinyboards=3

# Scale with load balancer
# Edit nginx configuration to add upstream backends
```

#### Database Replication
```yaml
# Add read replica (docker-compose.override.yml)
postgres-replica:
  image: postgres:15-alpine
  environment:
    POSTGRES_USER: tinyboards
    POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    POSTGRES_DB: tinyboards
    POSTGRES_MASTER_SERVICE: postgres
  command: |
    bash -c "
    until pg_basebackup -h postgres -D /var/lib/postgresql/data -U replication -v -P; do
      echo 'Waiting for master to be available...'
      sleep 1s
    done
    echo 'host replication replication 0.0.0.0/0 md5' >> /var/lib/postgresql/data/pg_hba.conf
    postgres
    "
```

### Monitoring and Observability

#### Health Monitoring
```bash
# Check all service health
docker compose ps

# Automated health checking script
#!/bin/bash
for service in tinyboards tinyboards-fe postgres nginx; do
  if ! docker compose exec $service healthcheck; then
    echo "Service $service is unhealthy"
    # Add alerting logic here
  fi
done
```

#### Performance Monitoring
```bash
# Resource usage
docker stats

# Service-specific metrics
docker compose exec postgres pg_stat_activity
docker compose exec redis redis-cli info stats
```

#### Log Aggregation
```yaml
# Add to docker-compose files for centralized logging
logging:
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
    labels: "service={{.Name}}"
```

### Backup Procedures

#### Database Backup
```bash
# Create backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="./backups"
mkdir -p $BACKUP_DIR

# Database backup
docker compose exec postgres pg_dump -U tinyboards -d tinyboards | gzip > $BACKUP_DIR/postgres_$DATE.sql.gz

# Media files backup
tar -czf $BACKUP_DIR/media_$DATE.tar.gz volumes/media/

echo "Backup completed: $DATE"
```

#### Automated Backups
```bash
# Add to crontab for daily backups
0 2 * * * /path/to/tinyboards/docker/backup.sh
```

#### Restore Procedures
```bash
# Restore database
gunzip -c backups/postgres_20231201_020000.sql.gz | docker compose exec -T postgres psql -U tinyboards -d tinyboards

# Restore media files
tar -xzf backups/media_20231201_020000.tar.gz -C volumes/
```

### Performance Optimization

#### Database Tuning
```sql
-- Connect to PostgreSQL
\c tinyboards

-- Analyze query performance
EXPLAIN ANALYZE SELECT * FROM posts WHERE board_id = 1;

-- Update statistics
ANALYZE;

-- View slow queries
SELECT query, mean_time, calls FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;
```

#### Redis Optimization
```bash
# Connect to Redis
docker compose exec redis redis-cli

# Check memory usage
INFO memory

# View performance metrics
INFO stats
```

#### Nginx Caching
```nginx
# Add to nginx.conf for aggressive caching
location ~* \.(css|js|png|jpg|jpeg|gif|ico|svg)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

location /api/ {
    proxy_cache my_cache;
    proxy_cache_valid 200 5m;
    proxy_cache_use_stale error timeout updating http_500 http_502 http_503 http_504;
}
```

### Custom Extensions

#### Adding New Services
```yaml
# Example: Add Elasticsearch for search
elasticsearch:
  image: elasticsearch:8.5.0
  environment:
    - discovery.type=single-node
    - ES_JAVA_OPTS=-Xms512m -Xmx512m
  volumes:
    - elasticsearch_data:/usr/share/elasticsearch/data
  networks:
    - tinyboards
```

#### Custom Configuration Overlays
```bash
# Create docker-compose.override.yml for local customizations
# This file is automatically loaded and overrides base configuration
```

## 🔧 Maintenance

### Regular Maintenance Tasks

#### Daily Tasks
```bash
# Check service health
docker compose ps

# Monitor disk usage
docker system df

# Review logs for errors
docker compose logs --since=24h | grep -i error
```

#### Weekly Tasks
```bash
# Update images (test in staging first)
docker compose pull
docker compose up -d

# Clean up unused resources
docker system prune -f

# Backup data
./backup.sh
```

#### Monthly Tasks
```bash
# Full system cleanup
docker system prune -a -f

# Review and rotate logs
docker compose logs --since=720h > logs/archive_$(date +%Y%m).log

# Security updates
docker compose build --no-cache
docker compose up -d
```

### Update Procedures

#### Application Updates
```bash
# 1. Backup current state
./backup.sh

# 2. Pull latest images
docker compose pull

# 3. Recreate services
docker compose up -d --force-recreate

# 4. Verify functionality
curl -f http://localhost/ || echo "Update failed!"
```

#### Database Migrations
```bash
# Migrations are handled automatically by the backend service
# Monitor logs during startup for migration status
docker compose logs tinyboards | grep migration
```

#### Rollback Procedures
```bash
# Rollback to previous image version
docker tag tinyboards:latest tinyboards:backup
docker compose down
docker compose up -d

# If database rollback needed
docker compose down
rm -rf volumes/postgres_data
docker volume create postgres_data
# Restore from backup
```

### Security Maintenance

#### Certificate Renewal (Let's Encrypt)
```bash
# Renew certificates
sudo certbot renew

# Copy new certificates
sudo cp /etc/letsencrypt/live/your-domain.com/fullchain.pem /etc/ssl/
sudo cp /etc/letsencrypt/live/your-domain.com/privkey.pem /etc/ssl/

# Reload nginx
docker compose exec nginx nginx -s reload
```

#### Security Scanning
```bash
# Scan for vulnerabilities
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock aquasec/trivy image kronusdev/tinyboards-be:latest

# Update base images regularly
docker compose build --pull --no-cache
```

### Monitoring and Alerting

#### Health Check Automation
```bash
#!/bin/bash
# health-monitor.sh
SERVICES="tinyboards tinyboards-fe postgres nginx"

for service in $SERVICES; do
  if ! docker compose ps $service | grep -q "Up"; then
    echo "ALERT: Service $service is down"
    # Send notification (email, Slack, etc.)
  fi
done
```

#### Performance Monitoring
```bash
# Monitor resource usage
docker stats --no-stream > /var/log/docker-stats.log

# Database performance
echo "SELECT pg_stat_reset();" | docker compose exec -T postgres psql -U tinyboards -d tinyboards
```

#### Log Rotation
```bash
# Configure log rotation
cat > /etc/logrotate.d/docker-tinyboards << EOF
/var/lib/docker/containers/*/*.log {
    rotate 7
    daily
    compress
    size=1M
    missingok
    delaycompress
    copytruncate
}
EOF
```

## 🐳 DockerHub Publishing and CI/CD

### Building and Pushing Images

The repository includes comprehensive scripts for building and pushing Docker images to DockerHub with multi-platform support.

#### Build Script Features
- **Multi-platform builds**: Supports AMD64 and ARM64 architectures
- **Automated versioning**: Git tag detection and semantic versioning
- **CI/CD integration**: GitHub Actions compatible
- **Security scanning**: Vulnerability assessment with Trivy
- **Build optimization**: Layer caching and multi-stage builds

#### Manual Build and Push
```bash
# Build and push both images with latest tag
./docker/scripts/build-and-push.sh both

# Build specific version
./docker/scripts/build-and-push.sh -v v1.2.3 both

# Build backend only
./docker/scripts/build-and-push.sh backend

# Build frontend only
./docker/scripts/build-and-push.sh frontend

# Build without pushing (testing)
./docker/scripts/build-and-push.sh --no-push both

# Custom DockerHub username
./docker/scripts/build-and-push.sh -u myusername both
```

#### Setup for DockerHub Publishing

**1. Configure DockerHub Credentials**
```bash
# Set environment variables
export DOCKERHUB_USERNAME=yourusername
export DOCKERHUB_TOKEN=your_access_token

# Or login interactively
docker login
```

**2. Setup Multi-platform Builder**
```bash
# Setup buildx for multi-platform builds
./docker/scripts/build-and-push.sh setup-buildx

# Verify builder
docker buildx ls
```

**3. Build and Push Images**
```bash
# Full build and push workflow
./docker/scripts/build-and-push.sh login
./docker/scripts/build-and-push.sh both
```

### CI/CD Integration

#### GitHub Actions Workflows

The repository includes automated GitHub Actions workflows for continuous integration:

**Main Build Workflow (`.github/workflows/docker-build.yml`)**
- **Triggers**: Push to main/master, tags, PRs, manual dispatch
- **Features**: Multi-platform builds, security scanning, testing
- **Outputs**: Tagged images pushed to DockerHub

**Release Workflow (`.github/workflows/release.yml`)**
- **Triggers**: GitHub releases, manual dispatch
- **Features**: Semantic versioning, production builds
- **Outputs**: Versioned releases with latest tags

#### Setting Up CI/CD

**1. Configure Repository Secrets**
```bash
# Required GitHub secrets:
DOCKERHUB_TOKEN=your_dockerhub_access_token
```

**2. Trigger Builds**
```bash
# Automatic builds on:
git push origin main                    # Latest build
git tag v1.2.3 && git push --tags     # Version build

# Manual builds via GitHub Actions web interface
```

#### CI/CD Environment Variables
```bash
# Build configuration
DOCKERHUB_USERNAME=kronusdev           # DockerHub username
CI=true                               # CI mode flag
GITHUB_REF=refs/tags/v1.2.3          # Git reference
GITHUB_SHA=abc123def456               # Commit SHA

# Build options
PLATFORMS=linux/amd64,linux/arm64     # Target platforms
DOCKER_BUILDKIT=1                     # Enable BuildKit
```

### Image Tagging Strategy

#### Automatic Tagging
- **Latest**: Main/master branch builds → `latest`
- **Semantic**: Git tags `v1.2.3` → `1.2.3`, `1.2`, `1`, `latest`
- **Branch**: Feature branches → `branch-name`
- **PR**: Pull requests → `pr-123`

#### Manual Tagging
```bash
# Specific version
./docker/scripts/build-and-push.sh -v 1.2.3 both

# Development branch
./docker/scripts/build-and-push.sh -v dev-feature both

# Release candidate
./docker/scripts/build-and-push.sh -v 1.2.3-rc1 both
```

### Using Published Images

#### Docker Compose with Published Images
```yaml
# Use published images in production
services:
  tinyboards:
    image: kronusdev/tinyboards-be:latest
    # ... rest of configuration

  tinyboards-fe:
    image: kronusdev/tinyboards-fe:latest
    # ... rest of configuration
```

#### Pulling Specific Versions
```bash
# Pull specific version
docker pull kronusdev/tinyboards-be:1.2.3
docker pull kronusdev/tinyboards-fe:1.2.3

# Pull latest
docker pull kronusdev/tinyboards-be:latest
docker pull kronusdev/tinyboards-fe:latest

# Pull development branch
docker pull kronusdev/tinyboards-be:develop
docker pull kronusdev/tinyboards-fe:develop
```

#### Production Deployment with Published Images
```bash
# Update production environment to use specific version
sed -i 's/tinyboards-be:.*/tinyboards-be:1.2.3/' docker-compose.prod.yml
sed -i 's/tinyboards-fe:.*/tinyboards-fe:1.2.3/' docker-compose.prod.yml

# Deploy with version pinning
docker compose -f docker-compose.prod.yml up -d
```

### Security and Best Practices

#### Image Security
- **Non-root users**: All images run as non-privileged users
- **Minimal base images**: Alpine Linux for smaller attack surface
- **Vulnerability scanning**: Automated security scans with Trivy
- **Layer optimization**: Multi-stage builds for minimal production images

#### DockerHub Security
```bash
# Enable Docker Content Trust
export DOCKER_CONTENT_TRUST=1

# Sign images (requires Docker Notary)
docker trust sign kronusdev/tinyboards-be:1.2.3

# Verify signatures
docker trust inspect kronusdev/tinyboards-be:1.2.3
```

#### Access Control
```bash
# Use Docker Hub access tokens instead of passwords
# Generate token at: https://hub.docker.com/settings/security

# Limit token permissions to specific repositories
# Configure token for read/write access only to required repos
```

### Troubleshooting Publishing Issues

#### Build Problems
```bash
# Clean build cache
docker buildx prune -f

# Reset builder
docker buildx rm tinyboards-builder
./docker/scripts/build-and-push.sh setup-buildx

# Check build logs
docker buildx build --progress=plain .
```

#### Push Problems
```bash
# Verify login
docker login --username yourusername

# Check image tags
docker images | grep tinyboards

# Manual push
docker push kronusdev/tinyboards-be:latest
```

#### CI/CD Problems
```bash
# Check GitHub Actions logs
# Verify repository secrets are set
# Ensure DOCKERHUB_TOKEN has correct permissions

# Test locally with same environment
export CI=true
export GITHUB_REF=refs/tags/v1.2.3
./docker/scripts/build-and-push.sh both
```

#### Multi-platform Issues
```bash
# Check builder platforms
docker buildx inspect

# Test single platform
./docker/scripts/build-and-push.sh -p linux/amd64 both

# Emulation setup (for local ARM64 builds on AMD64)
docker run --privileged --rm tonistiigi/binfmt --install all
```

---

## 📞 Support and Contributing

### Getting Help
- **Documentation**: Check the main repository README and CLAUDE.md
- **Issues**: Open GitHub issues for bugs and feature requests
- **Community**: Join community discussions

### Contributing
- **Docker Improvements**: Submit PRs for Docker configuration enhancements
- **Documentation**: Help improve this guide
- **Testing**: Test deployment scenarios and report issues

### License
This Docker setup follows the same license as the main TinyBoards project.

---

*Last updated: 2024-09-19*
*Docker Compose Version: 3.8*
*Documentation Version: 1.0*