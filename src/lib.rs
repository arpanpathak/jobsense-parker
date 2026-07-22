//! # jobsense-parker — Terminal-Based Job Search Engine
//!
//! Crawls multiple job sources (Remote OK, Reddit, Hacker News, and company
//! career sites), then scores each posting against your resume using fuzzy
//! text matching. All results and data are persisted locally.
//!
//! ## Quick Start
//!
//! ```text
//! # Interactive session (full menu)
//! $ jobsense-parker
//!
//! # One-shot search
//! $ jobsense-parker --search "rust engineer san francisco"
//!
//! # Load a resume and scan everything
//! $ jobsense-parker --resume ~/documents/Resume.pdf --scan
//! ```
//!
//! ## Architecture
//!
//! The pipeline is:
//!
//! ```text
//!  User query / resume
//!         │
//!         ▼
//!  ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
//!  │ CrawlerCoord│────▶│ Remote OK    │────▶│             │
//!  │ inator      │     │ Reddit       │     │  Matcher    │
//!  │ (concurrent)│────▶│ Hacker News  │────▶│  (scoring)  │────▶ Results
//!  │             │     │ Company      │     │             │
//!  └─────────────┘     │ Career Sites │     └─────────────┘
//!                      └──────────────┘
//! ```
//!
//! ## Modules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`cli`] | Interactive terminal menu + non-interactive flag dispatch |
//! | [`crawler`] | Concurrent job-source fetchers (boards + career sites) |
//! | [`matcher`] | Resume-to-job scoring engine with fuzzy matching |
//! | [`models`] | Shared data types (JobPost, Resume, MatchResult, etc.) |
//! | [`storage`] | JSON persistence layer (resumes, history, companies) |
//!
//! ## Data Flow
//!
//! 1. **Input**: User provides a resume (PDF/JSON/YAML/text) or a plain-text query
//! 2. **Crawl**: [`crawler::CrawlerCoordinator`] dispatches the query to all active
//!    sources concurrently. Each source fetches job listings from its respective API
//!    or HTML endpoint.
//! 3. **Filter**: Results are post-filtered against the search keywords so
//!    every returned post is relevant.
//! 4. **Auto-discover**: Company names are extracted from job posts and cached
//!    locally for future career-site crawling.
//! 5. **Company crawl**: The [`crawler::company::CompanyCrawler`] visits every
//!    cached company's careers page and extracts job listings using heuristics.
//! 6. **Match**: The [`matcher::Matcher`] scores each job against the resume
//!    using skill/keyword overlap, role-title matching, location, and job type.
//! 7. **View**: Results are presented in a vim-style paginated terminal viewer
//!    with OSC 8 clickable links and keyboard navigation.
//!
//! ## Persistence
//!
//! All data lives under `~/.jobsense-parker/`:
//! - `resume.json` — structured resume data (skills, roles, keywords)
//! - `preferences.json` — user preferences (sources, max results)
//! - `companies.json` — cached companies with careers-page URLs (80+ pre-seeded)
//! - `queries.json` — recent search queries (capped at 50)
//! - `scan_history.json` — scan records with timestamps, scores, counts
//! - `last_results.json` — most recent match results

pub mod applicant;
pub mod cli;
pub mod crawler;
pub mod matcher;
pub mod models;
pub mod resume;
pub mod storage;
