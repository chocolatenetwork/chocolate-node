
# HOSTNAME/PROJECT-ID/IMAGE@IMAGE-DIGEST
# Tag with this
# eu.gcr.io/united-option-342615/choc-dev/test:v1.0
# Dockerise
docker build /workspace/chocolate-node
# Change tag
docker tag $(docker images | awk '{print $3}' | awk 'NR==2') eu.gcr.io/united-option-342615/choc-dev/test
# Push to repo
docker push eu.gcr.io/united-option-342615/choc-dev/test

