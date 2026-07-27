# Arpan Pathak

::: project-box
PERSONAL PROJECTS
**Driving-CivicSense Companion (KMP)** | [github.com/arpanpathak/driving-civicsense-vision-model](https://github.com/arpanpathak/driving-civicsense-vision-model) | 2026 (In Progress)
- Building a **Kotlin Multiplatform** companion app for the CivicSense edge-AI perception system that watches intersections, detects blocking violations, enforces lane discipline, and tracks turn-signal compliance on-device (YOLOv8/v11 via ONNX + Deep SORT tracking).
- **Intersection blocking module**: renders real-time intersection occupancy heatmaps from Rust inference pipeline on a Jetpack Compose map overlay. Alerts driver when occupancy exceeds 70% with <30ft stopping distance.
- **Turn signal enforcement**: consumes amber-blinker detection events and lateral vehicle motion tracks from the Rust pipeline. Logs unsignaled lane changes, late signals, and multi-lane cuts with severity scoring.
- **Mesh hazard network viewer**: visualizes distributed hazard beacons (debris, animals, blocked intersections) reported by nearby CivicSense units via LoRa/BLE. Shows verified conditions when 3+ units detect same hazard.
- Shared KMP module handles gRPC deserialization, local persistence via SQLDelight, and offline-first state. Android UI in Jetpack Compose (MVVM + Coroutines/Flow). iOS UI in SwiftUI.
:::

**Seattle, Washington, USA** | Open to Relocation  
Email: arpan.pathak47@gmail.com | Phone: +1 (206) 306-6059  
LinkedIn: [linkedin.com/in/arpan-pathak-272341424](https://linkedin.com/in/arpan-pathak-272341424)  
GitHub: [github.com/arpanpathak](https://github.com/arpanpathak)  
**Visa: Requires H1B Transfer sponsorship**

---

## SUMMARY

Systems Software Engineer with 8+ years architecting high-performance, low-latency systems at **Amazon**, **Microsoft**, and **Oracle**, now applying systems depth to Android engineering. Deep expertise in **Kotlin**, **C++**, **Rust**, lock-free concurrency, SDK design, and real-time streaming at global scale. Brings memory-safe, cache-aware, performance-profiled engineering to mobile -- the same discipline that shipped multi-million-dollar infrastructure. Currently building a **Kotlin Multiplatform** companion app for an on-device AI perception system targeting intersection safety, lane discipline, and cooperative hazard detection.

---

## EXPERIENCE

### Software Developer II | **Microsoft** - Redmond, WA | June 2025 - Present

- **C++ to Rust Migration**: Spearheaded migration of the Information Protection service from **C++ to Rust**, shifting memory safety to compile-time. Eliminated use-after-free and buffer-overflow CVEs in the audited path while maintaining full performance parity -- a critical win for DORA compliance. Memory-safe concurrency discipline directly applies to Android platform development.
- **AI-Powered Policy Engine**: Built an AI-driven engine that auto-generates Cilium network policies and scans for misconfigurations, reducing manual policy review cycles from 4 hours to near-real-time with zero policy drift across sovereign regions. Reactive streaming design transferable to Android lifecycle-aware architectures.
- **eBPF & Kernel-Level Networking**: Designed and deployed eBPF-based network policies using **Cilium** for Microsoft Purview's sovereign cloud regions. Authored custom **Kubernetes Operators** (Kubebuilder) to reconcile compliance states across AKS clusters.
- **Developer Workflows**: Established CI/CD pipelines, containerized microservices, and automated validation frameworks ensuring reliable multi-region releases.
- Infra: **C++, Rust, Kotlin, Python, Kubernetes (AKS), Kubebuilder, Docker, CI/CD, Linux**

### Senior Software Engineer | **Oracle Cloud Infrastructure** - Seattle, WA | Oct 2024 - Mar 2025

- **High-Performance Engine**: Architected low-latency internals for a proprietary database engine, optimizing **lock-free data structures** to reduce contention in high-concurrency read-write paths, improving transaction throughput by **45,000 ops/s**. Lock-free patterns transfer directly to Android UI threading and render-performance engineering.
- **Infrastructure-as-Code SDK**: Engineered Terraform provider (developer-facing SDK) for Oracle Autonomous Database, replacing legacy control plane infrastructure with projected **$1.2M/month cost savings**. SDK design patterns directly applicable to Android library and framework development.
- **Security & Key Management**: Integrated OCI Security Vault across control plane services for secure secret management and automated key rotation protocols.
- Infra: **C++, Rust, Kotlin, Go, Terraform, OCI, PostgreSQL internals**

### Software Development Engineer II | **Amazon** - Seattle, WA & Hyderabad, India | Mar 2021 - Oct 2024

- **Real-Time ML Ranking Engine**: Architected an **ensemble model pipeline combining XGBoost and Deep Learning** for real-time ads & deal ranking across **FreeVee, Twitch, MiniTV, Prime Video, and Amazon Retail**, driving **5% CTR increase**, 30% higher deal impressions, and **$5.4M monthly revenue growth** -- processing millions of events with **100ms latency**. Same latency budgeting and performance profiling required for smooth mobile UI.
- **SDK & Distributed Systems Design**: Designed a uniqueness constraint indexing solution and **SDK** for a multi-tenant NoSQL datastore using two-phase commit, achieving **serializable isolation** at 3K writes/second -- a developer-facing library used across multiple publisher teams. API design and client library architecture analogous to Android SDK authorship.
- **Conversational AI (BERT)**: Developed a **BERT-based AI model** handling **5,000 QPS** at peak, reducing support ticket triage SLA from 7 days to 2 hours. Threading, resource pooling, and latency optimization at scale.
- **CI/CD & Developer Productivity**: Led migration to React.js microfrontend architecture with automated CI/CD pipelines, reducing deployment cycles from **5 days to 30 minutes** -- enabling full team autonomy and faster iteration.
- **Big Data Pipelines**: Engineered **Apache Spark** jobs processing millions of daily events into Amazon Redshift for OLAP with automated dashboards.
- Infra: **Python, C++, Kotlin, Java, Spark, Kafka, DynamoDB, Redshift, SageMaker, AWS, Docker**

### Software Development Engineer | **Razorpay** - Bengaluru, India | Aug 2020 - Feb 2021

- Built UPI Payment Gateway Aggregator using **Golang** and Protobuf, processing **1M+ hourly transactions**.
- Maintained **99.99% uptime** with **150ms P99 latency** under 50,000 TPM peak loads.
- Infra: **Go, Kafka, Redis, PostgreSQL**

### Software Engineer | **Mindfire Solutions** - Bhubaneswar, India | Aug 2018 - Feb 2020

- Developed full-stack workflow tools and **AR-based product search microservice**, serving 10,000+ daily queries. Mobile-adjacent engineering with camera and AR integration relevant to Android CameraX and ARCore.
- Built administrative dashboards reducing data retrieval latency from 5 min to 1 min.
- Infra: **Java, Kotlin, React, Python, MongoDB**

---

## SKILLS

**Android & Mobile**: Android SDK, Jetpack Compose, Kotlin Multiplatform (KMP), MVVM, Coroutines & Flow, gRPC mobile, BLE, SQLDelight, Mobile CI/CD

**Languages**: Kotlin, C++, Rust, Java, Go, Python, TypeScript, C#, Shell Script

**Performance Engineering**: Lock-Free Data Structures, Concurrency & Threading, Memory Profiling, Cache Optimization, Low-Latency Systems, Real-Time Pipelines, GPU-Accelerated Computing, eBPF

**SDK & Developer Tools**: SDK Design, API Development, Developer-Facing Libraries, Terraform Providers, CI/CD, Automated Testing, Gradle (KMP), Packaging

**Machine Learning & AI**: PyTorch, TensorFlow, XGBoost, Transformer Models (BERT), Computer Vision (YOLOv8/v11), ONNX Runtime, On-Device Edge AI

**Distributed Systems**: AWS (EC2, ECS, Lambda, Kinesis, EMR, SageMaker), Azure (AKS), Kubernetes (AKS, EKS), Docker, Terraform, Apache Kafka, Apache Spark, Redshift

**Core CS**: Data Structures & Algorithms, System Design, Networking (TCP/IP, TLS, DNS, HTTP/3), Operating Systems, Memory Safety

---

## EDUCATION

**Bachelor of Technology - Computer Science & Engineering**  
RCC Institute of Information Technology - Kolkata, India | 2018
