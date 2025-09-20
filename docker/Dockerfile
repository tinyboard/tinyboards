# Multi-stage Dockerfile optimized for production with cargo-chef
# Features: Minimal image size, security hardening, fast startup, efficient dependency caching

ARG RUST_VERSION=1.82
ARG DEBIAN_VERSION=bookworm

# Chef stage - installs cargo-chef for dependency caching
FROM rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} as chef

# Install build dependencies and cargo-chef
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    clang \
    lld \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean \
    && cargo install cargo-chef

# Create non-root user for building
RUN groupadd --gid 10001 builder && \
    useradd --uid 10001 --gid builder --create-home builder

# Set up cargo home directory for builder user
ENV CARGO_HOME=/home/builder/.cargo
RUN mkdir -p /home/builder/.cargo && \
    chown -R builder:builder /home/builder/.cargo

USER builder
WORKDIR /home/builder/build

# Planner stage - generates recipe.json for dependency caching
FROM chef AS planner
COPY --chown=builder:builder . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage - builds dependencies and application
FROM chef AS builder

# Set up cargo home directory for builder user
ENV CARGO_HOME=/home/builder/.cargo
RUN mkdir -p /home/builder/.cargo && \
    chown -R builder:builder /home/builder/.cargo

# Configure cargo for optimized production builds
RUN mkdir -p .cargo && \
    echo '[net]' > .cargo/config.toml && \
    echo 'git-fetch-with-cli = true' >> .cargo/config.toml && \
    echo '' >> .cargo/config.toml && \
    echo '[profile.release]' >> .cargo/config.toml && \
    echo 'opt-level = 3' >> .cargo/config.toml && \
    echo 'lto = true' >> .cargo/config.toml && \
    echo 'codegen-units = 1' >> .cargo/config.toml && \
    echo 'panic = "abort"' >> .cargo/config.toml && \
    echo 'strip = true' >> .cargo/config.toml && \
    echo '' >> .cargo/config.toml && \
    echo '[target.x86_64-unknown-linux-gnu]' >> .cargo/config.toml && \
    echo 'linker = "clang"' >> .cargo/config.toml && \
    echo 'rustflags = ["-C", "link-arg=-fuse-ld=lld", "-C", "target-cpu=native"]' >> .cargo/config.toml

# Build dependencies (this layer is cached unless dependencies change)
COPY --from=planner --chown=builder:builder /home/builder/build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code and build application
COPY --chown=builder:builder . .

# Handle potential Cargo.lock version mismatch
RUN if ! cargo tree > /dev/null 2>&1; then \
        echo "Regenerating Cargo.lock due to version mismatch..." && \
        rm -f Cargo.lock && \
        cargo generate-lockfile; \
    fi

# Build the optimized binary
RUN cargo build --release

# Runtime stage with minimal dependencies
FROM debian:${DEBIAN_VERSION}-slim as runtime

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for runtime
RUN groupadd --gid 10001 tinyboards && \
    useradd --uid 10001 --gid tinyboards --create-home --shell /sbin/nologin tinyboards

# Create application directory and set ownership
RUN mkdir -p /opt/tinyboards/{config,media} && \
    chown -R tinyboards:tinyboards /opt/tinyboards

# Copy binary and configuration
COPY --from=builder --chown=tinyboards:tinyboards /home/builder/build/target/release/tinyboards_server /usr/local/bin/
COPY --from=builder --chown=tinyboards:tinyboards /home/builder/build/config /opt/tinyboards/config/

# Set up runtime environment
USER tinyboards
WORKDIR /opt/tinyboards

# Expose application port
EXPOSE 8536

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8536/api/v1/site || exit 1

# Labels for metadata
LABEL org.opencontainers.image.title="TinyBoards Backend"
LABEL org.opencontainers.image.description="Backend server for TinyBoards social platform"
LABEL org.opencontainers.image.vendor="TinyBoards"
LABEL org.opencontainers.image.source="https://github.com/tinyboards/tinyboards"

# Start the application
CMD ["tinyboards_server"]