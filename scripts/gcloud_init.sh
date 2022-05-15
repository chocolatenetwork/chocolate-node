#!/usr/bin/bash
# Install deps
sudo apt-get install apt-transport-https ca-certificates gnupg -y
# add gcloud pkg
echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | sudo tee -a /etc/apt/sources.list.d/google-cloud-sdk.list
curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key --keyring /usr/share/keyrings/cloud.google.gpg add -
# cli
sudo apt-get update && sudo apt-get install google-cloud-sdk -y
# Init
gcloud init
gcloud auth configure-docker



set -e
set -u
# set -o pipefail


DOCKER_TAG=${1:-}
GCP_PROJECT=${2:-}
if [[ -z "$DOCKER_TAG" || -z "$GCP_PROJECT" ]]; then
    echo "Args not supplied: Docker_tag, GCP_PROJECT"
    exit 1
fi

# Pull the image (I did it on Cloud Shell)
docker pull $DOCKER_TAG

# Tag the image
docker tag $DOCKER_TAG "gcr.io/$GCP_PROJECT/$DOCKER_TAG"

#Push the image (no authentication issue on Cloud Shell)
docker push "gcr.io/$GCP_PROJECT/$DOCKER_TAG"
