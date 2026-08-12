# Arpan Pathak

Seattle, Washington, USA | Open to Relocation  
Email: arpan.pathak47@gmail.com | Phone: +1 (206) 306-6059  
LinkedIn: [linkedin.com/in/arpan-pathak-272341424](https://linkedin.com/in/arpan-pathak-272341424) | GitHub: [github.com/arpanpathak](https://github.com/arpanpathak)

---

## SUMMARY

Senior Software Engineer, **8 years** building developer tools, observability platforms, and performance-critical distributed systems at **Amazon, Microsoft, Oracle**. Full-stack **Rust, C++, Kotlin, Go, Python, JavaScript**; **Kubernetes**, **eBPF/Cilium**, GPU/**CUDA**, cloud **performance optimization**.

---

## EXPERIENCE

### Software Developer II | **Microsoft**, Redmond, WA | Jun 2025 – Aug 2026

- **Zero-Trust Sovereign Cloud Networking (eBPF/Cilium)**: Architected zero-trust sovereign cloud networking with **eBPF + Cilium** network policies enforcing **data residency** and **SecNumCloud** compliance; authored **Kubernetes operators (Kubebuilder)** to reconcile policy state across AKS clusters in France and Germany.
- **C++/C# to Rust Migration (Memory Safety)**: Spearheaded migration of the Information Protection service from **unsafe C++** and **C#** to **Rust**, moving memory safety to compile time: eliminated use-after-free, buffer-overflow, and null-safety bugs in the migrated path. Delivered **~40% lower P99 latency**, **2x throughput**, **3x lower memory** per request, and **6-figure annual cost savings**.
- **AI-Powered Policy Engine**: Built an AI engine that auto-generates Cilium policies and scans for misconfigurations: manual review cut from **4 hours to near-real-time**, zero policy drift across sovereign regions.
- TechStack: **Rust, C++, eBPF, Cilium, Kubernetes (AKS), Kubebuilder, Prometheus, Linux**

### Senior Software Engineer | **Oracle Cloud Infrastructure**, Seattle, WA | Oct 2024 – Mar 2025

- **High-Performance Database Engine**: Architected low-latency internals for a proprietary database engine, optimizing **lock-free data structures** to reduce contention in high-concurrency read-write paths, improving transaction throughput by **45,000 ops/s**.
- **Infrastructure-as-Code SDK**: Engineered a **Terraform provider** (developer-facing SDK) for Oracle Autonomous Database, replacing legacy control plane infrastructure with **$1.2M/month** projected savings.
- **Security & Key Management**: Integrated OCI Security Vault across control plane services for secure secret management and automated key rotation protocols.
- TechStack: **Java, Go, Terraform, Oracle Cloud Infrastructure, Autonomous Database**

### Software Development Engineer II | **Amazon**, Seattle, WA & Hyderabad, India | Mar 2021 – Oct 2024

- **Real-Time ML Ranking Engine**: Architected an **XGBoost + Deep Learning ensemble** for real-time ads & deal ranking across FreeVee, Twitch, MiniTV, Prime Video, and Amazon Retail, driving **5% CTR increase**, 30% higher deal impressions, and **$5.4M monthly revenue growth**.
- **BERT Inference Service**: Delivered **BERT-based** NLP inference handling **5,000 QPS** at peak, cutting support-ticket triage SLA from **7 days to 2 hours**.
- **Event Bus SDK (Kotlin, CSP)**: Built an event bus SDK using **Kotlin structured concurrency** and the **CSP (communicating sequential processes)** paradigm to deliver programmatic guaranteed ad platform deals in real-time with **< 50ms end-to-end latency**.
- **Distributed Systems & SDK Design**: Designed uniqueness-constraint indexing and an **SDK** for a multi-tenant NoSQL store using **two-phase commit**, achieving **serializable isolation** at **3K writes/second**.
- **Big Data Platform**: Built big data infrastructure & pipelines for the Amazon WorkEvents data platform stream team: **1-petabyte** data warehouse & data lake serving **1M+ daily BI queries**; wrote business logic in **AWS Glue (serverless Apache Spark)** for financial reporting and pay computation.
- TechStack: **Kotlin, Java, Python, PyTorch, XGBoost, Spark, Kafka, DynamoDB, Redshift, AWS**

### Software Development Engineer | **Razorpay**, Bengaluru, India | Aug 2020 – Feb 2021

- Built UPI Payment Gateway Aggregator in **Go** + Protobuf processing **1M+ hourly transactions** with **99.99% uptime** at **150ms P99** under **50,000 TPM** peak load.
- TechStack: **Go, Redis, PostgreSQL, Protobuf, Kafka**

### Software Engineer | **Mindfire Solutions**, Bhubaneswar, India | Aug 2018 – Feb 2020

- Delivered full-stack workflow tools and an **AR-based product search microservice** serving **10,000+ daily queries**; cut dashboard latency from **5 min to 1 min**.
- TechStack: **Java, Spring Boot, Angular, MySQL, MongoDB**

---

## PERSONAL PROJECTS

**Driving-CivicSense: Edge AI Vision System (Rust)**

- Real-time on-device perception pipeline in **Rust** on the **NVIDIA Jetson Orin Nano Super** (67 INT8 TOPS, 8 GB, 7–15 W): **YOLOv8/v11 INT8 ONNX** (~12 ms) + Deep SORT/Kalman tracking + formally-proven kinematic decision engine; 100% on-device, privacy-first.
- Research paper: **Deterministic Intersection Blockage Prediction** | [arpanpathak.github.io/driving-civicsense-vision-model/](https://arpanpathak.github.io/driving-civicsense-vision-model/) | Code: [github.com/arpanpathak/driving-civicsense-vision-model](https://github.com/arpanpathak/driving-civicsense-vision-model)
- Books: **Seeing Machines** | [arpanpathak.github.io/seeing-machines-book](https://arpanpathak.github.io/seeing-machines-book/foreword.html) · **CUDA Kernels** (CUDA-Oxide / Rust-to-CUDA) | [arpanpathak.github.io/gpu-parallel-book](https://arpanpathak.github.io/gpu-parallel-book/)

---

## SKILLS

**Languages**: Rust, C++, Kotlin, Go, C, Java, Python, TypeScript, JavaScript, Shell Script  
**Systems Programming & Performance**: Lock-Free Data Structures, Concurrency, Memory Safety, Memory Profiling, Cache Optimization, Performance Engineering, Low-Latency Systems, OS Internals, Computer Architecture  
**Networking & Protocols**: TCP/IP, UDP, TLS, mTLS, DNS, HTTP/2, HTTP/3, QUIC, WebSockets, RDMA, InfiniBand, DPDK, VXLAN, GRE, Socket Programming, Packet Processing, Network Performance Optimization  
**Network & Cloud Security**: eBPF, XDP, Cilium, Kernel-Level Networking, Packet Processing, VPC, Subnets, Routing, NAT, VPN, Peering, Load Balancing, Security Groups / Firewalls  
**Distributed Systems & Streaming**: Distributed Systems (consensus, replication, sharding), Apache Kafka, Apache Flink, Apache Spark, Spark Streaming, AWS Kinesis, Real-Time Data Pipelines, Change Data Capture (CDC), Microservices  
**APIs & Serialization**: gRPC, REST API Design, Protobuf  
**ML Inference & AI**: Deep Learning, Computer Vision, PyTorch, TensorFlow, XGBoost, Transformer Models (BERT), LLMs, ONNX Runtime, On-Device/Edge Inference, Model Pruning & Quantization, YOLOv8/v11, CNN, NLP, Recommendation Systems, MLOps  
**GPU & Parallel Computing**: CUDA, CUDA-Oxide (Rust-to-CUDA), SIMT, GPU Kernel Programming, Parallel Computing, GPU-Accelerated Computing  
**Cloud & Infra**: AWS (EC2, ECS, Lambda, Kinesis, EMR, SageMaker), Azure (AKS), Google Cloud, Kubernetes (EKS, AKS), Docker, Terraform, Kubebuilder, Istio, Kubernetes Gateway API  
**Observability**: Prometheus, Grafana, Azure Monitor, AWS CloudWatch  
**Data & Storage**: DynamoDB, PostgreSQL, Redis, MongoDB, MySQL, Amazon Redshift, Amazon S3  
**Core CS**: Data Structures & Algorithms, System Design, Operating Systems, Linux

---

## EDUCATION

**B.Tech, Computer Science & Engineering**, RCC Institute of Information Technology, Kolkata | 2018
