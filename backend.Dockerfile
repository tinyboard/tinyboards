# Stage 1: Build the backend binary
FROM rust:1.88-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests first for dependency caching
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/crates/utils/Cargo.toml crates/utils/Cargo.toml
COPY backend/crates/db/Cargo.toml crates/db/Cargo.toml
COPY backend/crates/api/Cargo.toml crates/api/Cargo.toml
COPY backend/crates/auth/Cargo.toml crates/auth/Cargo.toml

# Create dummy source files so cargo can resolve and cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs \
    && mkdir -p crates/utils/src && echo "" > crates/utils/src/lib.rs \
    && mkdir -p crates/db/src && echo "" > crates/db/src/lib.rs \
    && mkdir -p crates/api/src && echo "" > crates/api/src/lib.rs \
    && mkdir -p crates/auth/src && echo "" > crates/auth/src/lib.rs

# Build dependencies only (this layer is cached until Cargo.toml/Cargo.lock change)
RUN cargo build --release --locked 2>/dev/null || true

# Remove dummy artifacts so the real source gets compiled
RUN rm -rf src crates target/release/.fingerprint/tinyboards*

# Copy actual source code
COPY backend/src ./src
COPY backend/crates ./crates
COPY backend/diesel.toml ./diesel.toml

# Copy migrations (diesel.toml points to ../migrations relative to /build)
COPY migrations /migrations

# Build the real binary
RUN cargo build --release --locked

# Strip debug symbols to reduce binary size
RUN strip target/release/tinyboards_server

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Run as non-root
RUN groupadd -g 1001 tinyboards \
    && useradd -u 1001 -g tinyboards -s /usr/sbin/nologin -M tinyboards

WORKDIR /app

COPY --from=builder /build/target/release/tinyboards_server ./tinyboards

RUN mkdir -p /app/media && chown -R tinyboards:tinyboards /app

USER tinyboards

EXPOSE 8536

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:8536/ || exit 1

ENTRYPOINT ["./tinyboards"]
