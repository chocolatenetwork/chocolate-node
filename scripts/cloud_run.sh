#!/usr/bin/bash
set -euo pipefail

docker login
DATE=$(date +"%d%m%Y")
TAGGEDREPO="islami00/choc-test:$DATE"

cargo build --release
docker build -f .github/dockerfiles/Dockerfile.ubuntu-base -t $TAGGEDREPO .
#   give option for USER and GROUP
SED_ME="s+%%DOCKER_IMAGE%%+$TAGGEDREPO+g"

cat .github/dockerfiles/docker-compose.yml.template | sed $SED_ME  > docker-compose.yml &&

git add docker-compose.yml && git commit -m "release: New image hash"

docker push $TAGGEDREPO
