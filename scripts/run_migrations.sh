#!/usr/bin/env bash
set -euo pipefail

# Applies all SQL migration files in order.
# Tracks applied migrations in a migrations_run table to ensure idempotency.
# Safe to run multiple times — already-applied migrations are skipped.
#
# Usage: ./run_migrations.sh
#
# Requires DATABASE_URL to be set, or a .env file in /opt/tinyboards/.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Load .env if present
if [ -f "${PROJECT_DIR}/.env" ]; then
    # shellcheck disable=SC1091
    source "${PROJECT_DIR}/.env"
elif [ -f /opt/tinyboards/.env ]; then
    # shellcheck disable=SC1091
    source /opt/tinyboards/.env
fi

if [ -z "${DATABASE_URL:-}" ]; then
    echo "Error: DATABASE_URL is not set."
    echo "Set it in your environment or in a .env file."
    exit 1
fi

MIGRATIONS_DIR="${PROJECT_DIR}/migrations"

if [ ! -d "$MIGRATIONS_DIR" ]; then
    echo "Error: Migrations directory not found at ${MIGRATIONS_DIR}"
    exit 1
fi

# Create the tracking table if it doesn't exist
psql "$DATABASE_URL" -q <<'SQL'
CREATE TABLE IF NOT EXISTS migrations_run (
    filename TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
SQL

echo "Checking migrations..."

# Find all migration directories and process them in order
applied=0
skipped=0

for migration_dir in "$MIGRATIONS_DIR"/*/; do
    dirname="$(basename "$migration_dir")"
    up_file="${migration_dir}up.sql"

    if [ ! -f "$up_file" ]; then
        echo "Warning: No up.sql found in ${dirname}, skipping."
        continue
    fi

    # Check if this migration has already been applied
    already_applied=$(psql "$DATABASE_URL" -tAc \
        "SELECT COUNT(*) FROM migrations_run WHERE filename = '${dirname}'")

    if [ "$already_applied" -gt 0 ]; then
        echo "Skipping: ${dirname} (already applied)"
        skipped=$((skipped + 1))
        continue
    fi

    echo "Applying: ${dirname}..."

    # Run the migration inside a transaction
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 --single-transaction -q -f "$up_file"

    # Record successful application
    psql "$DATABASE_URL" -q -c \
        "INSERT INTO migrations_run (filename) VALUES ('${dirname}')"

    echo "  Done."
    applied=$((applied + 1))
done

echo ""
echo "Migrations complete: ${applied} applied, ${skipped} skipped."
