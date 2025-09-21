# TinyBoards 

> A modern, self-hosted social media platform built with Rust and GraphQL

[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.82+-orange.svg)](https://www.rust-lang.org/)
[![GraphQL](https://img.shields.io/badge/GraphQL-API-E10098.svg)](https://graphql.org/)

## About TinyBoards

TinyBoards is a standalone social media platform similar to Reddit, Hacker News, and Lemmy. Users can:

- 📋 **Subscribe to boards** - Join topic-based communities
- 📝 **Post content** - Share links, text posts, and images
- 💬 **Engage in discussions** - Comment and reply in nested threads
- ⬆️ **Vote on content** - Upvote and downvote posts and comments
- 👤 **Manage profiles** - Customize user profiles and settings
- 🔒 **Moderate communities** - Board-level and site-level moderation tools

### Key Features

- **Self-hosted** - Run your own instance with full control
- **Modern Tech Stack** - Built with Rust, GraphQL, and PostgreSQL
- **Real-time Features** - Notifications, messaging, and live updates
- **Comprehensive API** - Full GraphQL API for integrations
- **Flexible Permissions** - Role-based access control
- **Media Support** - Image and file upload capabilities
- **Rate Limiting** - Built-in protection against abuse
- **Admin Tools** - Comprehensive site administration

### Why TinyBoards?

- 🏠 **Self-hosted** - Own your data and community
- 🎨 **Customizable** - Tailor the platform to your needs
- 🔓 **Open Source** - Transparent and community-driven
- 🚫 **Ad-free** - No corporate control or advertisements
- ⚡ **Performance** - Built with Rust for speed and reliability
- 🔌 **API-first** - Easy integration and automation



## Quick Start

### Prerequisites

Before setting up TinyBoards, ensure you have:

- **Rust** (latest stable) - [Install from rustup.rs](https://rustup.rs/)
- **PostgreSQL** (12+) - [Download here](https://www.postgresql.org/download/)
- **System dependencies**:
  - Ubuntu/Debian: `sudo apt install libpq-dev gcc pkg-config`
  - RHEL/CentOS: `sudo yum install postgresql-devel gcc pkgconfig`
  - macOS: `brew install postgresql gcc`

### Local Development Setup

#### 1. Clone the Repository
```bash
git clone <repository-url>
cd tinyboards
```

#### 2. Database Setup
```bash
# Connect to PostgreSQL
sudo -u postgres psql

# Create database and user
CREATE DATABASE tinyboards;
CREATE USER tinyboards WITH PASSWORD 'tinyboards';
GRANT ALL PRIVILEGES ON DATABASE tinyboards TO tinyboards;
\c tinyboards
GRANT ALL ON SCHEMA public TO tinyboards;
\q
```

#### 3. Environment Configuration
```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export DATABASE_URL=postgresql://tinyboards:tinyboards@localhost:5432/tinyboards

# Reload your shell or run:
source ~/.bashrc
```

#### 4. Install Diesel CLI
```bash
cargo install diesel_cli --no-default-features --features postgres
```

#### 5. Run Migrations
```bash
diesel migration run
```

#### 6. Build and Run
```bash
# Build the project
cargo build

# Run the development server
cargo run
```

#### 7. Verify Installation
- Server runs at: `http://localhost:8536`
- GraphQL playground: `http://localhost:8536/graphql`
- Check logs for any errors

### Common Issues

<details>
<summary><strong>Database Connection Errors</strong></summary>

- Verify PostgreSQL is running: `sudo systemctl status postgresql`
- Check DATABASE_URL is set: `echo $DATABASE_URL`
- Test connection manually: `psql $DATABASE_URL`
</details>

<details>
<summary><strong>Build Failures</strong></summary>

- Update Rust: `rustup update`
- Clear cache: `cargo clean`
- Install missing dependencies (see Prerequisites)
</details>

<details>
<summary><strong>Migration Errors</strong></summary>

- Check database permissions
- Verify DATABASE_URL format
- Reset database: `diesel database reset`
</details>



## Production Docker Deployment

### Prerequisites
- VPS or server (Ubuntu 20.04+ recommended)
- Domain name pointed to your server (optional but recommended)

#### Installing Docker Prerequisites

**Ubuntu/Debian:**
```bash
# Update package index
sudo apt update

# Install required packages
sudo apt install apt-transport-https ca-certificates curl gnupg lsb-release

# Add Docker's official GPG key
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# Add Docker repository
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker Engine
sudo apt update
sudo apt install docker-ce docker-ce-cli containerd.io

# Install Docker Compose (standalone)
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Add your user to docker group (optional, to run docker without sudo)
sudo usermod -aG docker $USER
newgrp docker

# Verify installation
docker --version
docker-compose --version
```

**CentOS/RHEL/Rocky Linux:**
```bash
# Install required packages
sudo yum install -y yum-utils

# Add Docker repository
sudo yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo

# Install Docker Engine
sudo yum install docker-ce docker-ce-cli containerd.io

# Install Docker Compose (standalone)
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Start and enable Docker service
sudo systemctl start docker
sudo systemctl enable docker

# Add your user to docker group (optional)
sudo usermod -aG docker $USER
newgrp docker

# Verify installation
docker --version
docker-compose --version
```

### Docker Setup

#### 1. Download Docker Files
```bash
# Download the production docker-compose file
wget https://raw.githubusercontent.com/tinyboard/tinyboards/master/docker/docker-compose.prod.yml

# Create required directories
mkdir -p nginx/conf volumes/media volumes/postgres

# Download NGINX configuration
wget -O nginx/conf/nginx.conf https://raw.githubusercontent.com/tinyboard/tinyboards/master/docker/nginx/conf/nginx.conf
```

#### 2. Configure Environment Variables
Create a `.env` file with your production settings:
```bash
# Create .env file
nano .env
```

Add the following required environment variables:
```env
# Database Configuration (REQUIRED)
POSTGRES_PASSWORD=your_very_secure_database_password_here
POSTGRES_USER=tinyboards
POSTGRES_DB=tinyboards
POSTGRES_HOST=postgres
POSTGRES_PORT=5432

# Security Configuration (REQUIRED)
JWT_SECRET=your_super_secret_jwt_key_min_32_chars_long_random_string
REDIS_PASSWORD=your_secure_redis_password_here

# Domain Configuration (REQUIRED for production)
DOMAIN=your-domain.com
LETSENCRYPT_EMAIL=admin@your-domain.com

# Optional Configuration
NODE_ENV=production
RUST_LOG=info
NUXT_PUBLIC_USE_HTTPS=true
NUXT_PUBLIC_DOMAIN=your-domain.com

# Container Images (optional - uses latest by default)
TINYBOARDS_IMAGE=kronusdev/tinyboards-be:latest
TINYBOARDS_FE_IMAGE=kronusdev/tinyboards-fe:latest
```

**Important Security Notes:**
- Use strong, unique passwords for `POSTGRES_PASSWORD` and `REDIS_PASSWORD`
- Generate a random JWT secret with at least 32 characters
- Replace `your-domain.com` with your actual domain name
- Keep your `.env` file secure and never commit it to version control

#### 3. Deploy
```bash
# Start production services
docker-compose -f docker-compose.prod.yml up -d
```

#### 5. Verify Deployment
```bash
# Check container status
docker-compose ps

# View logs
docker-compose logs -f tinyboards

# Test connectivity
curl http://localhost:8536/api/v2/graphql
```

### Docker Management

```bash
# Stop services
docker-compose down

# Update and restart
docker-compose pull
docker-compose up -d

# View logs
docker-compose logs -f [service-name]

# Database backup
docker-compose exec postgres pg_dump -U tinyboards tinyboards > backup.sql
```

## Testing

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test package
cargo test -p tinyboards_api
cargo test -p tinyboards_db

# Run with output
cargo test -- --nocapture
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for security issues
cargo audit
```

## Development

### Project Structure
```
tinyboards/
├── crates/
│   ├── api/           # GraphQL API layer
│   ├── db/            # Database models and operations
│   └── utils/         # Shared utilities
├── migrations/        # Database schema migrations
├── config/           # Configuration files
├── docker/           # Docker deployment files
└── docs/             # Documentation
```

### Available Commands
```bash
# Development server with auto-reload
cargo run

# Build for production
cargo build --release

# Database operations
diesel migration generate <name>  # Create new migration
diesel migration run              # Apply migrations
diesel migration revert           # Rollback last migration

# Docker development
docker-compose up -d              # Start services
docker-compose logs -f            # View logs
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run the test suite: `cargo test`
5. Format your code: `cargo fmt`
6. Run clippy: `cargo clippy`
7. Commit your changes: `git commit -am 'Add feature'`
8. Push to the branch: `git push origin feature-name`
9. Submit a pull request

### Code Style
- Follow Rust conventions and `cargo fmt` formatting
- Add documentation for public APIs
- Include tests for new functionality
- Update documentation when adding features

## License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Support

- 📚 **Documentation**: Check the [`docs/`](docs/) directory
- 🐛 **Bug Reports**: Open an issue on GitHub
- 💬 **Discussions**: Use GitHub Discussions for questions
- 📧 **Security Issues**: Email security concerns privately

---

## Documentation

For comprehensive documentation, see the [`docs/`](docs/) directory:

### GraphQL API Documentation
- **[GraphQL API Guide](docs/api/GRAPHQL_API_GUIDE.md)** - Complete integration guide with practical examples
- **[GraphQL Quick Reference](docs/api/GRAPHQL_QUICK_REFERENCE.md)** - Concise reference for all operations
- **[Authentication & Security](docs/api/API_AUTHENTICATION_SECURITY.md)** - JWT authentication and security features
- **[Data Models](docs/api/DATA_MODELS.md)** - Entity relationships and database schema

### API Endpoints
- **GraphQL API**: `http://localhost:8536/api/v2/graphql`
- **GraphQL Playground**: `http://localhost:8536/graphql` (development)

### Quick API Example
```javascript
// Login and get user data
const response = await fetch('http://localhost:8536/api/v2/graphql', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    query: `
      mutation Login($username: String!, $password: String!) {
        login(usernameOrEmail: $username, password: $password) {
          token
        }
      }
    `,
    variables: { username: "your_username", password: "your_password" }
  })
});
```

---
