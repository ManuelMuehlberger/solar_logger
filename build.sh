#!/bin/bash

VERSION="0.1.0"
IMAGE_NAME="sbnhndrt/solarlogger"

# Build the Docker image
docker build \
  --build-arg GIT_USERNAME="${GIT_USERNAME}" \
  --build-arg GIT_PASSWORD="${GIT_PASSWORD}" \
  -t "${IMAGE_NAME}:${VERSION}" \
  -t "${IMAGE_NAME}:latest" \
  .

# Push the image to Docker Hub
docker push "${IMAGE_NAME}:${VERSION}"
docker push "${IMAGE_NAME}:latest"