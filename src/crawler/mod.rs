//! # Job-Source Crawlers
//!
//! This module contains all job-source crawlers and the coordinator that
//! orchestrates them. Each crawler implements the [`SourceCrawler`] trait
//! and is registered in [`CrawlerCoordinator`].
//!
//! ## Architecture
//!
//! ```text
//!                    ┌──────────────────────────────────┐
//!                    │     CrawlerCoordinator            │
//!                    │  (filter by config.sources, then  │
//!                    │   run all concurrently, then      │
//!                    │   post-filter by keywords)        │
//!                    └──────┬──────┬──────┬──────┬──────┘
//!                            │      │      │      │
//!              ┌─────────────┘      │      │      └─────────────┐
//!              ▼                    ▼      ▼                    ▼
//!     ┌────────────┐    ┌────────────┐    ┌────────────┐    ┌──────────┐
//!     │ Remote OK   │    │   Reddit    │    │ HackerNews  │    │ Company  │
//!     │ (tagged     │    │ (JSON API,  │    │ (Algolia +  │    │ (heuristic│
//!     │  JSON API)  │    │  5 subs)    │    │  Firebase)  │    │  scraper) │
//!     └────────────┘    └────────────┘    └────────────┘    └──────────┘
//! ```
//!
//! ## Adding a New Source
//!
//! 1. Create a new module (e.g. `indeed.rs`) with a struct that implements
//!    [`SourceCrawler`]
//! 2. Register it in [`CrawlerCoordinator::new`]
//! 3. Add a variant to [`JobSource`](crate::models::JobSource) if it's a
//!    distinct board (otherwise `Custom` works)

pub mod company;
pub mod fetcher;
pub mod hackernews;
pub mod reddit;
pub mod remoteok;

use anyhow::Result;
use colored::Colorize;
use regex::Regex;

use crate::models::{JobPost, SearchConfig};

/// A single job-source crawler.
///
/// Implementations must be `Send + Sync` so the coordinator can run them
/// concurrently in an async context via [`futures::future::join_all`].
///
/// # Contract
///
/// - `crawl()` should respect [`SearchConfig::keywords`], [`SearchConfig::max_results`],
///   and [`SearchConfig::sources`]
/// - Return only jobs that match the keywords (either via API filtering or
///   internal filtering)
/// - Set meaningful `company`, `title`, `url`, and `source` on each [`JobPost`]
#[async_trait::async_trait]
pub trait SourceCrawler: Send + Sync {
    /// Human-readable name of the source (used for logging and filtering).
    ///
    /// Must match the `Display` representation of the corresponding
    /// [`JobSource`](crate::models::JobSource) variant so that source filtering
    /// in [`CrawlerCoordinator::crawl_all`] works correctly.
    ///
    /// # Examples
    ///
    /// - `"Remote OK"` → matched against `JobSource::RemoteOk` (renders as `"Remote OK"`)
    /// - `"Reddit"` → matched against `JobSource::Reddit`
    /// - `"Hacker News"` → matched against `JobSource::HackerNews`
    fn name(&self) -> &str;

    /// Fetch job posts matching the given search configuration.
    ///
    /// # Errors
    ///
    /// Errors are logged by the coordinator and do not fail the entire crawl.
    /// Common failures: network timeouts, rate limiting, API changes.
    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>>;
}

/// Orchestrates multiple [`SourceCrawler`] instances, runs them concurrently,
/// and post-filters the combined results against the user's search keywords.
///
/// # Concurrency
///
/// All active crawlers (those whose `name()` matches a source in
/// [`SearchConfig::sources`]) are spawned concurrently via
/// [`futures::future::join_all`]. This means a 3-source crawl takes roughly
/// as long as the *slowest* single source, not the sum of all three.
///
/// # Post-filtering
///
/// After all crawlers complete, every job is checked against the search
/// keywords (`job.title`, `job.description`, `job.company`, `job.tags`).
/// Jobs that don't contain at least one keyword are discarded. This is a
/// safety net for crawlers that may not fully respect `config.keywords`.
pub struct CrawlerCoordinator {
    crawlers: Vec<Box<dyn SourceCrawler>>,
}

impl CrawlerCoordinator {
    /// Create a coordinator with all built-in crawlers registered.
    ///
    /// Currently registered:
    ///
    /// | Index | Crawler | Source |
    /// |-------|---------|--------|
    /// | 0 | [`remoteok::RemoteOkCrawler`] | Remote OK JSON API |
    /// | 1 | [`reddit::RedditCrawler`] | Reddit JSON API (5 subs) |
    /// | 2 | [`hackernews::HackerNewsCrawler`] | HN Algolia + Firebase |
    pub fn new() -> Self {
        Self {
            crawlers: vec![
                Box::new(remoteok::RemoteOkCrawler),
                Box::new(reddit::RedditCrawler),
                Box::new(hackernews::HackerNewsCrawler),
            ],
        }
    }

    /// Run all matching crawlers **concurrently** and aggregate their results.
    ///
    /// Only crawlers whose `name()` matches one of the sources in
    /// [`SearchConfig::sources`] are invoked. Errors from individual crawlers
    /// are logged to stderr but do not fail the whole operation.
    ///
    /// ## Post-filtering
    ///
    /// After collection, results are filtered against the search keywords so
    /// that every returned post is actually relevant to what the user asked for.
    /// Jobs whose combined `title + description + company + tags` contain none
    /// of the keywords are discarded. Logs a count if any were removed.
    pub async fn crawl_all(&self, config: &SearchConfig) -> Vec<JobPost> {
        let futures: Vec<_> = self
            .crawlers
            .iter()
            .filter(|crawler| {
                let name = crawler.name();
                config
                    .sources
                    .iter()
                    .any(|s| name.to_lowercase().contains(&s.to_string().to_lowercase()))
            })
            .map(|crawler| {
                let name = crawler.name().to_string();
                async move {
                    let result = crawler.crawl(config).await;
                    match &result {
                        Ok(posts) => {
                            eprintln!(
                                "  {} {} posts from {}",
                                "+".green(),
                                posts.len(),
                                name.cyan()
                            );
                        }
                        Err(e) => {
                            eprintln!("  {} {} -- {}", "x".red(), name.cyan(), e);
                        }
                    }
                    result
                }
            })
            .collect();

        let mut all_posts = Vec::new();

        for result in futures::future::join_all(futures).await {
            if let Ok(posts) = result {
                all_posts.extend(posts);
            }
        }

        // Post-filter: remove jobs that don't match search keywords.
        // Uses a two-pass approach:
        //   1. TITLE-GATE — at least one keyword MUST appear in the job TITLE.
        //      This eliminates "Senior Vice President" when searching
        //      "senior software engineer" (title lacks "software" or "engineer").
        //   2. SOFT-AND — of the remaining, require N of K keywords in full text.
        if !config.keywords.is_empty() {
            let before = all_posts.len();
            let kw_count = config.keywords.len();
            let kw_lower: Vec<String> = config.keywords.iter().map(|k| k.to_lowercase()).collect();

            // Minimum keywords required: 1 of 1, 2 of 2, 2 of 3, half of 4+
            let min_required = if kw_count == 1 {
                1
            } else if kw_count <= 3 {
                kw_count - 1 // 2 of 2, 2 of 3
            } else {
                kw_count / 2
            };

            // Pass 1: Title-gate — min_required keywords must appear in TITLE.
            // This eliminates "Senior Vice President" when searching
            // "senior software engineer" because the title only has "senior".
            //
            // Additionally: if the query contains role-type keywords
            // (e.g. "software" in "senior software engineer"), at least one
            // such keyword must appear in the title. This eliminates
            // "Senior Quality Engineer" — it hits the soft-AND minimum (2 of 3
            // with "senior"+"engineer") but lacks "software" or any role type.
            all_posts.retain(|job| {
                let title_lower = job.title.to_lowercase();

                // Check role-type keywords first
                let role_keywords = ["software", "frontend", "backend", "full.?stack", "fullstack",
                    "devops", "data", "ml", "ai", "machine.?learning", "platform", "security",
                    "systems", "network", "site.?reliability", "sre", "qa", "test",
                    "ios", "android", "mobile", "embedded", "firmware", "hardware",
                    "infrastructure", "cloud", "rust", "go", "python", "java", "javascript",
                    "typescript", "react", "angular", "vue", "node", "devsecops"];
                let has_role_keyword = role_keywords.iter().any(|rk| {
                    if rk.contains(".?") {
                        let re = regex::Regex::new(rk).unwrap();
                        re.is_match(&title_lower)
                    } else {
                        title_lower.contains(rk)
                    }
                });
                // If the query has role keywords and none match in the title, drop it
                let query_has_role = role_keywords.iter().any(|rk| {
                    let rk_lower = rk.replace(".?", "");
                    kw_lower.iter().any(|kw| kw.contains(&rk_lower))
                });
                if query_has_role && !has_role_keyword {
                    return false;
                }

                // Standard title keyword count
                let title_matches = kw_lower.iter().filter(|kw| title_lower.contains(kw.as_str())).count();
                title_matches >= min_required
            });

            // Pass 2: Soft-AND on full text (NOT including tags — they
            // contain platform-level tag dumps that pollute matching).
            all_posts.retain(|job| {
                let text = format!(
                    "{} {} {}",
                    job.title,
                    job.description,
                    job.company.as_deref().unwrap_or(""),
                )
                .to_lowercase();
                let matches = kw_lower.iter().filter(|kw| text.contains(kw.as_str())).count();
                matches >= min_required
            });

            let removed = before - all_posts.len();
            if removed > 0 {
                eprintln!(
                    "  {} {} posts filtered out (keyword not in title or insufficient matches)",
                    "-".yellow(),
                    removed
                );
            }
        }

        all_posts
    }
}
