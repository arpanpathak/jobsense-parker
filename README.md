<div align="center">

# 🔍 jobsense-parker

**Terminal-based job search engine** — crawls job boards, hiring threads, and **80+ company career sites**, then scores every posting against your resume.

![Rust](https://img.shields.io/badge/rust-1.75%2B-black?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
![Status](https://img.shields.io/badge/status-beta-brightgreen?style=flat-square)
![PRs](https://img.shields.io/badge/PRs-welcome-orange?style=flat-square)
![Release](https://img.shields.io/github/v/release/arpanpathak/jobsense-parker?style=flat-square)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey?style=flat-square)

```text
╔══════════════════════════════════════════════════════╗
║  JobSense-Parker  v0.4.0                             ║
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
- **Smart scoring**: Weighted algorithm — title skill match 35%, skill coverage 30%, keyword ratio 15%, role title 10%, location 5%, job type 5% — with word-boundary + alias-aware matching (`k8s` ≈ `kubernetes`, `golang` ≈ `go`)
- **Auto-discovery**: Company names extracted from job posts are cached locally — future scans automatically crawl their career pages
- **Vim-style results viewer**: Full-screen paginated browser with `j`/`k` navigation, `Enter` to open URLs, `?` for help
- **OSC 8 clickable links**: Cmd+click (macOS) or Ctrl+click (Linux/Windows) any job URL to open in your browser
- **Progress spinner**: Real-time feedback during crawling — no more silent waiting
- **Auto-fill applications**: Press `a` on a job — a visible Chrome window opens and **types** the application form fields with your profile via CDP (real key events, no chromedriver), then stays open for you to review and Submit
- **Fully concurrent**: All sources are crawled simultaneously; HN comments are fetched in parallel
- **Persistent**: Resumes, preferences, company database, scan history — all saved to `~/.jobsense-parker/`

---

## 📦 Install

Prebuilt binaries for **macOS (Intel + Apple Silicon), Linux (x86_64 + arm64) and Windows (x86_64)** are attached to every [GitHub Release](https://github.com/arpanpathak/jobsense-parker/releases).

### macOS / Linux — one-liner

```bash
curl -fsSL https://raw.githubusercontent.com/arpanpathak/jobsense-parker/master/install.sh | bash
```

Pin a version:

```bash
curl -fsSL https://raw.githubusercontent.com/arpanpathak/jobsense-parker/master/install.sh | bash -s -- v0.4.0
```

### Windows — one-liner (PowerShell)

```powershell
irm https://raw.githubusercontent.com/arpanpathak/jobsense-parker/master/install.ps1 | iex
```

### Cargo install (needs Rust)

```bash
cargo install jobsense-parker
```

### From source

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
? jobsense-parker ›
❯ Scan jobs (all sources + career sites)
  Search with custom query
  View results (no results)
  Company career sites (87 companies cached)
  Load resume (not loaded)
  Show current resume
  Filter / sort results
  Scan history
  Set profile (for auto-fill)
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
| `a` | Auto-fill the application form (visible Chrome, real typing) |
| `q` / `Esc` | Back to menu |
| `?` | Toggle keybinding help overlay |

All URLs support **Cmd+click** (macOS) or **Ctrl+click** (Linux/Windows) to open directly.

### 🤖 Auto-Fill Applications (visible browser)

Press `a` on any job and jobsense-parker launches Chrome in a **visible window**,
navigates to the job page, and **types** common application form fields with
your profile using Chrome DevTools Protocol — real key events, so React-based
ATS forms (Greenhouse, Lever, ...) register the input correctly (no chromedriver
needed):

- Name (first/last/full), email, phone, location
- LinkedIn and GitHub / portfolio URLs
- Detects fields by `name` (exact or substring), `autocomplete`, `placeholder`,
  `aria-label`, and `<label>` text

Set your profile once from the menu (**Set profile (for auto-fill)**) — it is
stored in `~/.jobsense-parker/preferences.json`. The Chrome window **stays open
after filling** so you can review the form and click Submit yourself. Requires a
local Chrome/Chromium install.

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
  │  • Title skill match (35%)                    │
  │  • Skill coverage (30%)                       │
  │  • Keyword ratio (15%)                        │
  │  • Role-title match (10%)                     │
  │  • Location & job-type (5% each)              │
  │  • Word-boundary + alias matching             │
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
| **Title skill match** | 35% | Any of your resume skills appearing in the job TITLE (e.g. "Rust Engineer") — the strongest possible signal |
| **Skill coverage** | 30% | How many of your skills appear in the job text; saturates at `min(10, total_skills)` so broad resumes aren't penalised for knowing more |
| **Keyword ratio** | 15% | Fraction of resume keywords found in the job text |
| **Role-title match** | 10% | Job title contains one of your role titles ("software engineer", "developer") |
| **Location match** | 5% | Job location aligns with your preferred location |
| **Job-type match** | 5% | Job type matches your preferred type (e.g. "remote") |

Skills are matched with **word boundaries** — so `"go"` doesn't match `"google"` and `"rust"` doesn't match `"trust"` — plus **aliases**: `k8s` ≈ `kubernetes`, `golang` ≈ `go`, `cpp` ≈ `c++`, `js` ≈ `javascript`, `postgres` ≈ `postgresql`. Role titles and locations additionally use the [Jaro-Winkler distance](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance) with a 0.85 threshold to catch typos and close variants.

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
