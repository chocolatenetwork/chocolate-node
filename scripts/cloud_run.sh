#!/usr/bin/bash

# Run docker build with tag
TAG=islami00/choc-test
docker build -f .github/dockerfiles/Dockerfile.ubuntu-base -t $TAG .
#   give option for USER and GROUP
# Grab dockerimage hash.
HASH=$(docker images | awk '{print $3}' | awk 'NR==2')
#   Combine image hash with name
IMAGE="${TAG}@${HASH}" 
#   Replace DOCKER_IMAGE in docker-compose.yml.template
cat .github/dockerfiles/docker-compose.yml.template | sed "s/%%DOCKER_IMAGE%%/${IMAGE}/g" > docker-compose.yml
#   Make commit with tag release
git add docker-compose.yml && git commit -m "release: New image hash"
# Push to repo
docker push $TAG
