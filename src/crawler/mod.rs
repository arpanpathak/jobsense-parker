//! Job-board crawlers that collect job postings from various sources.
//!
//! Each crawler implements the [`SourceCrawler`] trait. The
//! [`CrawlerCoordinator`] manages all registered crawlers and dispatches
//! searches to the applicable ones based on [`SearchConfig`].

pub mod fetcher;
pub mod remoteok;
pub mod reddit;
pub mod hackernews;

use anyhow::Result;
use colored::Colorize;

use crate::models::{JobPost, SearchConfig};

/// A single job-source crawler.
///
/// Implementations must be `Send + Sync` so the coordinator can run them
/// concurrently (or sequentially) in an async context.
#[async_trait::async_trait]
pub trait SourceCrawler: Send + Sync {
    /// Human-readable name of the source (used for logging and filtering).
    fn name(&self) -> &str;
    /// Fetch job posts matching the given search configuration.
    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>>;
}

/// Orchestrates multiple [`SourceCrawler`] instances.
pub struct CrawlerCoordinator {
    crawlers: Vec<Box<dyn SourceCrawler>>,
}

impl CrawlerCoordinator {
    /// Create a coordinator with all built-in crawlers registered.
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
    /// `config.sources` are invoked. Errors from individual crawlers
    /// are logged to stderr but do not fail the whole operation.
    ///
    /// After collection, results are **filtered** against the search
    /// keywords so that every returned post is actually relevant to
    /// what the user asked for.
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

        // Post-filter: throw out jobs that don't mention any search keyword.
        // This catches crawlers that may not filter internally (or do it poorly).
        if !config.keywords.is_empty() {
            let before = all_posts.len();
            all_posts.retain(|job| {
                let text = format!(
                    "{} {} {} {}",
                    job.title,
                    job.description,
                    job.company.as_deref().unwrap_or(""),
                    job.tags.join(" ")
                )
                .to_lowercase();
                config.keywords.iter().any(|kw| text.contains(&kw.to_lowercase()))
            });
            let removed = before - all_posts.len();
            if removed > 0 {
                eprintln!(
                    "  {} {} posts filtered out (didn't match keywords)",
                    "-".yellow(),
                    removed
                );
            }
        }

        all_posts
    }
}
