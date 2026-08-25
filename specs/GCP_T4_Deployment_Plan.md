# HoloEngine 3D — Plan de Déploiement GCP (NVIDIA T4 Remote GPU)

Ce document résume le plan de déploiement Cloud pour exécuter le moteur **HoloEngine 3D** et ses *Compute Shaders WGSL* sur une instance Compute Engine Google Cloud (GCP) équipée d'un GPU **NVIDIA Tesla T4**.

---

## ☁️ 1. Spécifications de la VM GCP T4

- **Instance Name** : `holo-engine-t4-node`
- **Zone** : `us-central1-a` (ou `europe-west4-a`)
- **Type de VM** : `n1-standard-4` (4 vCPUs, 15 GB RAM)
- **Accélérateur GPU** : 1x NVIDIA Tesla T4 (16 GB VRAM)
- **Disque système** : 100 GB SSD (`pd-ssd`)
- **OS Base** : Ubuntu 22.04 LTS

---

## 🛠️ 2. Scripts et Fichiers de Déploiement Créés

1. **Dockerfile Container WebGPU Headless** : `specs/deploy/Dockerfile.gcp_t4`
   - Image de base : `nvidia/cuda:12.2.2-devel-ubuntu22.04`
   - Pilotes Vulkan (`libvulkan-dev`, `vulkan-tools`, `mesa-vulkan-drivers`) et toolchain Rust.
   - Compilation automatique release de `poc2_runner` et `amcp_agent_daemon`.

2. **Script CLI d'Instanciation GCP** : `specs/deploy/deploy_gcp_t4.sh`
   - Commande `gcloud compute instances create` préconfigurée avec les pilotes NVIDIA 535 et Vulkan SDK en startup-script.

---

## 🚀 3. Étapes de Déploiement (Commandes CLI)

### Étape A : Créer l'instance sur GCP
```bash
bash specs/deploy/deploy_gcp_t4.sh
```

### Étape B : Se connecter et vérifier le GPU & Vulkan
```bash
gcloud compute ssh holo-engine-t4-node --zone=us-central1-a
nvidia-smi
vulkaninfo | head -n 20
```

### Étape C : Cloner et exécuter le conteneur Docker GPU
```bash
git clone https://github.com/xaviercallens/Universcraft.git
cd Universcraft
docker build -t holo-engine-gcp -f specs/deploy/Dockerfile.gcp_t4 .
docker run --gpus all -p 8080:8080 -p 9090:9090 holo-engine-gcp
```

---

## 📊 4. Invariants & Performances Attendues sur T4

- **Débit WGPU Compute Shader (DONN / SDF)** : Évaluation simultanée sur 32 768 threads WGSL en **< 1.2 ms**.
- **Framerate Cible** : **+60 à 120 FPS constants** sans goulot d'étranglement CPU.
- **Support AMCP Agent Daemon** : Synchronisation réseau maillée sur port `9090`.
