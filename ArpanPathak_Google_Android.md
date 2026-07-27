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

Systems Software Engineer with 8+ years building high-performance, low-latency software at Amazon, Microsoft, and Oracle, now applying systems depth to Android engineering. Deep expertise in Kotlin, C++, Rust, lock-free concurrency, SDK design, and real-time streaming pipelines at global scale. Brings cache-aware, memory-safe, performance-profiled engineering rigor to mobile -- the same discipline behind multi-million-dollar infrastructure. Currently building a Kotlin Multiplatform companion app for an on-device AI perception system targeting intersection safety, lane discipline, and cooperative hazard detection.

---

## EXPERIENCE

### Software Developer II | **Microsoft** - Redmond, WA | June 2025 - Present

- **Low-Level Performance Engineering**: Migrated the Information Protection service from C++ to Rust, removing use-after-free and buffer-overflow CVEs at compile time while maintaining full performance parity. Memory-safe concurrency discipline directly applies to Android platform development.
- **Real-Time Policy Engine**: Built an AI engine that auto-generates network policies and scans misconfigurations, reducing manual review from 4 hours to near-real-time. Reactive streaming design transferable to Android lifecycle-aware architectures.
- **eBPF & Kernel Networking**: Deployed eBPF policies with Cilium and custom Kubernetes Operators for sovereign cloud regions. Deep systems expertise informs mobile performance optimization and resource-constrained execution.
- Infra: **Rust, C++, Kotlin, Python, Kubernetes (AKS), CI/CD, Linux**

### Senior Software Engineer | **Oracle Cloud Infrastructure** - Seattle, WA | Oct 2024 - Mar 2025

- **High-Performance Engine Architecture**: Architected lock-free data structures for a proprietary database engine, reducing contention in high-concurrency read-write paths and improving throughput by **45,000 ops/s**. Patterns transfer directly to Android UI threading and render-performance engineering.
- **SDK Design**: Built Terraform provider (developer-facing SDK) for Oracle Autonomous Database, replacing legacy control plane with **$1.2M/month savings**. SDK patterns apply to Android library and framework development.
- Infra: **Kotlin, C++, Rust, Go, Terraform, OCI, PostgreSQL internals**

### Software Development Engineer II | **Amazon** - Seattle, WA & Hyderabad, India | Oct 2021 - Oct 2024

- **Real-Time Low-Latency Engine**: Architected ensemble ML ranking (XGBoost + Deep Learning) operating at **100ms P99** across FreeVee, Twitch, Prime Video, and Amazon Retail, driving **5% CTR increase** and **$5.4M monthly revenue**. Same latency budgeting and performance profiling required for smooth mobile UI.
- **SDK Design**: Designed uniqueness constraint indexing SDK for multi-tenant NoSQL datastore with serializable isolation at 3K writes/second. API design and client library architecture analogous to Android SDK authorship.
- **Conversational AI (BERT)**: Built BERT-based system handling **5,000 QPS**, reducing support triage SLA from 7 days to 2 hours. Threading, resource pooling, and latency optimization at scale.
- Infra: **Kotlin, Python, C++, Java, Spark, Kafka, DynamoDB, AWS**

### Software Development Engineer | **Razorpay** - Bengaluru, India | Aug 2020 - Feb 2021

- Built UPI Payment Gateway Aggregator in **Golang** with Protobuf, processing **1M+ hourly transactions** at **99.99% uptime** with **150ms P99 latency** under 50,000 TPM peak. Systems performance discipline for mobile resource management.
- Infra: **Go, Kafka, Redis, PostgreSQL**

### Software Engineer | **Mindfire Solutions** - Bhubaneswar, India | Aug 2018 - Feb 2020

- Built an **AR-based product search microservice** serving 10,000+ daily queries. Mobile-adjacent engineering with camera and AR integration relevant to Android CameraX and ARCore.
- Reduced data retrieval latency from 5 min to 1 min with administrative dashboards.
- Infra: **Java, Kotlin, React, Python, MongoDB**

---

## SKILLS

**Android & Mobile**: Android SDK, Jetpack Compose, Kotlin Multiplatform (KMP), MVVM, Coroutines & Flow, gRPC mobile, BLE, SQLDelight, Mobile CI/CD

**Languages**: Kotlin, C++, Rust, Java, Go, Python, TypeScript, C#, Shell Script

**Performance Engineering**: Lock-Free Data Structures, Concurrency & Threading, Memory Profiling, Cache Optimization, Sub-100ms Latency Systems, Real-Time Pipelines, GPU-Accelerated Computing, eBPF

**SDK & Developer Tools**: SDK Design, API Development, Developer-Facing Libraries, Terraform Providers, CI/CD, Automated Testing, Gradle (KMP), Packaging

**Machine Learning & AI**: PyTorch, TensorFlow, XGBoost, Transformer Models (BERT), Computer Vision (YOLOv8/v11), ONNX Runtime, On-Device Edge AI

**Distributed Systems**: AWS (EC2, ECS, Lambda, Kinesis, EMR, SageMaker), Azure (AKS), Kubernetes, Docker, Terraform, Apache Kafka, Apache Spark

**Core CS**: Data Structures & Algorithms, System Design, Networking (TCP/IP, TLS, DNS, HTTP/3), Operating Systems, Memory Safety

---

## EDUCATION

**Bachelor of Technology - Computer Science & Engineering**  
RCC Institute of Information Technology - Kolkata, India | 2018
