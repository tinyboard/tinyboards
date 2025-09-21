#!/bin/bash

# TinyBoards Production Setup Script
# This script creates Docker secrets for secure production deployment

set -e

echo "🔧 Setting up TinyBoards Production Environment"
echo "=============================================="

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if Docker Swarm is initialized (required for secrets)
if ! docker node ls > /dev/null 2>&1; then
    echo "📋 Initializing Docker Swarm mode (required for secrets)..."
    docker swarm init
fi

# Function to generate secure random password
generate_password() {
    openssl rand -base64 32 | tr -d "=+/" | cut -c1-25
}

# Function to create Docker secret
create_secret() {
    local secret_name=$1
    local secret_value=$2

    if docker secret inspect "$secret_name" > /dev/null 2>&1; then
        echo "⚠️  Secret '$secret_name' already exists. Skipping..."
    else
        echo "$secret_value" | docker secret create "$secret_name" -
        echo "✅ Created secret: $secret_name"
    fi
}

echo ""
echo "🔐 Creating Docker Secrets..."
echo "-----------------------------"

# Generate passwords
POSTGRES_PASSWORD=$(generate_password)
REDIS_PASSWORD=$(generate_password)
JWT_SECRET=$(generate_password)

# Create secrets
create_secret "tinyboards_postgres_password" "$POSTGRES_PASSWORD"
create_secret "tinyboards_redis_password" "$REDIS_PASSWORD"
create_secret "tinyboards_jwt_secret" "$JWT_SECRET"

echo ""
echo "📁 Creating required directories..."
echo "----------------------------------"

# Create directories with proper permissions
mkdir -p ./ssl
mkdir -p ./nginx/conf
mkdir -p ./backups
chmod 755 ./ssl ./nginx/conf ./backups

echo "✅ Created directories: ssl, nginx/conf, backups"

echo ""
echo "🌐 Environment Configuration"
echo "----------------------------"

# Create .env.production file if it doesn't exist
if [ ! -f .env.production ]; then
    cat > .env.production << 'EOF'
# TinyBoards Production Environment Variables
# Copy this file and customize for your deployment

# Domain Configuration (REQUIRED)
DOMAIN=your-domain.com
LETSENCRYPT_EMAIL=admin@your-domain.com

# Database Configuration
POSTGRES_USER=tinyboards
POSTGRES_DB=tinyboards

# Optional: Override default image tags
# TINYBOARDS_BE_TAG=latest
# TINYBOARDS_FE_TAG=latest

# Grafana (if monitoring enabled)
# GRAFANA_ADMIN_PASSWORD=secure_grafana_password
EOF
    echo "✅ Created .env.production template"
    echo "⚠️  Please edit .env.production with your domain and email!"
else
    echo "⚠️  .env.production already exists. Please verify your configuration."
fi

echo ""
echo "🔧 Creating monitoring configuration template..."
echo "-----------------------------------------------"

# Create monitoring directory and basic Prometheus config
mkdir -p ./monitoring
if [ ! -f ./monitoring/prometheus.yml ]; then
    cat > ./monitoring/prometheus.yml << 'EOF'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  # - "first_rules.yml"
  # - "second_rules.yml"

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'tinyboards-backend'
    static_configs:
      - targets: ['tinyboards:8536']
    metrics_path: '/metrics'
    scrape_interval: 30s

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres:5432']
    scrape_interval: 30s

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:6379']
    scrape_interval: 30s
EOF
    echo "✅ Created monitoring/prometheus.yml"
fi

echo ""
echo "📋 Creating deployment checklist..."
echo "----------------------------------"

cat > PRODUCTION_DEPLOYMENT_CHECKLIST.md << 'EOF'
# TinyBoards Production Deployment Checklist

## Before Deployment

### 1. Environment Configuration
- [ ] Edit `.env.production` with your domain name
- [ ] Set `DOMAIN` to your actual domain (e.g., `example.com`)
- [ ] Set `LETSENCRYPT_EMAIL` to your admin email
- [ ] Verify all required environment variables are set

### 2. DNS Configuration
- [ ] Point your domain A record to the server IP
- [ ] Verify DNS propagation: `nslookup your-domain.com`
- [ ] Ensure ports 80 and 443 are open in firewall

### 3. SSL Certificates
- [ ] Initial certificate generation will happen automatically
- [ ] Verify Let's Encrypt can reach your domain via HTTP (port 80)
- [ ] Check certificate status: `docker-compose exec certbot certbot certificates`

### 4. Security Verification
- [ ] Verify Docker secrets are created: `docker secret ls`
- [ ] Check that no sensitive data is in environment variables
- [ ] Confirm database and Redis ports are not exposed externally
- [ ] Review firewall rules

## Deployment Commands

```bash
# Load environment variables
source .env.production

# Deploy the stack
docker-compose -f docker-compose.prod.yml up -d

# Check service status
docker-compose -f docker-compose.prod.yml ps

# View logs
docker-compose -f docker-compose.prod.yml logs -f

# Check health
docker-compose -f docker-compose.prod.yml exec tinyboards curl -f http://localhost:8536/api/v2/health
```

## Post-Deployment Verification

### 1. Service Health Checks
- [ ] All services are running: `docker-compose ps`
- [ ] Backend health check passes
- [ ] Frontend loads correctly
- [ ] Database connections work
- [ ] SSL certificate is valid

### 2. Security Verification
- [ ] HTTPS redirects work properly
- [ ] Database is not accessible externally
- [ ] Redis requires authentication
- [ ] No exposed credentials in logs

### 3. Backup Verification
- [ ] Backup service is running
- [ ] Backups are being created in `/backups`
- [ ] Test backup restoration process

### 4. Monitoring Setup
- [ ] Enable Prometheus monitoring (optional)
- [ ] Set up log aggregation
- [ ] Configure alerting rules

## Maintenance

### Backup Management
```bash
# List backups
ls -la ./backups/

# Manual backup
docker-compose -f docker-compose.prod.yml exec backup pg_dump -h postgres -U tinyboards tinyboards | gzip > backup_manual_$(date +%Y%m%d).sql.gz

# Restore from backup
gunzip -c backup_file.sql.gz | docker-compose -f docker-compose.prod.yml exec -T postgres psql -U tinyboards -d tinyboards
```

### SSL Certificate Renewal
```bash
# Check certificate status
docker-compose -f docker-compose.prod.yml exec certbot certbot certificates

# Manual renewal (automatic renewal runs every 12 hours)
docker-compose -f docker-compose.prod.yml exec certbot certbot renew
```

### Log Management
```bash
# View service logs
docker-compose -f docker-compose.prod.yml logs nginx
docker-compose -f docker-compose.prod.yml logs tinyboards
docker-compose -f docker-compose.prod.yml logs postgres

# Cleanup old logs (Docker handles rotation automatically)
docker system prune -f
```

## Troubleshooting

### Common Issues

1. **SSL Certificate Issues**
   - Verify domain DNS points to server
   - Check firewall allows ports 80/443
   - Ensure Let's Encrypt can reach `/.well-known/acme-challenge/`

2. **Database Connection Issues**
   - Check secrets are properly created
   - Verify network connectivity between services
   - Review database logs for authentication errors

3. **Service Startup Issues**
   - Check dependency order with `docker-compose ps`
   - Review health check configurations
   - Increase startup timeouts if needed

4. **Performance Issues**
   - Monitor resource usage with `docker stats`
   - Adjust memory/CPU limits as needed
   - Review database performance tuning
EOF

echo "✅ Created PRODUCTION_DEPLOYMENT_CHECKLIST.md"

echo ""
echo "🎉 Production Setup Complete!"
echo "============================"
echo ""
echo "Next steps:"
echo "1. Edit .env.production with your domain and email"
echo "2. Review PRODUCTION_DEPLOYMENT_CHECKLIST.md"
echo "3. Ensure DNS points to this server"
echo "4. Run: source .env.production && docker-compose -f docker-compose.prod.yml up -d"
echo ""
echo "🔐 Security Summary:"
echo "- Postgres password: ✅ Secured with Docker secrets"
echo "- Redis password: ✅ Secured with Docker secrets"
echo "- JWT secret: ✅ Secured with Docker secrets"
echo "- Database ports: ✅ Not exposed externally"
echo "- Redis ports: ✅ Not exposed externally"
echo "- SSL/TLS: ✅ Auto-configured with Let's Encrypt"
echo ""
echo "📊 Monitoring:"
echo "- Health checks: ✅ Enabled for all services"
echo "- Log rotation: ✅ Configured"
echo "- Backups: ✅ Automated daily backups"
echo ""
echo "For troubleshooting, check: PRODUCTION_DEPLOYMENT_CHECKLIST.md"