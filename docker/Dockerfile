# Simplified Dockerfile without cargo-chef for debugging
ARG RUST_VERSION=1.82
ARG DEBIAN_VERSION=bookworm
ARG CACHE_BUST_SIMPLE=1

# Builder stage
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} as builder
ARG CACHE_BUST_SIMPLE=1

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /tmp/build

# Copy source code
COPY . .

# Remove any existing cargo config
RUN rm -rf .cargo

# Cache bust to force rebuild
RUN echo "Building with cache bust: $CACHE_BUST_SIMPLE"

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd --gid 10001 tinyboards && \
    useradd --uid 10001 --gid tinyboards --create-home --shell /sbin/nologin tinyboards

# Create application directory
RUN mkdir -p /opt/tinyboards/config /opt/tinyboards/media && \
    chown -R tinyboards:tinyboards /opt/tinyboards

# Copy binary
COPY --from=builder --chown=tinyboards:tinyboards /tmp/build/target/release/tinyboards_server /usr/local/bin/

# Set up runtime environment
USER tinyboards
WORKDIR /opt/tinyboards

# Expose port
EXPOSE 8536

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8536/api/v1/site || exit 1

# Start the application
CMD ["tinyboards_server"]