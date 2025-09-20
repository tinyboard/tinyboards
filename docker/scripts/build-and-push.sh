#!/bin/bash

# TinyBoards DockerHub Build and Push Script
# This script builds and pushes both backend and frontend images to DockerHub
# Supports multi-platform builds, versioning, and automated CI/CD integration

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCKER_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$DOCKER_DIR")"
FRONTEND_ROOT="$(dirname "$PROJECT_ROOT")/tinyboards-fe"

# DockerHub configuration
DOCKERHUB_USERNAME="${DOCKERHUB_USERNAME:-kronusdev}"
BACKEND_REPO="${DOCKERHUB_USERNAME}/tinyboards-be"
FRONTEND_REPO="${DOCKERHUB_USERNAME}/tinyboards-fe"

# Build configuration
DEFAULT_VERSION="latest"
PLATFORMS="linux/amd64,linux/arm64"
DOCKERFILE_BACKEND="docker/Dockerfile"
DOCKERFILE_FRONTEND="$FRONTEND_ROOT/docker/Dockerfile"

# CI/CD detection
IS_CI="${CI:-false}"
GITHUB_REF="${GITHUB_REF:-}"
GITHUB_SHA="${GITHUB_SHA:-}"

# Functions
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

log_build() {
    echo -e "${PURPLE}[BUILD]${NC} $1"
}

show_usage() {
    cat << EOF
Usage: $0 [OPTIONS] COMMAND

Commands:
    backend             Build and push backend image only
    frontend            Build and push frontend image only
    both                Build and push both images (default)
    setup-buildx        Setup Docker buildx for multi-platform builds
    login               Login to DockerHub
    test                Test built images locally
    clean               Clean up build cache and temporary images

Options:
    -v, --version TAG   Set image version/tag (default: latest)
    -u, --username USER Set DockerHub username (default: kronusdev)
    -p, --platforms STR Set build platforms (default: linux/amd64,linux/arm64)
    --no-cache         Build without using cache
    --dry-run          Show commands without executing
    --push             Push images after building (default: true)
    --no-push          Build only, don't push
    --single-platform  Build for current platform only (faster for testing)
    --latest           Also tag as latest (automatic for version tags)
    -h, --help         Show this help message

Examples:
    # Build and push both images with latest tag
    $0 both

    # Build specific version
    $0 -v v1.2.3 both

    # Build backend only without pushing
    $0 --no-push backend

    # Setup for CI/CD
    $0 setup-buildx && $0 login && $0 both

    # Test builds locally
    $0 --no-push both && $0 test

Environment Variables:
    DOCKERHUB_USERNAME  DockerHub username (default: kronusdev)
    DOCKERHUB_TOKEN     DockerHub access token (for CI/CD)
    DOCKER_BUILDKIT     Enable BuildKit (default: 1)
    CI                  Set to 'true' for CI/CD mode
    GITHUB_REF          GitHub ref for automatic versioning
    GITHUB_SHA          GitHub commit SHA for metadata

EOF
}

check_requirements() {
    log_info "Checking requirements..."

    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi

    # Check Docker version
    docker_version=$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo "unknown")
    log_info "Docker version: $docker_version"

    # Check if buildx is available
    if ! docker buildx version &> /dev/null; then
        log_error "Docker buildx is not available. Please update Docker or run 'setup-buildx' command"
        exit 1
    fi

    # Check if we can access Docker daemon
    if ! docker info &> /dev/null; then
        log_error "Cannot access Docker daemon. Please check Docker is running and user has permissions"
        exit 1
    fi

    # Enable BuildKit
    export DOCKER_BUILDKIT=1
    export DOCKER_CLI_EXPERIMENTAL=enabled

    # Ensure we have a buildx builder available
    if ! docker buildx ls | grep -q "tinyboards-builder"; then
        log_info "Setting up buildx builder..."
        setup_buildx
    else
        # Use existing builder
        docker buildx use tinyboards-builder 2>/dev/null || docker buildx use default
    fi

    log_success "Requirements check passed"
}

setup_buildx() {
    log_info "Setting up Docker buildx for multi-platform builds..."

    # Create buildx builder if it doesn't exist
    if ! docker buildx ls | grep -q tinyboards-builder; then
        log_info "Creating new buildx builder..."
        docker buildx create --name tinyboards-builder --driver docker-container --bootstrap
    fi

    # Use the builder
    docker buildx use tinyboards-builder

    # Inspect builder capabilities
    log_info "Builder capabilities:"
    docker buildx inspect --bootstrap

    log_success "Buildx setup completed"
}

dockerhub_login() {
    log_info "Logging into DockerHub..."

    if [[ -n "${DOCKERHUB_TOKEN:-}" ]]; then
        # Use token for CI/CD
        echo "$DOCKERHUB_TOKEN" | docker login --username "$DOCKERHUB_USERNAME" --password-stdin
        log_success "Logged in using token"
    elif [[ "$IS_CI" == "true" ]]; then
        log_error "DOCKERHUB_TOKEN environment variable required for CI/CD"
        exit 1
    else
        # Interactive login
        log_info "Please enter your DockerHub credentials:"
        docker login --username "$DOCKERHUB_USERNAME"
        log_success "Logged in interactively"
    fi
}

determine_version() {
    local version="$1"

    # If version is not provided, try to determine from git or CI
    if [[ "$version" == "latest" ]]; then
        if [[ -n "$GITHUB_REF" ]]; then
            if [[ "$GITHUB_REF" =~ refs/tags/v?([0-9]+\.[0-9]+\.[0-9]+.*) ]]; then
                version="${BASH_REMATCH[1]}"
            elif [[ "$GITHUB_REF" =~ refs/heads/(.+) ]]; then
                branch="${BASH_REMATCH[1]}"
                if [[ "$branch" == "main" || "$branch" == "master" ]]; then
                    version="latest"
                else
                    version="$branch"
                fi
            fi
        elif git rev-parse --git-dir > /dev/null 2>&1; then
            # Try to get version from git tag
            if git_version=$(git describe --tags --exact-match 2>/dev/null); then
                version="${git_version#v}"
            elif git_branch=$(git branch --show-current 2>/dev/null); then
                if [[ "$git_branch" == "main" || "$git_branch" == "master" ]]; then
                    version="latest"
                else
                    version="$git_branch"
                fi
            fi
        fi
    fi

    # Clean the version string to ensure it's a valid Docker tag
    # Remove any invalid characters and ensure it follows Docker tag rules
    version=$(echo "$version" | sed 's/[^a-zA-Z0-9._-]//g' | sed 's/^[.-]*//' | sed 's/[.-]*$//')

    # If version becomes empty, default to latest
    if [[ -z "$version" ]]; then
        version="latest"
    fi

    echo "$version"
}

generate_build_args() {
    local build_args=""

    # Add build metadata
    if [[ -n "${GITHUB_SHA:-}" ]]; then
        build_args="$build_args --build-arg GIT_SHA=$GITHUB_SHA"
    elif git rev-parse HEAD &>/dev/null; then
        local git_sha=$(git rev-parse HEAD)
        build_args="$build_args --build-arg GIT_SHA=$git_sha"
    fi

    # Add build timestamp
    build_args="$build_args --build-arg BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ')"

    # Add version
    build_args="$build_args --build-arg VERSION=$VERSION"

    echo "$build_args"
}

check_frontend_repo() {
    if [[ ! -d "$FRONTEND_ROOT" ]]; then
        log_error "Frontend repository not found at: $FRONTEND_ROOT"
        log_info "Please clone tinyboards-fe repository alongside tinyboards"
        exit 1
    fi

    if [[ ! -f "$FRONTEND_ROOT/package.json" ]]; then
        log_error "Frontend package.json not found"
        exit 1
    fi

    log_success "Frontend repository check passed"
}

create_frontend_dockerfile() {
    log_info "Creating optimized Dockerfile for frontend..."

    mkdir -p "$(dirname "$DOCKERFILE_FRONTEND")"

    cat > "$DOCKERFILE_FRONTEND" << 'EOF'
# Multi-stage build for Nuxt.js frontend
ARG NODE_VERSION=18

FROM node:${NODE_VERSION}-alpine AS base
WORKDIR /app

# Build stage
FROM base AS builder

# Build arguments
ARG VERSION=latest
ARG BUILD_DATE
ARG GIT_SHA

# Labels
LABEL org.opencontainers.image.title="TinyBoards Frontend"
LABEL org.opencontainers.image.description="TinyBoards Frontend - Social media platform frontend"
LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.created="${BUILD_DATE}"
LABEL org.opencontainers.image.revision="${GIT_SHA}"
LABEL org.opencontainers.image.vendor="TinyBoards"
LABEL org.opencontainers.image.source="https://github.com/tinyboards/tinyboards-fe"

# Install dependencies
COPY package*.json ./
RUN npm ci --only=production --no-audit --no-fund && npm cache clean --force

# Copy source code
COPY . .

# Build the application
ENV NODE_ENV=production
RUN npm run build

# Production stage
FROM node:${NODE_VERSION}-alpine AS runtime

# Security: Create non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nuxt -u 1001 && \
    apk add --no-cache curl

WORKDIR /app

# Copy built application with correct ownership
COPY --from=builder --chown=nuxt:nodejs /app/.output /app/.output

# Switch to non-root user
USER nuxt

# Expose port
EXPOSE 3000

# Environment variables
ENV NODE_ENV=production
ENV NUXT_HOST=0.0.0.0
ENV NUXT_PORT=3000
ENV NODE_OPTIONS="--max-old-space-size=512"

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Start the application
CMD ["node", ".output/server/index.mjs"]
EOF

    log_success "Frontend Dockerfile created"
}

build_backend() {
    local version="$1"
    local push="$2"
    local no_cache="$3"
    local dry_run="$4"

    log_build "Building backend image: $BACKEND_REPO:$version"

    cd "$PROJECT_ROOT"

    # Check if Dockerfile exists
    if [[ ! -f "$DOCKERFILE_BACKEND" ]]; then
        log_error "Backend Dockerfile not found: $DOCKERFILE_BACKEND"
        exit 1
    fi

    # Prepare build arguments array
    local build_args_array=()

    # Add build metadata
    if [[ -n "${GITHUB_SHA:-}" ]]; then
        build_args_array+=("--build-arg" "GIT_SHA=$GITHUB_SHA")
    elif git rev-parse HEAD &>/dev/null; then
        local git_sha=$(git rev-parse HEAD)
        build_args_array+=("--build-arg" "GIT_SHA=$git_sha")
    fi

    # Add build timestamp
    build_args_array+=("--build-arg" "BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ')")

    # Add version
    build_args_array+=("--build-arg" "VERSION=$version")

    # Prepare platform
    local platform="$PLATFORMS"
    if [[ "$push" == "false" ]] && [[ "$PLATFORMS" == *","* ]]; then
        log_warning "Multi-platform build detected with --no-push. Building for single platform only."
        platform="linux/amd64"
    fi

    # Build the command array
    local cmd_array=(
        "docker" "buildx" "build"
        "--platform" "$platform"
        "--file" "$DOCKERFILE_BACKEND"
        "--tag" "$BACKEND_REPO:$version"
    )

    # Add latest tag for version releases
    if [[ "$version" != "latest" ]] && [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
        cmd_array+=("--tag" "$BACKEND_REPO:latest")
    fi

    # Add build args
    cmd_array+=("${build_args_array[@]}")

    # Add cache option
    if [[ "$no_cache" == "true" ]]; then
        cmd_array+=("--no-cache")
    fi

    # Add push or load option
    if [[ "$push" == "true" ]]; then
        cmd_array+=("--push")
    else
        cmd_array+=("--load")
    fi

    # Add build context using git archive to avoid volume permission issues
    cmd_array+=("-")

    # Execute or show command
    if [[ "$dry_run" == "true" ]]; then
        log_info "Backend build command (dry run):"
        printf '%q ' "${cmd_array[@]}"
        echo " < <(git archive --format=tar HEAD)"
    else
        log_info "Executing backend build..."
        log_info "Build command: ${cmd_array[*]} < <(git archive --format=tar HEAD)"

        # Use git archive to create clean build context without volume directories
        if git archive --format=tar HEAD | "${cmd_array[@]}"; then
            log_success "Backend build completed"
        else
            log_error "Backend build failed"
            exit 1
        fi
    fi
}

build_frontend() {
    local version="$1"
    local push="$2"
    local no_cache="$3"
    local dry_run="$4"

    log_build "Building frontend image: $FRONTEND_REPO:$version"

    check_frontend_repo

    cd "$FRONTEND_ROOT"

    # Create Dockerfile if it doesn't exist
    if [[ ! -f "$DOCKERFILE_FRONTEND" ]]; then
        create_frontend_dockerfile
    fi

    # Prepare build arguments array
    local build_args_array=()

    # Add build metadata
    if [[ -n "${GITHUB_SHA:-}" ]]; then
        build_args_array+=("--build-arg" "GIT_SHA=$GITHUB_SHA")
    elif git rev-parse HEAD &>/dev/null; then
        local git_sha=$(git rev-parse HEAD)
        build_args_array+=("--build-arg" "GIT_SHA=$git_sha")
    fi

    # Add build timestamp
    build_args_array+=("--build-arg" "BUILD_DATE=$(date -u +'%Y-%m-%dT%H:%M:%SZ')")

    # Add version
    build_args_array+=("--build-arg" "VERSION=$version")

    # Prepare platform
    local platform="$PLATFORMS"
    if [[ "$push" == "false" ]] && [[ "$PLATFORMS" == *","* ]]; then
        log_warning "Multi-platform build detected with --no-push. Building for single platform only."
        platform="linux/amd64"
    fi

    # Build the command array
    local cmd_array=(
        "docker" "buildx" "build"
        "--platform" "$platform"
        "--file" "$DOCKERFILE_FRONTEND"
        "--tag" "$FRONTEND_REPO:$version"
    )

    # Add latest tag for version releases
    if [[ "$version" != "latest" ]] && [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
        cmd_array+=("--tag" "$FRONTEND_REPO:latest")
    fi

    # Add build args
    cmd_array+=("${build_args_array[@]}")

    # Add cache option
    if [[ "$no_cache" == "true" ]]; then
        cmd_array+=("--no-cache")
    fi

    # Add push or load option
    if [[ "$push" == "true" ]]; then
        cmd_array+=("--push")
    else
        cmd_array+=("--load")
    fi

    # Add build context
    cmd_array+=(".")

    # Execute or show command
    if [[ "$dry_run" == "true" ]]; then
        log_info "Frontend build command (dry run):"
        printf '%q ' "${cmd_array[@]}"
        echo
    else
        log_info "Executing frontend build..."
        log_info "Build command: ${cmd_array[*]}"

        if "${cmd_array[@]}"; then
            log_success "Frontend build completed"
        else
            log_error "Frontend build failed"
            exit 1
        fi
    fi
}

test_images() {
    local version="$1"

    log_info "Testing built images locally..."

    # Test backend image
    log_info "Testing backend image: $BACKEND_REPO:$version"
    if docker run --rm "$BACKEND_REPO:$version" --version; then
        log_success "Backend image test passed"
    else
        log_warning "Backend image test failed or not available locally"
    fi

    # Test frontend image
    log_info "Testing frontend image: $FRONTEND_REPO:$version"
    if docker run --rm -d --name test-frontend -p 3001:3000 "$FRONTEND_REPO:$version"; then
        sleep 5
        if curl -f http://localhost:3001/health > /dev/null 2>&1; then
            log_success "Frontend image test passed"
        else
            log_warning "Frontend health check failed"
        fi
        docker stop test-frontend > /dev/null 2>&1 || true
    else
        log_warning "Frontend image test failed or not available locally"
    fi
}

clean_build_cache() {
    log_info "Cleaning build cache and temporary images..."

    # Clean buildx cache
    docker buildx prune -f

    # Clean general cache
    docker system prune -f

    # Remove dangling images
    docker image prune -f

    log_success "Cleanup completed"
}

show_build_summary() {
    local version="$1"
    local command="$2"

    log_success "Build Summary"
    echo "============================================="
    echo "Command: $command"
    echo "Version: $version"
    echo "Platform: $PLATFORMS"
    echo "DockerHub User: $DOCKERHUB_USERNAME"
    echo ""

    case "$command" in
        "backend"|"both")
            echo "Backend Image: $BACKEND_REPO:$version"
            if [[ "$version" != "latest" ]] && [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
                echo "Backend Latest: $BACKEND_REPO:latest"
            fi
            ;;
    esac

    case "$command" in
        "frontend"|"both")
            echo "Frontend Image: $FRONTEND_REPO:$version"
            if [[ "$version" != "latest" ]] && [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
                echo "Frontend Latest: $FRONTEND_REPO:latest"
            fi
            ;;
    esac

    echo "============================================="
}

# Default values
VERSION="$DEFAULT_VERSION"
NO_CACHE="false"
DRY_RUN="false"
PUSH="true"
COMMAND="both"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -u|--username)
            DOCKERHUB_USERNAME="$2"
            BACKEND_REPO="${DOCKERHUB_USERNAME}/tinyboards-be"
            FRONTEND_REPO="${DOCKERHUB_USERNAME}/tinyboards-fe"
            shift 2
            ;;
        -p|--platforms)
            PLATFORMS="$2"
            shift 2
            ;;
        --no-cache)
            NO_CACHE="true"
            shift
            ;;
        --dry-run)
            DRY_RUN="true"
            shift
            ;;
        --push)
            PUSH="true"
            shift
            ;;
        --no-push)
            PUSH="false"
            shift
            ;;
        --single-platform)
            PLATFORMS="linux/amd64"
            shift
            ;;
        --latest)
            # This is handled automatically for version tags
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        backend|frontend|both|setup-buildx|login|test|clean)
            COMMAND="$1"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Determine final version
VERSION=$(determine_version "$VERSION")
log_info "Using version: $VERSION"

# Main execution
case "$COMMAND" in
    "setup-buildx")
        check_requirements
        setup_buildx
        ;;
    "login")
        dockerhub_login
        ;;
    "backend")
        check_requirements
        if [[ "$PUSH" == "true" ]] && [[ "$DRY_RUN" == "false" ]]; then
            dockerhub_login
        fi
        build_backend "$VERSION" "$PUSH" "$NO_CACHE" "$DRY_RUN"
        show_build_summary "$VERSION" "$COMMAND"
        ;;
    "frontend")
        check_requirements
        if [[ "$PUSH" == "true" ]] && [[ "$DRY_RUN" == "false" ]]; then
            dockerhub_login
        fi
        build_frontend "$VERSION" "$PUSH" "$NO_CACHE" "$DRY_RUN"
        show_build_summary "$VERSION" "$COMMAND"
        ;;
    "both")
        check_requirements
        if [[ "$PUSH" == "true" ]] && [[ "$DRY_RUN" == "false" ]]; then
            dockerhub_login
        fi
        build_backend "$VERSION" "$PUSH" "$NO_CACHE" "$DRY_RUN"
        build_frontend "$VERSION" "$PUSH" "$NO_CACHE" "$DRY_RUN"
        show_build_summary "$VERSION" "$COMMAND"
        ;;
    "test")
        test_images "$VERSION"
        ;;
    "clean")
        clean_build_cache
        ;;
    *)
        log_error "Unknown command: $COMMAND"
        show_usage
        exit 1
        ;;
esac

log_success "Script completed successfully!"