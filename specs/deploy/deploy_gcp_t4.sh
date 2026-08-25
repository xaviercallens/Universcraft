#!/bin/bash
set -e

# GCP NVIDIA T4 Instance Provisioning Script for HoloEngine GPU Pipeline
INSTANCE_NAME="holo-engine-t4-node"
ZONE="us-central1-a"
MACHINE_TYPE="n1-standard-4"
ACCELERATOR="type=nvidia-tesla-t4,count=1"
IMAGE_FAMILY="ubuntu-2204-lts"
IMAGE_PROJECT="ubuntu-os-cloud"

echo "🚀 Provisioning GCP Compute Instance with NVIDIA T4 GPU..."
gcloud compute instances create ${INSTANCE_NAME} \
    --zone=${ZONE} \
    --machine-type=${MACHINE_TYPE} \
    --accelerator=${ACCELERATOR} \
    --image-family=${IMAGE_FAMILY} \
    --image-project=${IMAGE_PROJECT} \
    --boot-disk-size=100GB \
    --boot-disk-type=pd-ssd \
    --maintenance-policy=TERMINATE \
    --metadata=startup-script='#!/bin/bash
    sudo apt-get update
    sudo apt-get install -y linux-headers-$(uname -r)
    curl -s -L https://nvidia.github.io/libnvidia-container/gpgkey | sudo apt-key add -
    sudo apt-get install -y nvidia-driver-535 nvidia-utils-535 vulkan-tools libvulkan-dev
    '

echo "✅ Instance ${INSTANCE_NAME} created successfully in ${ZONE}."
echo "🔗 Connect via SSH: gcloud compute ssh ${INSTANCE_NAME} --zone=${ZONE}"
