# Arpan Pathak

::: project-box
PERSONAL PROJECTS
**Driving-CivicSense (Edge AI Vision)** | [github.com/arpanpathak/driving-civicsense-vision-model](https://github.com/arpanpathak/driving-civicsense-vision-model) | 2026
- On-device AI perception system built in **Rust** for intersection discipline and lane awareness — runs entirely on-device (**< 50ms inference latency**, no cloud dependency) for aftermarket dashcam and AR-glasses form factors.
- Fine-tunes **YOLOv8/v11** and custom **CNN** architectures; applies model pruning and quantization to fit tight memory and power budgets on low-powered single-board computers (Raspberry Pi 5).
- Trains on cloud GPU infrastructure with synthetic data augmentation; planning to release a public driving-behavior dataset on **Hugging Face**.
:::

**Seattle, Washington, USA** | Open to Relocation  
Email: arpan.pathak47@gmail.com | Phone: +1 (206) 306-6059  
LinkedIn: [linkedin.com/in/arpan-pathak-272341424](https://linkedin.com/in/arpan-pathak-272341424)  
GitHub: [github.com/arpanpathak](https://github.com/arpanpathak)  
**Visa: Requires H1B Transfer sponsorship**

---

## SUMMARY

Systems Software Engineer with 8+ years architecting performance-critical backend systems, distributed infrastructure, and real-time **ML inference** pipelines at **Amazon**, **Microsoft**, and **Oracle**. Deep expertise in **Rust**, **C++**, **Go**, and **Python** — lock-free concurrency, memory safety, low-latency streaming (**Kafka**), and on-device inference. Proven track record shipping high-throughput, resource-efficient systems at global scale; focused on systems programming for consumer electronics, where performance, memory budgets, and real-time responsiveness are the product.

---

## EXPERIENCE

### Software Developer II | **Microsoft** — Redmond, WA | June 2025 – Present

- **C++ to Rust Migration**: Spearheaded migration of the Information Protection service from **C++ to Rust**, shifting memory safety to compile-time. Eliminated use-after-free and buffer-overflow CVEs in the audited path while maintaining full performance parity — a critical win for DORA compliance.
- **AI-Powered Policy Engine**: Built an AI-driven engine that auto-generates Cilium network policies and scans for misconfigurations, reducing manual policy review cycles from 4 hours to near-real-time with zero policy drift across sovereign regions.
- **eBPF & Kernel-Level Networking**: Designed and deployed eBPF-based network policies using **Cilium** for Microsoft Purview's sovereign cloud regions. Authored custom **Kubernetes Operators** (Kubebuilder) to reconcile compliance states across AKS clusters.
- **Developer Workflows**: Established CI/CD pipelines, containerized microservices, and automated validation frameworks ensuring reliable multi-region releases.
- Infra: **Rust, C++, Kubernetes, eBPF**

### Senior Software Engineer | **Oracle Cloud Infrastructure** — Seattle, WA | Oct 2024 – Mar 2025

- **High-Performance Engine**: Architected low-latency internals for a proprietary database engine, optimizing **lock-free data structures** to reduce contention in high-concurrency read-write paths, improving transaction throughput by **45,000 ops/s**.
- **Infrastructure-as-Code SDK**: Engineered a Terraform provider (developer-facing SDK) for Oracle Autonomous Database, replacing legacy control plane infrastructure with projected **$1.2M/month cost savings**.
- **Security & Key Management**: Integrated OCI Security Vault across control plane services for secure secret management and automated key rotation protocols.
- Infra: **Java, Go, Terraform Provider, Oracle Autonomous Database**

### Software Development Engineer II | **Amazon** — Seattle, WA & Hyderabad, India | Mar 2021 – Oct 2024

- **Real-Time ML Inference Engine**: Architected an **ensemble model pipeline combining XGBoost and Deep Learning** for real-time ads & deal ranking across FreeVee, Twitch, MiniTV, Prime Video, and Amazon Retail — processing millions of events at **100ms inference latency** while driving **5% CTR increase**, 30% higher deal impressions, and **$5.4M monthly revenue growth**.
- **Distributed Systems & SDK Design**: Designed a uniqueness constraint indexing solution and **SDK** for a multi-tenant NoSQL datastore using **two-phase commit**, achieving **serializable isolation** at **3K writes/second** — a developer-facing library used across multiple publisher teams.
- **Conversational AI (BERT)**: Developed a **BERT-based inference service** handling **5,000 QPS** at peak, reducing support ticket triage SLA from 7 days to 2 hours.
- **Big Data Pipelines**: Engineered **Apache Spark** jobs processing millions of daily events into Amazon Redshift for OLAP with automated dashboards.
- Infra: **Kotlin, Java, Python, TypeScript, AWS CDK, React.Js, DynamoDB, Redshift, AWS, PostgreSQL, PyTorch, XGBoost**

### Software Development Engineer | **Razorpay** — Bengaluru, India | Aug 2020 – Feb 2021

- Built UPI Payment Gateway Aggregator using **Golang** and Protobuf, processing **1M+ hourly transactions**.
- Maintained **99.99% uptime** with **150ms P99 latency** under 50,000 TPM peak loads.
- Infra: **Go, Redis, Protobuf**

### Software Engineer | **Mindfire Solutions** — Bhubaneswar, India | Aug 2018 – Feb 2020

- Developed full-stack workflow tools and an **AR-based product search microservice**, serving 10,000+ daily queries.
- Built administrative dashboards reducing data retrieval latency from 5 min to 1 min.
- Infra: **Java, Angular, React, MongoDB, MySQL, SpringBoot**

---

## SKILLS

**Languages**: Rust, C++, Go, Python, Java, Kotlin, C, TypeScript

**Systems Programming & Performance**: Lock-Free Data Structures, Concurrency, Memory Safety, Memory Profiling, Cache Optimization, eBPF, Kernel-Level Networking, Performance Engineering, Low-Latency Systems

**Distributed Systems & Streaming**: Apache Kafka, Apache Flink, Apache Spark, Distributed Systems (consensus, replication, sharding), Real-Time Data Pipelines, gRPC, REST API Design

**ML Inference & AI**: PyTorch, TensorFlow, XGBoost, Transformer Models (BERT), ONNX Runtime, On-Device/Edge Inference, Model Pruning & Quantization, Computer Vision (YOLOv8/v11), Recommendation Systems, MLOps

**Cloud & Infra**: AWS (EC2, ECS, Lambda, Kinesis, EMR, SageMaker), Azure (AKS), Kubernetes (EKS, AKS), Docker, Terraform, Cilium, Kubebuilder, Istio

**Data & Storage**: DynamoDB, PostgreSQL, Redis, MongoDB, Amazon Redshift, Amazon S3

**Core CS**: Data Structures & Algorithms, System Design, Operating Systems, Networking (TCP/IP, UDP, TLS, DNS, HTTP/3)

---

## EDUCATION

**Bachelor of Technology — Computer Science & Engineering**  
RCC Institute of Information Technology — Kolkata, India | 2018
