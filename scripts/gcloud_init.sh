# Install deps
sudo apt-get install apt-transport-https ca-certificates gnupg
# add gcloud pkg
echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | sudo tee -a /etc/apt/sources.list.d/google-cloud-sdk.list
curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo apt-key --keyring /usr/share/keyrings/cloud.google.gpg add -
# cli
sudo apt-get update && sudo apt-get install google-cloud-sdk
# Init
gcloud init
gcloud auth configure-docker