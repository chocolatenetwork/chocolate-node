#!/usr/bin/bash

# Run docker build with tag
TAG=islami00/choc-test
docker build -f .github/dockerfiles/Dockerfile.ubuntu-base -t $TAG .
#   give option for USER and GROUP
# Grab dockerimage hash.
TAGGED_HASH=$(docker inspect --format='{{index .RepoDigests 0}}' $TAG) 
#   Combine image hash with name
SED_ME="s+%%DOCKER_IMAGE%%+${TAGGED_HASH}+g"
#   Replace DOCKER_IMAGE in docker-compose.yml.template
cat .github/dockerfiles/docker-compose.yml.template | sed $SED_ME  > docker-compose.yml &&
#   Make commit with tag release
git add docker-compose.yml && git commit -m "release: New image hash"
# Push to repo
docker push $TAG
