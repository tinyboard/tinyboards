#!/bin/sh
IMAGE_NAME="kronusdev/tinyboards-be"
IMAGE_TAG=$(git rev-parse --short HEAD) # first 7 characters of current commit

echo "Building Docker image ${IMAGE_NAME}:${IMAGE_TAG}, and tagging as latest"
docker build -f ./docker/Dockerfile -t "${IMAGE_NAME}:${IMAGE_TAG}" .
docker tag "${IMAGE_NAME}:${IMAGE_TAG}" "${IMAGE_NAME}:latest"

echo "Authenticating and pushing image to Docker Hub"
# SET DOCKER_USERNAME and DOCKER_PASSWORD IN YOUR ENVIRONMENT VARIABLES (your docker creds)
docker login -u "${DOCKER_USERNAME}" -p "${DOCKER_PASSWORD}"
docker push "${IMAGE_NAME}:${IMAGE_TAG}"
docker push "${IMAGE_NAME}:latest"

docker image prune -af
