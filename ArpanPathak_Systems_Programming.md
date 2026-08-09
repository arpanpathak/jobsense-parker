# Arpan Pathak

**Seattle, Washington, USA** | Open to Relocation  
Email: arpan.pathak47@gmail.com  
Phone: +1 (206) 306-6059  
LinkedIn: [linkedin.com/in/arpan-pathak-272341424](https://linkedin.com/in/arpan-pathak-272341424)  
GitHub: [github.com/arpanpathak](https://github.com/arpanpathak)  

---

## SUMMARY

Systems Software Engineer with **8 years** architecting high-performance, low-latency systems at **Amazon**, **Microsoft**, and **Oracle** — database internals, kernel-level networking, distributed systems, and real-time ML inference. Deep expertise in **Rust**, **C++**, **Go**, and **C**: lock-free concurrency, memory safety, eBPF/Cilium, and GPU-accelerated computing. Shipped measurable wins: **~40% lower P99 latency**, **2x throughput**, **45,000 ops/s** throughput gains, **5,000 QPS** inference serving, and **$5.4M/month** revenue impact.

---

## PERSONAL PROJECTS

**Driving-CivicSense (Edge AI Vision)** | [github.com/arpanpathak/driving-civicsense-vision-model](https://github.com/arpanpathak/driving-civicsense-vision-model) | 2026

- On-device AI perception in **Rust** (intersection discipline, lane awareness) with **< 50ms inference** and zero cloud dependency on Raspberry Pi Zero / Pi 5; fine-tunes **YOLOv8/v11** + custom **CNN** with pruning & quantization for tight memory/power budgets.

**cuda-oxide (Rust-to-CUDA Compiler)** | [github.com/arpanpathak/cuda-oxide](https://github.com/arpanpathak/cuda-oxide)

- Experimental **Rust-to-CUDA** compiler: write SIMT GPU kernels in idiomatic Rust and compile straight to **PTX** — no DSLs, no bindings.

**Companion Book: Seeing Machines: Deep Learning & Computer Vision from Python to Bare Metal** | [arpanpathak.github.io/seeing-machines-book](https://arpanpathak.github.io/seeing-machines-book/foreword.html)

**Physics & Mathematics of Game Development in Rust (Book)** | [arpanpathak.github.io/bevy-physics-book](https://arpanpathak.github.io/bevy-physics-book/ch01-foreword.html) | 2026

**CUDA Kernels: GPU & Parallel Programming from First Principles (Book)** | [arpanpathak.github.io/gpu-parallel-book](https://arpanpathak.github.io/gpu-parallel-book/) | [github.com/arpanpathak/gpu-parallel-book](https://github.com/arpanpathak/gpu-parallel-book) | 2026

---

## EXPERIENCE

### Software Developer II | **Microsoft**, Redmond, WA | Jun 2025 – Present

- **C++/C# → Rust Migration (Memory Safety)**: Spearheaded migration of the Information Protection service from **unsafe C++** and **C#** to **Rust**, moving memory safety to compile time — eliminated use-after-free, buffer-overflow, and null-safety bugs in the migrated path. Delivered **~40% lower P99 latency**, **2x throughput**, **3x lower memory** per request, and **6-figure annual cost savings**.
- **Zero-Trust Kernel Networking (eBPF/Cilium)**: Architected zero-trust sovereign cloud networking with **eBPF + Cilium** network policies enforcing **data residency** and **SecNumCloud** compliance; authored **Kubernetes operators (Kubebuilder)** to reconcile policy state across AKS clusters in France and Germany.
- **AI-Powered Policy Engine**: Built an AI engine that auto-generates Cilium policies and scans for misconfigurations — manual review cut from **4 hours to near-real-time**, zero policy drift across sovereign regions.
- **CI/CD & Observability**: Containerized microservices with blue-green deployments, **Prometheus**/**Azure Monitor**, and automated validation for reliable multi-region releases.
- TechStack: **Rust, C++, eBPF, Cilium, Kubernetes (AKS), Kubebuilder, Prometheus, Linux**

### Senior Software Engineer | **Oracle Cloud Infrastructure**, Seattle, WA | Oct 2024 – Mar 2025

- **High-Performance Database Internals**: Architected low-latency internals for a proprietary database engine, optimizing **lock-free data structures** to cut contention in high-concurrency read-write paths — **45,000 ops/s** transaction throughput gain.
- **Infrastructure-as-Code SDK**: Engineered a **Terraform provider** (developer-facing SDK) for Oracle Autonomous Database, replacing legacy control-plane tooling with **$1.2M/month** projected savings.
- **Security & Key Management**: Integrated OCI Security Vault across control-plane services for secret management and automated key rotation.
- TechStack: **Rust, C++, Go, Terraform, PostgreSQL internals, OCI**

### Software Development Engineer II | **Amazon**, Seattle, WA & Hyderabad, India | Mar 2021 – Oct 2024

- **Real-Time ML Inference Engine**: Architected **XGBoost + Deep Learning ensemble** ranking for ads & deals across FreeVee, Twitch, Prime Video, and Amazon Retail — millions of events at **100ms inference latency**, driving **5% CTR lift**, 30% more deal impressions, and **$5.4M/month revenue growth**.
- **BERT Inference Service**: Delivered **BERT-based** NLP inference handling **5,000 QPS** at peak, cutting support-ticket triage SLA from **7 days to 2 hours**.
- **Distributed Systems & SDK Design**: Designed uniqueness-constraint indexing and an **SDK** for a multi-tenant NoSQL store using **two-phase commit**, achieving **serializable isolation** at **3K writes/second** — a developer-facing library used by multiple publisher teams.
- **Event-Driven Ads (CDC)**: Delivered programmatic guaranteed ad deals via real-time **change data capture (CDC)** with **< 50ms latency** for auction and programmatic buying platforms.
- **Big Data Platform**: Built **1-petabyte** data lake & warehouse with **Apache Spark**, **AWS Glue**, Kotlin Spark API, and a **DSL execution engine** for OLTP and BI use cases.
- TechStack: **Kotlin, Java, Python, PyTorch, XGBoost, Spark, Kafka, DynamoDB, Redshift, AWS**

### Software Development Engineer | **Razorpay**, Bengaluru, India | Aug 2020 – Feb 2021

- Built UPI Payment Gateway Aggregator in **Go** + Protobuf processing **1M+ hourly transactions** with **99.99% uptime** at **150ms P99** under **50,000 TPM** peak load.
- TechStack: **Go, Redis, PostgreSQL, Protobuf, Kafka**

### Software Engineer | **Mindfire Solutions**, Bhubaneswar, India | Aug 2018 – Feb 2020

- Delivered full-stack workflow tools and an **AR-based product search microservice** serving **10,000+ daily queries**; cut dashboard latency from **5 min to 1 min**.
- TechStack: **Java, Spring Boot, Angular, MySQL, MongoDB**

---

## SKILLS

**Languages**: Rust, C++, Go, C, Python, Java, Kotlin, TypeScript, Shell Script

**Systems Programming & Performance**: Lock-Free Data Structures, Concurrency, Memory Safety, Memory Profiling, Cache Optimization, Performance Engineering, Low-Latency Systems, OS Internals, Computer Architecture

**Networking & Protocols**: TCP/IP, UDP, TLS, mTLS, DNS, HTTP/2, HTTP/3, QUIC, WebSockets, Socket Programming, Network Performance Optimization

**Network & Cloud Security**: eBPF, XDP, Cilium, Kernel-Level Networking, Packet Processing, VPC, Subnets, Routing, NAT, VPN, Peering, Load Balancing, Security Groups / Firewalls

**Distributed Systems & Streaming**: Distributed Systems (consensus, replication, sharding), Apache Kafka, Apache Flink, Apache Spark, Real-Time Data Pipelines, Change Data Capture (CDC), Microservices

**APIs & Serialization**: gRPC, REST API Design, Protobuf

**ML Inference & AI**: PyTorch, TensorFlow, XGBoost, Transformer Models (BERT), LLMs, ONNX Runtime, On-Device/Edge Inference, Model Pruning & Quantization, Computer Vision (YOLOv8/v11), ANN, CNN, NLP, Deep Learning, Recommendation Systems, MLOps

**GPU & Parallel Computing**: CUDA, CUDA-Oxide (Rust-to-CUDA), PTX, SIMT, GPU Kernel Programming, Parallel Computing, GPU-Accelerated Computing

**Cloud & Infra**: AWS (EC2, ECS, Lambda, Kinesis, EMR, SageMaker), Azure (AKS, Bicep), Google Cloud, Kubernetes (EKS, AKS), Docker, Terraform, Kubebuilder, Istio, Kubernetes Gateway API

**Observability**: Prometheus, Grafana, Azure Monitor, AWS CloudWatch

**Data & Storage**: DynamoDB, PostgreSQL, Redis, MongoDB, MySQL, Amazon Redshift, Amazon S3

**Core CS**: Data Structures & Algorithms, System Design, Operating Systems, Linux, Computer Architecture

---

## EDUCATION

**Bachelor of Technology in Computer Science & Engineering**  
RCC Institute of Information Technology, Kolkata, India | 2018
