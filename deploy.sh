#!/bin/bash

# Build and push script for TinyBoards Backend
# Usage: ./deploy.sh [dockerhub-username] [tag]

set -e

# Configuration
DOCKERHUB_USER=${1:-"kronusdev"}
TAG=${2:-"latest"}
IMAGE_NAME="tinyboards-be"
FULL_IMAGE_NAME="$DOCKERHUB_USER/$IMAGE_NAME:$TAG"

echo "🚀 Building and pushing TinyBoards Backend to Docker Hub..."
echo "   Image: $FULL_IMAGE_NAME"

# Build the image
echo "🔨 Building Docker image..."
echo "📋 Using Dockerfile at docker/Dockerfile"
docker build \
    -f docker/Dockerfile \
    -t $IMAGE_NAME \
    -t $FULL_IMAGE_NAME .

# Login to Docker Hub (if not already logged in)
echo "🔐 Logging into Docker Hub..."
docker login

# Push to Docker Hub
echo "📤 Pushing to Docker Hub..."
docker push $FULL_IMAGE_NAME

echo "✅ Successfully pushed $FULL_IMAGE_NAME to Docker Hub!"
echo ""
echo "🚀 To deploy on your server:"
echo "   docker pull $FULL_IMAGE_NAME"
echo "   docker run -d -p 8536:8536 \\"
echo "     -e DATABASE_URL=postgresql://user:pass@host:5432/tinyboards \\"
echo "     -e TINYBOARDS_CONFIG_FILE=/opt/tinyboards/config.hjson \\"
echo "     --name tinyboards-be \\"
echo "     $FULL_IMAGE_NAME"
echo ""
echo "📝 Required Environment Variables:"
echo "   DATABASE_URL: PostgreSQL connection string"
echo "   TINYBOARDS_CONFIG_FILE: Path to config file (optional)"
echo ""
echo "💡 For a complete production setup with database, nginx, etc.:"
echo "   Use docker/docker-compose.prod.yml with docker/scripts/deploy.sh"
echo ""
echo "🔧 Make sure to:"
echo "   - Have PostgreSQL database running and accessible"
echo "   - Run database migrations: diesel migration run"
echo "   - Mount config files if needed"