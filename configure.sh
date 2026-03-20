#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# configure.sh — Generate .env from tinyboards.hjson
#
# tinyboards.hjson is the single source of truth for the stack.
# This script extracts the few values docker-compose needs (postgres
# credentials, domain, logging) and writes them to .env.
#
# Usage:
#   ./configure.sh                          # uses ./tinyboards.hjson
#   ./configure.sh /path/to/tinyboards.hjson
# ---------------------------------------------------------------------------
set -euo pipefail

CONFIG="${1:-./tinyboards.hjson}"

if [[ ! -f "$CONFIG" ]]; then
  echo "Error: config file not found: $CONFIG"
  echo "Copy the example first:"
  echo "  cp tinyboards.example.hjson tinyboards.hjson"
  exit 1
fi

# ---------------------------------------------------------------------------
# Strip hjson comments and extract values using a simple state machine.
# Tracks the current section (top-level brace-delimited block) so we can
# distinguish e.g. database.user from a top-level "user" key.
# ---------------------------------------------------------------------------

# Clean the file: strip line comments (# and //) that aren't inside quotes.
# This is "good enough" for the simple hjson files we generate.
cleaned=$(sed -E '
  s|^\s*#.*||
  s|^\s*//.*||
  s|\s+#[^"]*$||
  s|\s+//[^"]*$||
' "$CONFIG")

# Extract a value from a specific section.
# Usage: extract_from <section> <key>
extract_from() {
  local section="$1"
  local key="$2"
  local in_section=false

  while IFS= read -r line; do
    # Detect section start
    if [[ "$line" =~ ^[[:space:]]*${section}[[:space:]]*:[[:space:]]*\{ ]]; then
      in_section=true
      continue
    fi

    if $in_section; then
      # Detect section end
      if [[ "$line" =~ ^[[:space:]]*\} ]]; then
        in_section=false
        continue
      fi
      # Match key: "value" or key: value
      if [[ "$line" =~ ^[[:space:]]*${key}[[:space:]]*:[[:space:]]*\"([^\"]+)\" ]]; then
        echo "${BASH_REMATCH[1]}"
        return
      elif [[ "$line" =~ ^[[:space:]]*${key}[[:space:]]*:[[:space:]]*([^[:space:],}\"]+) ]]; then
        echo "${BASH_REMATCH[1]}"
        return
      fi
    fi
  done <<< "$cleaned"
}

# Extract a top-level value (not inside any named section).
# Usage: extract_top <key>
# Note: hjson wraps everything in outer {}. Top-level keys are at depth 1.
# Named sections (e.g. "database: {") push to depth 2. We match at depth 1.
extract_top() {
  local key="$1"
  local depth=0

  while IFS= read -r line; do
    # Lines that open a named section ("key: {") increase depth
    if [[ "$line" =~ \{ ]]; then
      ((depth++)) || true
      # If this is a named section (not the outer brace), skip the line
      if [[ "$line" =~ ^[[:space:]]*[a-zA-Z_]+[[:space:]]*:[[:space:]]*\{ ]]; then
        continue
      fi
      # Outer brace — continue to next line
      continue
    fi
    if [[ "$line" =~ \} ]]; then
      ((depth--)) || true
      continue
    fi

    # Top-level keys live at depth 1 (inside the outer {})
    # but NOT inside a named section (depth 2+)
    if [[ $depth -eq 1 ]]; then
      if [[ "$line" =~ ^[[:space:]]*${key}[[:space:]]*:[[:space:]]*\"([^\"]+)\" ]]; then
        echo "${BASH_REMATCH[1]}"
        return
      elif [[ "$line" =~ ^[[:space:]]*${key}[[:space:]]*:[[:space:]]*([^[:space:],}\"]+) ]]; then
        echo "${BASH_REMATCH[1]}"
        return
      fi
    fi
  done <<< "$cleaned"
}

# ---------------------------------------------------------------------------
# Extract values from hjson
# ---------------------------------------------------------------------------
DB_USER=$(extract_from "database" "user")
DB_PASS=$(extract_from "database" "password")
DB_NAME=$(extract_from "database" "database")
HOSTNAME=$(extract_top "hostname")
TLS_ENABLED=$(extract_top "tls_enabled")
USE_HTTPS=$(extract_from "frontend" "use_https")
RUST_LOG=$(extract_from "logging" "rust_log")
IMAGE_TAG=$(extract_from "docker" "image_tag")
COMPOSE_PROJECT=$(extract_from "docker" "compose_project_name")

# Determine protocol
if [[ "${USE_HTTPS:-false}" == "true" ]] || [[ "${TLS_ENABLED:-false}" == "true" ]]; then
  PROTOCOL="https"
  HTTPS_FLAG="true"
else
  PROTOCOL="http"
  HTTPS_FLAG="false"
fi

# ---------------------------------------------------------------------------
# Generate nginx configuration
# ---------------------------------------------------------------------------
DOMAIN="${HOSTNAME:-localhost}"

if [[ "${HTTPS_FLAG}" == "true" ]]; then
  NGINX_CONF="nginx/ssl.conf"
  # Generate ssl.conf from the template, substituting the domain.
  # Only DOMAIN is replaced — nginx variables ($host, $scheme, etc.) are preserved.
  sed "s/\${DOMAIN}/${DOMAIN}/g" nginx/ssl.conf.template > nginx/ssl.conf
  echo "Generated nginx/ssl.conf for domain: ${DOMAIN}"
else
  NGINX_CONF="nginx/default.conf"
  echo "Using HTTP-only nginx config (TLS not enabled)"
fi

# ---------------------------------------------------------------------------
# Write .env
# ---------------------------------------------------------------------------
cat > .env <<EOF
# Auto-generated by configure.sh from tinyboards.hjson — do not edit directly.
# Re-run ./configure.sh after changing tinyboards.hjson.

# PostgreSQL container credentials (synced from tinyboards.hjson database section)
POSTGRES_USER=${DB_USER:-tinyboards}
POSTGRES_PASSWORD=${DB_PASS}
POSTGRES_DB=${DB_NAME:-tinyboards}

# Domain
DOMAIN=${DOMAIN}
USE_HTTPS=${HTTPS_FLAG}
TLS_ENABLED=${HTTPS_FLAG}

# Nginx config file (default.conf for HTTP, ssl.conf for HTTPS)
NGINX_CONF=${NGINX_CONF}

# Frontend public URLs
NUXT_PUBLIC_SITE_URL=${PROTOCOL}://${DOMAIN}
NUXT_PUBLIC_DOMAIN=${DOMAIN}
NUXT_PUBLIC_USE_HTTPS=${HTTPS_FLAG}

# Logging
RUST_LOG=${RUST_LOG:-info}

# Docker
TINYBOARDS_TAG=${IMAGE_TAG:-latest}
COMPOSE_PROJECT_NAME=${COMPOSE_PROJECT:-tinyboards}
EOF

echo ""
echo "Generated .env from $CONFIG"
echo "  POSTGRES_USER=${DB_USER:-tinyboards}"
echo "  POSTGRES_DB=${DB_NAME:-tinyboards}"
echo "  DOMAIN=${DOMAIN}"
echo "  TLS_ENABLED=${HTTPS_FLAG}"
echo "  NGINX_CONF=${NGINX_CONF}"
echo ""
echo "Run: docker compose up -d"
