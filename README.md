<p align="center">
  <br>
  <img src="https://img.shields.io/badge/rust-2024-edition?style=for-the-badge&logo=rust&logoColor=white&color=black" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT-blue?style=for-the-badge&color=blue" alt="MIT License">
  <img src="https://img.shields.io/badge/status-alpha-orange?style=for-the-badge" alt="Alpha">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen?style=for-the-badge" alt="PRs Welcome">
</p>

<p align="center">
  <pre align="center">
   ╔══════════════════════════════════════════════════╗
   ║   🔍  JobSense-Parker  v0.1.0                  ║
   ║   ────────────────────────────────────────────   ║
   ║   Hunt the internet for your next gig.          ║
   ║   (LinkedIn-free zone)                          ║
   ╚══════════════════════════════════════════════════╝
  </pre>
</p>

<p align="center">
  <b>jobense-parker</b> is an interactive CLI that crawls job boards, social media,
  and hiring threads — then matches them against your resume so you only see
  what's relevant. No recruiter spam. No LinkedIn. Just clean, scored results.
</p>

---

<details open>
<summary><b>📦 Install</b></summary>
<br>

```bash
# Clone it
git clone https://github.com/yourname/jobsense-parker
cd jobsense-parker

# Build it
cargo build --release

# Run it
./target/release/jobsense-parker
```

> **Prerequisites:** Rust 1.75+ ([install via rustup](https://rustup.rs/))

</details>

---

<details open>
<summary><b>🎮 Usage</b></summary>

<br>

When you launch the tool, you're greeted with an interactive menu:

```
╔══════════════════════════════════════════════════╗
║  🔍 JobSense-Parker  v0.1.0                     ║
║  Hunt the internet for your next gig.            ║
║  (LinkedIn-free zone)                            ║
╚══════════════════════════════════════════════════╝

? jobense-parker ›
❯ 🔍  Scan jobs (all sources)
  🎯  Search with custom query
  📋  View results (no results)
  📄  Load resume (not loaded)
  👤  Show current resume
  🔧  Filter / sort results
  🚪  Quit
```

### 1️⃣ Load your resume

Paste skills, a structured JSON/YAML resume, or a file path:

```
? Resume text (paste or type)
> Rust, TypeScript, React, Docker, Kubernetes, Python, AWS, PostgreSQL
✓ Resume loaded! (8 skills, 0 roles)
```

### 2️⃣ Scan

Hit <kbd>Enter</kbd> on **Scan jobs** and watch the crawlers go to work:

```
🌐 Scanning with keywords: Rust, TypeScript, React, Docker, Kubernetes, Python, AWS, PostgreSQL

  ✓ 12 posts from Indeed
  ✓ 8 posts from Google Jobs
  ✓ 15 posts from Reddit
  ✓ 22 posts from Hacker News

📊 Found 57 raw job posts. Matching against resume...
✓ 31 matched results (above threshold)
```

### 3️⃣ Browse results

Paginated, scored, with matched vs missing skills:

```
📋 Results (page 1/4 — 31 total)
────────────────────────────────────────────────────────────

 1. Senior Rust Engineer           92% [████████████████████] [Indeed]
     Company: Cloudflare
     Location: Remote / US
     ✓ rust, python, kubernetes, aws
     ✗ terraform
     https://www.indeed.com/.../job

 2. Full-Stack TypeScript Dev     71% [███████████████░░░░░] [Hacker News]
     Company: Vercel
     ✓ typescript, react, aws
     ✗ kubernetes, docker
     https://news.ycombinator.com/item?id=...
```

### 4️⃣ Filter & sort

```
? Filter results ›
❯ Sort by score (high → low)
  Sort by score (low → high)
  Show only high matches (>70%)
  Show only medium matches (40-70%)
  Show only low matches (<40%)
  Reset filters
  Back
```

</details>

---

<details open>
<summary><b>🗺️ Architecture</b></summary>

<br>

```
                         ┌─────────────┐
                         │   CLI Menu   │
                         │  (dialoguer) │
                         └──────┬──────┘
                                │ commands
                                ▼
          ┌─────────────────────────────────────┐
          │              App                     │
          │  ┌──────────┐  ┌──────────────────┐ │
          │  │  Matcher  │  │ CrawlerCoordinator│ │
          │  │  (scorer) │  │  (orchestrator)  │ │
          │  └──────────┘  └────────┬─────────┘ │
          └─────────────────────────┼────────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
              ▼                     ▼                     ▼
        ┌──────────┐         ┌──────────┐          ┌────────────┐
        │ Indeed   │         │  Google  │          │   Reddit   │
        │ scraper  │         │  Jobs    │          │  scraper   │
        └──────────┘         └──────────┘          └────────────┘
                                                           :
        ┌────────────┐        ┌──────────────────┐         :
        │ Hacker News │       │   HTTP Fetcher    │
        │ (Algolia   │       │  (UA rotation +   │
        │  API)      │       │   polite delays)  │
        └────────────┘        └──────────────────┘
```

### Modules

| Module | Responsibility |
|--------|---------------|
| `models/` | Data types: `JobPost`, `Resume`, `MatchResult`, `SearchConfig` |
| `crawler/` | `SourceCrawler` trait + per-source implementations + `Fetcher` with UA rotation |
| `matcher/` | Resume parsing, fuzzy matching (Jaro-Winkler), score computation |
| `cli/` | Interactive menu, paginated viewer, filter/sort controls |

### Scoring formula

| Factor | Weight | Detail |
|--------|--------|--------|
| Skill overlap | **50%** | Exact + fuzzy match of resume skills against job text |
| Keyword/role match | **25%** | Role titles & user keywords found in job posting |
| Title bonus | **10%** | Job title contains a resume role title |
| Location bonus | **10%** | Preferred location matches job location |
| Job type bonus | **5%** | Preferred type (remote, contract, etc.) matches |

Fuzzy matching uses the [Jaro-Winkler distance](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance) with a 0.85 threshold — catches typos and close variants.

</details>

---

<details open>
<summary><b>🔌 Adding a custom source</b></summary>
<br>

Implement the `SourceCrawler` trait:

```rust
use async_trait::async_trait;
use jobsense_parker::crawler::SourceCrawler;
use jobsense_parker::models::{JobPost, SearchConfig};

pub struct MyJobBoard;

#[async_trait]
impl SourceCrawler for MyJobBoard {
    fn name(&self) -> &str {
        "My Board"
    }

    async fn crawl(&self, config: &SearchConfig) -> anyhow::Result<Vec<JobPost>> {
        // 1. Fetch & parse HTML/JSON
        // 2. Return Vec<JobPost>
        todo!()
    }
}
```

Then register it in [`CrawlerCoordinator::new()`](src/crawler/mod.rs).

</details>

---

<details open>
<summary><b>🧪 Tests</b></summary>
<br>

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Specific test
cargo test test_score_rust_job
```

</details>

---

<details open>
<summary><b>📄 License</b></summary>
<br>

MIT — see [LICENSE](LICENSE). Do what you want with it.

</details>

---

<p align="center">
  <sub>Built with 🦀 Rust &middot; Crawls ethically with polite delays &middot; Not affiliated with LinkedIn</sub>
</p>
