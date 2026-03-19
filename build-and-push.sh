#!/bin/bash

# Build and push tinyboards Docker images to Docker Hub.
# Usage: ./build-and-push.sh [tag]
#
# Examples:
#   ./build-and-push.sh              # pushes with tag "latest"
#   ./build-and-push.sh v1.2.0       # pushes with tag "v1.2.0"
#   ./build-and-push.sh v1.2.0 latest # pushes with both tags

set -euo pipefail

DOCKERHUB_ORG="tinyboard"
BACKEND_IMAGE="$DOCKERHUB_ORG/tinyboards-backend"
FRONTEND_IMAGE="$DOCKERHUB_ORG/tinyboards-frontend"

TAGS=("${@:-latest}")

echo "Building and pushing tinyboards images..."
echo "  Backend:  $BACKEND_IMAGE"
echo "  Frontend: $FRONTEND_IMAGE"
echo "  Tags:     ${TAGS[*]}"
echo ""

# Build tag arguments for docker build (-t for each tag)
backend_tag_args=()
frontend_tag_args=()
for tag in "${TAGS[@]}"; do
    backend_tag_args+=(-t "$BACKEND_IMAGE:$tag")
    frontend_tag_args+=(-t "$FRONTEND_IMAGE:$tag")
done

# Build backend
echo "--- Building backend ---"
docker build --pull --no-cache \
    "${backend_tag_args[@]}" \
    -f backend.Dockerfile \
    .

echo ""

# Build frontend
echo "--- Building frontend ---"
docker build --pull --no-cache \
    "${frontend_tag_args[@]}" \
    -f frontend.Dockerfile \
    .

echo ""

# Push all tags
echo "--- Pushing images ---"
for tag in "${TAGS[@]}"; do
    echo "Pushing $BACKEND_IMAGE:$tag"
    docker push "$BACKEND_IMAGE:$tag"

    echo "Pushing $FRONTEND_IMAGE:$tag"
    docker push "$FRONTEND_IMAGE:$tag"
done

echo ""
echo "Done. Pushed:"
for tag in "${TAGS[@]}"; do
    echo "  $BACKEND_IMAGE:$tag"
    echo "  $FRONTEND_IMAGE:$tag"
done
