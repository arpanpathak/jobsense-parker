<div align="center">

# 🔍 jobsense-parker

**Terminal-based job search engine** — crawls job boards, hiring threads, and **80+ company career sites**, then scores every posting against your resume.

![Rust](https://img.shields.io/badge/rust-1.75%2B-black?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
![Status](https://img.shields.io/badge/status-beta-brightgreen?style=flat-square)
![PRs](https://img.shields.io/badge/PRs-welcome-orange?style=flat-square)

```text
╔══════════════════════════════════════════════════════╗
║  🔍  JobSense-Parker  v0.3                          ║
║  ──────────────────────────────────────────────────   ║
║  Hunt the internet for your next gig.                ║
║  Type '?' at any results view for keybindings        ║
║  (LinkedIn-free zone)                                ║
╚══════════════════════════════════════════════════════╝
```

No recruiter spam. No LinkedIn. Just clean, scored results from real job sources.

</div>

---

## ✨ Features

- **4 job sources**: Remote OK, Reddit (5 hiring subreddits), Hacker News "Who is Hiring?", and **80+ company career sites** — auto-crawled during every scan
- **Resume matching**: Paste your resume or a PDF/JSON file — skills, roles, and keywords are extracted automatically and matched against every job
- **Smart scoring**: Weighted algorithm (skill overlap 50%, keywords 25%, role title 10%, location 10%, job type 5%) with Jaro-Winkler fuzzy matching
- **Auto-discovery**: Company names extracted from job posts are cached locally — future scans automatically crawl their career pages
- **Vim-style results viewer**: Full-screen paginated browser with `j`/`k` navigation, `Enter` to open URLs, `?` for help
- **OSC 8 clickable links**: Cmd+click (macOS) or Ctrl+click (Linux/Windows) any job URL to open in your browser
- **Progress spinner**: Real-time feedback during crawling — no more silent waiting
- **Fully concurrent**: All sources are crawled simultaneously; HN comments are fetched in parallel
- **Persistent**: Resumes, preferences, company database, scan history — all saved to `~/.jobsense-parker/`

---

## 📦 Install

```bash
# Prerequisites: Rust 1.75+ (install via https://rustup.rs/)
git clone https://github.com/arpanpathak/jobsense-parker.git
cd jobsense-parker
cargo build --release

# Run the interactive menu
./target/release/jobsense-parker
```

---

## 🎮 Quick Start

### Interactive mode (no arguments)

```bash
./target/release/jobsense-parker
```

Shows the menu:

```text
? jobense-parker ›
❯ Scan jobs (all sources + career sites)
  Search with custom query
  View results (no results)
  Company career sites (87 companies cached)
  Load resume (not loaded)
  Show current resume
  Filter / sort results
  Scan history
  Quit
```

### Non-interactive mode (flags)

```bash
# Search from the command line
./target/release/jobsense-parker --search "rust engineer"

# Scan with a loaded resume
./target/release/jobsense-parker --resume ~/Resume.pdf --scan

# View cached companies
./target/release/jobsense-parker --companies

# Add a company career site
./target/release/jobsense-parker --add-company "Tesla" "https://www.tesla.com/careers"
```

---

## 📋 Vim-Style Results Viewer

Select "View results" from the menu to enter the full-screen paginated browser:

```text
  ▸ results (page 1/3 · 25 total)
  ────────────────────────────────────────────────────────────

   1. Senior Rust Engineer 78% [Remote OK] @ Stripe
       https://stripe.com/jobs/engineering/senior-rust-engineer
       + rust, distributed-systems, api-design
       - kubernetes, aws

   2. Backend Engineer     65% [Hacker News] @ Jane Street
       https://news.ycombinator.com/item?id=12345678
       + ocaml, python
       - kubernetes, docker, aws

  ▸3. Full Stack Developer 45% [Company Careers] @ Shopify
       https://shopify.com/careers/fullstack-developer-123

  [j↓ k↑  n→ p←  g/G  Enter:open  ?:help  q:quit]  ▸ Full Stack Developer
```

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `n` / `→` | Next page |
| `p` / `←` | Previous page |
| `g` | First page |
| `G` | Last page |
| `Enter` / `o` | Open job URL in browser |
| `q` / `Esc` | Back to menu |
| `?` | Toggle keybinding help overlay |

All URLs support **Cmd+click** (macOS) or **Ctrl+click** (Linux/Windows) to open directly.

---

## 🏢 Company Career Sites

On first run, the tool seeds a local database with **80+ major tech companies** and their careers-page URLs:

```
Big Tech:     Google, Meta, Apple, Amazon, Microsoft, Netflix, Spotify, Stripe, Shopify, GitLab ...
Fintech:      Stripe, Coinbase, Robinhood, Plaid, Jane Street, Citadel, Two Sigma ...
Enterprise:   Salesforce, Adobe, Atlassian, Datadog, MongoDB, Databricks, HashiCorp ...
Cloud:        Cloudflare, Snowflake, Confluent, Elastic, Vercel, Netlify, Supabase ...
Gaming:       Riot Games, Blizzard, Epic Games, Unity, Roblox ...
```

During every scan, new companies are **auto-discovered** from job postings and added to the cache. The career-page crawler uses URL heuristics to find job listings (supports Greenhouse, Lever, Workday, and standard career portals).

```bash
# List cached companies
./target/release/jobsense-parker --companies

# Add a company manually
./target/release/jobsense-parker --add-company "MyCorp" "https://mycorp.com/careers"

# Remove a company
./target/release/jobsense-parker --remove-company "Twitter/X"
```

---

## 🗂 Project Structure

```
jobsense-parker/
├── Cargo.toml                  # Dependencies & metadata
├── Cargo.lock
├── LICENSE                     # MIT
├── README.md                   # ← you are here
└── src/
    ├── main.rs                 # CLI entry point (flag parsing)
    ├── lib.rs                  # Crate root, module declarations
    ├── cli/
    │   ├── mod.rs              # Interactive menu loop, command dispatch
    │   └── views.rs            # Terminal rendering (banner, vim viewer, help)
    ├── crawler/
    │   ├── mod.rs              # CrawlerCoordinator (concurrency + post-filter)
    │   ├── fetcher.rs          # HTTP client (concurrent-safe, 15s timeout)
    │   ├── remoteok.rs         # Remote OK job board (tagged JSON API)
    │   ├── reddit.rs           # Reddit hiring subreddits (JSON API)
    │   ├── hackernews.rs       # HN "Who is Hiring?" (Algolia + Firebase)
    │   └── company.rs          # Company career-site crawler (heuristics)
    ├── matcher/
    │   ├── mod.rs              # Resume-to-job matching engine
    │   └── scoring.rs          # Scoring algorithm (weights, fuzzy match)
    ├── models/
    │   └── mod.rs              # Data types (JobPost, Resume, MatchResult, etc.)
    └── storage/
        └── mod.rs              # JSON persistence to ~/.jobsense-parker/
```

---

## 🏗 Architecture

```text
  ┌──────────────┐
  │  User Input   │  Query string or resume file (PDF/JSON/YAML/text)
  └──────┬───────┘
         │
         ▼
  ┌───────────────────────────────────────────────────────────────┐
  │                    CrawlerCoordinator                         │
  │  (filter sources, then run ALL concurrently via join_all)     │
  └────┬──────────┬──────────┬──────────┬────────────────────────┘
       │          │          │          │
       ▼          ▼          ▼          ▼
  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────────┐
  │Remote  │ │ Reddit │ │Hacker  │ │ Company    │   ← concurrent
  │ OK     │ │ (JSON  │ │ News   │ │ Career     │
  │(tagged │ │  API,  │ │(Algolia│ │ Sites      │
  │ JSON)  │ │ 5 subs)│ │+Firebase│ │ (80+ seeded)│
  └────────┘ └────────┘ └────────┘ └────────────┘
       │          │          │          │
       └──────────┴──────────┴──────────┘
                        │
                        ▼ post-filter by keywords
                        │
                        ▼ auto-discover companies from job posts
                        │
                        ▼
  ┌──────────────────────────────────────────────┐
  │                 Matcher                       │
  │  score_all(jobs) → Vec<MatchResult>           │
  │  • Skill overlap (50%)                        │
  │  • Keyword ratio (25%)                        │
  │  • Role-title match (10%)                     │
  │  • Location match (10%)                       │
  │  • Job-type match (5%)                        │
  │  • Jaro-Winkler fuzzy matching                │
  └──────────────────┬───────────────────────────┘
                     │
                     ▼ sorted by score (desc)
                     │
                     ▼
  ┌──────────────────────────────────────────────┐
  │          Vim-Style Results Viewer             │
  │  j/k/Enter/q/?  →  Terminal (with OSC 8     │
  │                      clickable links)        │
  └──────────────────────────────────────────────┘
```

---

## 📟 CLI Reference

| Flag | Description | Example |
|------|-------------|---------|
| *(no args)* | Interactive menu | `jobsense-parker` |
| `--help`, `-h` | Show help | |
| `--scan`, `-s` | Scan all sources with loaded resume | `jobsense-parker --scan` |
| `--search <query>` | Search with custom query | `jobsense-parker --search "rust engineer"` |
| `--resume`, `-r <file>` | Load resume file | `jobsense-parker -r ~/resume.pdf` |
| `--results` | View last cached results | |
| `--history` | Show scan history | |
| `--companies` | List cached companies | |
| `--add-company <name> <url>` | Add a company career site | `--add-company "Tesla" "https://tesla.com/careers"` |
| `--remove-company <name>` | Remove a company | `--remove-company "Twitter/X"` |

### Persisted data (`~/.jobsense-parker/`)

| File | Purpose |
|------|---------|
| `resume.json` | Parsed resume (skills, roles, keywords) |
| `preferences.json` | User preferences (sources, max results) |
| `companies.json` | 80+ seeded + auto-discovered companies |
| `queries.json` | Recent search queries (capped at 50) |
| `scan_history.json` | Scan records (capped at 100) |
| `last_results.json` | Most recent match results |

---

## 🔬 Scoring Algorithm

| Component | Weight | How it works |
|-----------|--------|-------------|
| **Skill ratio** | 50% | `matched_skills / total_skills` — what fraction of your skills appear in the job description? |
| **Keyword ratio** | 25% | `matched_keywords / total_keywords` — broad keyword overlap |
| **Role-title match** | 10% | Does the job title contain one of your role titles? Uses Jaro-Winkler fuzzy match. |
| **Location match** | 10% | Does the job location contain your preferred location? Fuzzy matched. |
| **Job-type match** | 5% | Does the job type match your preferred type (e.g. "remote")? |

Fuzzy matching uses the [Jaro-Winkler distance](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance) with a 0.85 threshold — catches typos and close variants.

---

## 🔌 Adding a Custom Source

Implement the `SourceCrawler` trait and register it in `CrawlerCoordinator`:

```rust
use async_trait::async_trait;
use jobsense_parker::crawler::SourceCrawler;
use jobsense_parker::models::{JobPost, SearchConfig};

pub struct MyJobBoard;

#[async_trait]
impl SourceCrawler for MyJobBoard {
    fn name(&self) -> &str { "My Board" }

    async fn crawl(&self, config: &SearchConfig) -> anyhow::Result<Vec<JobPost>> {
        // Fetch API/HTML, parse, filter by config.keywords
        // Return Vec<JobPost>
        todo!()
    }
}
```

Then in [`src/crawler/mod.rs`](src/crawler/mod.rs), add it to `CrawlerCoordinator::new()`.

---

## 🧪 Tests

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_score_rust_job

# Doc tests only
cargo test --doc
```

---

## 📄 License

MIT — see [LICENSE](LICENSE). Go build something cool.

---

<div align="center">
  <sub>Built with 🦀 Rust · Crawls ethically with no artificial delays · Not affiliated with LinkedIn</sub>
</div>
