# Arpan Pathak

::: project-box
PERSONAL PROJECTS
**Driving-CivicSense (Edge AI Vision)** | [github.com/arpanpathak/driving-civicsense-vision-model](https://github.com/arpanpathak/driving-civicsense-vision-model) | 2026
- On-device AI perception system built in **Rust** for intersection discipline and lane awareness, running entirely on-device (**< 50ms inference latency**, no cloud dependency) for aftermarket dashcam and AR-glasses form factors.
- Fine-tunes **YOLOv8/v11** and custom **CNN** architectures; applies model pruning and quantization to fit tight memory and power budgets on low-powered single-board computers (Raspberry Pi 5).
- Trains on cloud GPU infrastructure with synthetic data augmentation; planning to release a public driving-behavior dataset on **Hugging Face**.
**Physics & Mathematics of Game Development in Rust (Book)** | [arpanpathak.github.io/bevy-physics-book](https://arpanpathak.github.io/bevy-physics-book/ch01-foreword.html) | 2026
- Authored an unpublished book on the **physics and mathematics of game development in Rust**, deep-diving the math and systems behind real-time simulation.
:::

**Seattle, Washington, USA** | Open to Relocation  
Email: arpan.pathak47@gmail.com  
Phone: +1 (206) 306-6059  
LinkedIn: [linkedin.com/in/arpan-pathak-272341424](https://linkedin.com/in/arpan-pathak-272341424)  
GitHub: [github.com/arpanpathak](https://github.com/arpanpathak)  
**Visa: Requires H1B Transfer sponsorship**

---

## SUMMARY

Software Engineer with nearly 8 years of industry experience architecting high-performance **database internals**, **cloud infrastructure**, **distributed systems**, and real-time **ML inference** pipelines at **Amazon**, **Microsoft**, and **Oracle**. Deep expertise in **Kotlin (JVM)**, **Rust**, **C++**, **Go**, and **Python**: lock-free concurrency, memory safety, low-latency streaming (**Kafka**), and ML inference at scale. Proven track record shipping high-throughput, resource-efficient systems at global scale — from lock-free database engines and cloud control planes to real-time ML ranking and petabyte-scale data platforms.

---

## EXPERIENCE

### Software Developer II | **Microsoft**, Redmond, WA | Jun 2025 – Aug 2026

- **Rust Migration (Legacy C++/C#)**: Spearheaded migration of the Information Protection service from a legacy **unsafe C++** and garbage-collected **C#** codebase to **Rust**, shifting memory safety to compile-time. Completely eliminated **use-after-free**, **buffer-overflow**, and **null-safety** bugs in the migrated path. Delivered measurable performance gains over C#, **~40% lower P99 latency**, **~2x throughput**, and **~3x lower memory consumption per request**, with **6-figure annual cost savings**. Significantly increased developer productivity and confidence writing memory-safe code for MSEC.
- **AI-Powered Policy Engine**: Built an AI-driven engine that auto-generates Cilium network policies and scans for misconfigurations, reducing manual policy review cycles from 4 hours to near-real-time with zero policy drift across sovereign regions.
- **Zero-Trust Sovereign Cloud Networking**: Architected **zero-trust sovereign cloud networking** using **eBPF and Cilium-based network policies** to ensure **data residency** and **SecNumCloud** compliance standards. Built **Kubernetes operators** to auto-generate network policies with CI/CD pipelines.
- **CI/CD & Observability**: Established CI/CD pipelines, containerized microservices, and automated validation frameworks ensuring reliable multi-region releases. Integrated observability with **Prometheus** and **Azure Monitor**, and implemented **blue-green deployments**, improving operational efficiency.
- TechStack: **Rust, C++, Kubernetes, eBPF**

### Senior Software Engineer | **Oracle Cloud Infrastructure**, Seattle, WA | Oct 2024 – Mar 2025

- **High-Performance Engine**: Architected low-latency internals for a proprietary database engine, optimizing **lock-free data structures** to reduce contention in high-concurrency read-write paths, improving transaction throughput by **45,000 ops/s**.
- **Infrastructure-as-Code SDK**: Engineered a Terraform provider (developer-facing SDK) for Oracle Autonomous Database, replacing legacy control plane infrastructure with projected **$1.2M/month cost savings**.
- **Security & Key Management**: Integrated OCI Security Vault across control plane services for secure secret management and automated key rotation protocols.
- TechStack: **Java, Go, Terraform Provider, Oracle Autonomous Database**

### Software Development Engineer II | **Amazon**, Seattle, WA & Hyderabad, India | Mar 2021 – Oct 2024

- **Real-Time ML Inference Engine**: Architected an **ensemble model pipeline combining XGBoost and Deep Learning** for real-time ads & deal ranking across FreeVee, Twitch, MiniTV, Prime Video, and Amazon Retail, processing millions of events at **100ms inference latency** while driving **5% CTR increase**, 30% higher deal impressions, and **$5.4M monthly revenue growth**.
- **Event-Driven Ads Systems**: Designed and delivered event-driven systems for the ads publisher platform, delivering **programmatic guaranteed ad deals** via real-time **change data capture (CDC)** with **< 50ms latency** for ad consumer-side platforms handling auction and programmatic buying inventory.
- **Distributed Systems & SDK Design**: Designed a uniqueness constraint indexing solution and **SDK** for a multi-tenant NoSQL datastore using **two-phase commit**, achieving **serializable isolation** at **3K writes/second**, a developer-facing library used across multiple publisher teams.
- **Conversational AI (BERT)**: Developed a **BERT-based inference service** handling **5,000 QPS** at peak, reducing support ticket triage SLA from 7 days to 2 hours.
- **Big Data Platform & Data Lake**: Built big data platform, data warehouse, and data lake infrastructure for Amazon WorkEvents data handling **1 petabyte** of data and serving OLTP use cases. Designed a **domain-specific language (DSL) execution engine** to deploy datasets to **AWS QuickSight** for business intelligence, using **AWS Glue**, **Kotlin Spark API**, and **AWS CDK (TypeScript)** to orchestrate job infrastructure and data refresh strategies.
- TechStack: **Kotlin, Java, Python, TypeScript, AWS CDK, React.Js, DynamoDB, Redshift, AWS, PostgreSQL, PyTorch, XGBoost**

### Software Development Engineer | **Razorpay**, Bengaluru, India | Aug 2020 – Feb 2021

- Built UPI Payment Gateway Aggregator using **Golang** and Protobuf, processing **1M+ hourly transactions**.
- Maintained **99.99% uptime** with **150ms P99 latency** under 50,000 TPM peak loads.
- TechStack: **Go, Redis, PostgreSQL, Protobuf, Google Cloud, AWS**

### Software Engineer | **Mindfire Solutions**, Bhubaneswar, India | Aug 2018 – Feb 2020

- Developed full-stack workflow tools and an **AR-based product search microservice**, serving 10,000+ daily queries.
- Built administrative dashboards reducing data retrieval latency from 5 min to 1 min.
- TechStack: **Java, SpringBoot, Angular, MySQL, MongoDB, AWS**

---

## SKILLS

**Languages**: Rust, C++, Go, Python, Java, Kotlin, C, TypeScript, Shell Script

**Systems Programming & Performance**: Lock-Free Data Structures, Concurrency, Memory Safety, Memory Profiling, Cache Optimization, Performance Engineering, Low-Latency Systems

**Networking & Protocols**: TCP/IP, UDP, TLS, mTLS, DNS, HTTP/2, HTTP/3, WebSockets, Socket Programming, Network Performance Optimization

**Network & Cloud Security**: eBPF, Cilium, XDP, Kernel-Level Networking, Packet Processing, VPC, Subnets, Routing, NAT, VPN, Peering, Load Balancing, Security Groups / Firewalls

**Distributed Systems & Streaming**: Apache Kafka, Apache Flink, Apache Spark, Distributed Systems (consensus, replication, sharding), Real-Time Data Pipelines

**APIs & Serialization**: gRPC, REST API Design, Protobuf

**ML Inference & AI**: PyTorch, TensorFlow, XGBoost, Transformer Models (BERT), ONNX Runtime, On-Device/Edge Inference, Model Pruning & Quantization, Computer Vision (YOLOv8/v11), Artificial Neural Networks (ANN), Convolutional Neural Networks (CNN), Sequential Models, Natural Language Processing (NLP), Deep Learning, Recommendation Systems, MLOps

**Cloud & Infra**: AWS (EC2, ECS, Lambda, Kinesis, EMR, SageMaker), Google Cloud, Azure (AKS), Kubernetes (EKS, AKS), Docker, Terraform, Kubebuilder, Istio

**Observability**: Prometheus, Grafana, Azure Monitor, AWS CloudWatch

**Data & Storage**: DynamoDB, PostgreSQL, Redis, MongoDB, Amazon Redshift, Amazon S3

**Core CS**: Data Structures & Algorithms, System Design, Operating Systems, Computer Architecture

---

## EDUCATION

**Bachelor of Technology in Computer Science & Engineering**  
RCC Institute of Information Technology, Kolkata, India | 2018
