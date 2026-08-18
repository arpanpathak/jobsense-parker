# Arpan Pathak

Seattle, WA | Email: arpan.pathak47@gmail.com | Phone: +1 (206) 306-6059  
LinkedIn: [linkedin.com/in/arpan-pathak-272341424](https://linkedin.com/in/arpan-pathak-272341424) | GitHub: [github.com/arpanpathak](https://github.com/arpanpathak)  
YouTube: [youtube.com/@ArpanPathak](https://www.youtube.com/@ArpanPathak)  
**Visa**: Requires H1B Transfer sponsorship

---

## SUMMARY

Senior Software Engineer, **8 years**: **AI inference serving**, **backend microservices**, cloud-native infrastructure at **Amazon, Microsoft, Oracle, Razorpay**. Strong **Python**, **modern memory-safe C++**, **Rust**, **CUDA**; **BERT inference at 5,000 QPS** (ONNX, INT8) on **Docker/Kubernetes** with CI/CD; passionate about **CV/DL for road safety** on **NVIDIA Jetson Orin** (TensorRT).

---

## PERSONAL PROJECTS

**Driving-CivicSense: Full-Stack Edge AI (Rust, CUDA), NVIDIA Jetson Orin Nano Super NX** | [github](https://github.com/arpanpathak/driving-civicsense-vision-model)

- **Full-stack data collection**: **50,000 miles** of driving footage across all weather and terrain via vehicle camera sensors and **LiDAR**; simulation-driven synthetic data; privacy-first, on-device processing on edge neural chips.
- **Multi-chip edge system**: **YOLOv8/v11 INT8 → TensorRT** at **~12 ms** on **Jetson Orin Nano Super NX**; sensor data streamed to a **Raspberry Pi Zero 2 W UDP server**; validated on **Raspberry Pi 5** and **Orange Pi Zero**.
- **Containerized VLM inference**: building edge VLM serving with **vLLM** and **TensorRT-LLM** in Docker containers on the Jetson.
- **CUDA-Oxide (Rust-to-CUDA)**: contributing GPU kernels and documentation fixes (unsafe intrinsics wrapped in explicit unsafe blocks); pushing Jetson limits with idiomatic, memory-safe, fearlessly concurrent kernels.
- **Research & safety standards**: paper on **Intersection Blockage Prediction** | [paper](https://arpanpathak.github.io/driving-civicsense-vision-model/); studying ISO 26262. Books: **Seeing Machines** · **CUDA Kernels** | [book](https://arpanpathak.github.io/gpu-parallel-book/foreword.html)

---

## EXPERIENCE

### Software Developer II | **Microsoft**, Redmond, WA | Jun 2025 – Aug 2026

- **Containerized Cloud Infrastructure**: zero-trust sovereign-cloud networking with eBPF + Cilium enforcing **data residency** & SecNumCloud compliance; Kubebuilder operators reconcile state across AKS clusters (France, Germany).
- **Data Protection Services**: built and maintained **data loss prevention (DLP)** and **data lifecycle management** services with a **C#/.NET** backend.
- **Memory Safety (C++)**: refactored a **legacy C++ codebase** with **modern C++** practices (**smart pointers**, **RAII**), fixing **use-after-free** and **buffer-overflow** vulnerabilities.
- TechStack: **C++, C#, .NET, Python, Kubernetes, Cilium, eBPF, Docker, CI/CD, Linux**

### Senior Software Engineer | **Oracle Cloud Infrastructure**, Seattle, WA | Oct 2024 – Mar 2025

- **Backend Services & SDKs**: owned the **Terraform provider** (developer SDK) for Autonomous Database, replacing legacy control-plane tooling; **$1.2M/month** projected savings.
- **High-Performance Systems**: lock-free data structures reducing contention and latency in high-concurrency read-write paths.
- **Security & Key Management**: integrated OCI Security Vault with automated key rotation across control-plane services.
- TechStack: **Java, Go, REST/gRPC, Terraform, OCI**

### Software Development Engineer II | **Amazon**, Seattle, WA & Hyderabad, India | Mar 2021 – Oct 2024

- **Production Inference Serving**: scaled low-latency **BERT-based inference** to **5,000 QPS** (ONNX Runtime, INT8 quantization, latency tuning); validated throughput/latency; cut triage SLA **7 days → 2 hours**.
- **Real-Time ML Ranking**: owned end-to-end (training, serving, latency tuning) XGBoost + Deep Learning ensemble across **FreeVee, Twitch, MiniTV, Prime Video, Amazon Retail**; **5% ad CTR lift**, **$5.4M/month revenue**.
- **Event-Driven Microservices**: led a team of 3 building **programmatic guaranteed (PG) ad delivery** at **<50 ms** via real-time CDC (Kotlin); REST/gRPC APIs.
- **Distributed Systems**: designed a uniqueness-constraint indexing SDK (**two-phase commit**, **serializable isolation**, **3K writes/s**).
- **Data Platform**: built the **1-petabyte** warehouse/lake (AWS Glue, Spark) for financial reporting & pay computation.
- **Telemetry & Automation**: real-time monitoring and automated controls cut deal over-delivery ~30% and budget overspend 25%.
- TechStack: **Python, Kotlin, PyTorch, XGBoost, ONNX Runtime, Spark, Kafka, DynamoDB, Redshift, AWS**

### SDE | **Razorpay**, Bengaluru, India | Aug 2020 – Feb 2021

- **Backend Microservices**: P2P Unified Payments Gateway (Go, Protobuf/gRPC): **3M+ transactions/hour** peak, **99.99% uptime**, **150ms P99**; customers: **Zomato, Swiggy, Zerodha, Groww** (AWS, GCP).

### Software Engineer | **Mindfire Solutions**, Bhubaneswar, India | Aug 2018 – Feb 2020

- Java/Spring full-stack tools; AR product-search microservice (**10K+ daily queries**).

---

## SKILLS

**Languages**: Python, C++, Rust, Go, Java, Kotlin, C, Shell/Bash  
**AI/ML Inference**: Deep Learning, Computer Vision, PyTorch, XGBoost, BERT, LLMs, ONNX Runtime, TensorRT (INT8), vLLM, SGLang, TensorRT-LLM, YOLOv8/v11, Edge Inference, Quantization, MLOps  
**GPU & Parallel Computing**: CUDA, CUDA-Oxide (Rust-to-CUDA), SIMT, GPU Kernel Programming, NVIDIA Jetson Orin, HPC  
**Systems Programming & Performance**: Lock-Free Data Structures, Concurrency, Memory Safety, Performance Engineering, Low-Latency Systems, Computer Architecture  
**Networking**: TCP/IP, mTLS, HTTP/2/3, QUIC, eBPF, XDP, Cilium, Load Balancing  
**Distributed Systems & Streaming**: Consensus, Replication, Sharding, Kafka, Spark, CDC, Microservices, gRPC, REST, Protobuf  
**Cloud & Infra**: AWS (EC2, ECS, EKS, Lambda, Kinesis, EMR, SageMaker, CDK), Azure Kubernetes Service (AKS), Google Kubernetes Engine (GKE), Kubernetes, Docker, Helm, Terraform, Kubebuilder, Istio, CI/CD  
**Observability**: Prometheus, Grafana, AWS CloudWatch  
**Data & Storage**: DynamoDB, PostgreSQL, MySQL, Redis, Amazon Redshift, Amazon S3

---

## EDUCATION

**B.Tech, Computer Science & Engineering**, RCC Institute of Information Technology, Kolkata | 2018
