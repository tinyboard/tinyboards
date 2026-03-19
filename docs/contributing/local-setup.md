# Local Development Setup

Step-by-step guide to get TinyBoards running on your local machine for development.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Step 1: Clone the Repository](#step-1-clone-the-repository)
- [Step 2: PostgreSQL](#step-2-postgresql)
- [Step 3: Backend](#step-3-backend)
- [Step 4: Frontend](#step-4-frontend)
- [Step 5: Verify Everything Works](#step-5-verify-everything-works)
- [Development Workflow](#development-workflow)

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | 1.75+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | 20+ | [nodesource.com](https://github.com/nodesource/distributions) |
| pnpm | 8+ | `npm install -g pnpm` |
| PostgreSQL | 15+ | `sudo apt install postgresql-16` |
| cargo-watch | Latest | `cargo install cargo-watch` (optional, for auto-reload) |

## Step 1: Clone the Repository

```bash
git clone https://github.com/tinyboard/tinyboards-rewrite.git
cd tinyboards-rewrite
```

## Step 2: PostgreSQL

### Create the Database

```bash
sudo -u postgres psql <<SQL
CREATE USER tinyboards WITH PASSWORD 'tinyboards_dev';
CREATE DATABASE tinyboards_dev OWNER tinyboards;
GRANT ALL PRIVILEGES ON DATABASE tinyboards_dev TO tinyboards;
\c tinyboards_dev
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
SQL
```

### Verify Connection

```bash
psql -U tinyboards -h localhost -d tinyboards_dev -c "SELECT 1;"
```

If you get a password authentication error, check `pg_hba.conf` and ensure `md5` or `scram-sha-256` auth is configured for local connections.

## Step 3: Backend

### Configure Environment

```bash
cp tinyboards.example.hjson tinyboards.hjson
```

Edit `tinyboards.hjson` with development values:

```hjson
database: {
  password: "tinyboards_dev"
  host: "localhost"
  database: "tinyboards_dev"
}
hostname: "localhost"
salt_suffix: "dev_salt_suffix_not_for_production"
environment: "dev"
media: {
  media_path: "./media"
}
storage: {
  fs: {
    root: "./media"
  }
}
frontend: {
  internal_api_host: "http://localhost:8536"
}
```

### Build and Run

```bash
cd backend

# First build (takes a few minutes)
cargo build

# Run the server (applies migrations automatically)
cargo run
```

The backend starts at `http://localhost:8536`.

Verify:

```bash
curl http://localhost:8536/
# Should return: ok
```

### Auto-Reload During Development

```bash
cargo watch -x run
```

This rebuilds and restarts the server when Rust files change.

## Step 4: Frontend

### Install Dependencies

```bash
cd frontend
pnpm install
```

### Configure

The frontend reads from the same `tinyboards.hjson` configuration file as the backend. No separate frontend `.env` is needed. If you followed the backend configuration step above, the frontend is already configured via `frontend.internal_api_host` in `tinyboards.hjson`.

### Start Development Server

```bash
pnpm dev
```

The frontend starts at `http://localhost:3000` with hot module replacement.

## Step 5: Verify Everything Works

1. Open `http://localhost:3000` in your browser.
2. You should see the TinyBoards homepage (or initial setup wizard on first run).
3. Test the GraphQL endpoint:

```bash
curl -X POST http://localhost:8536/api/v2/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ site { name } }"}'
```

## Development Workflow

### Running Backend and Frontend Together

Open two terminals:

```bash
# Terminal 1: Backend
cd backend && cargo watch -x run

# Terminal 2: Frontend
cd frontend && pnpm dev
```

### Resetting the Database

```bash
# Drop and recreate
sudo -u postgres psql -c "DROP DATABASE tinyboards_dev;"
sudo -u postgres psql -c "CREATE DATABASE tinyboards_dev OWNER tinyboards;"
sudo -u postgres psql -d tinyboards_dev -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"
sudo -u postgres psql -d tinyboards_dev -c "CREATE EXTENSION IF NOT EXISTS pg_trgm;"

# Restart backend to re-run migrations
cargo run
```

### Viewing Logs

Backend logs are controlled by the `RUST_LOG` environment variable:

```bash
# Verbose
RUST_LOG=debug cargo run

# Specific modules
RUST_LOG=info,tinyboards_api=debug cargo run
```

### Accessing the Database

```bash
psql -U tinyboards -h localhost -d tinyboards_dev
```

Useful queries:

```sql
-- List all tables
\dt

-- Check migration status
SELECT * FROM __diesel_schema_migrations ORDER BY run_on DESC LIMIT 10;

-- Count users
SELECT COUNT(*) FROM users;
```
