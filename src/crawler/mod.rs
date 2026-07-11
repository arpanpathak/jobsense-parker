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

    /// Run all matching crawlers and aggregate their results.
    ///
    /// Only crawlers whose `name()` matches one of the sources in
    /// `config.sources` are invoked. Errors from individual crawlers
    /// are logged to stderr but do not fail the whole operation.
    pub async fn crawl_all(&self, config: &SearchConfig) -> Vec<JobPost> {
        let mut all_posts = Vec::new();

        for crawler in &self.crawlers {
            let name = crawler.name();
            if !config
                .sources
                .iter()
                .any(|s| name.to_lowercase().contains(&s.to_string().to_lowercase()))
            {
                continue;
            }

            match crawler.crawl(config).await {
                Ok(posts) => {
                    eprintln!(
                        "  {} {} posts from {}",
                        "+".green(),
                        posts.len(),
                        name.cyan()
                    );
                    all_posts.extend(posts);
                }
                Err(e) => {
                    eprintln!("  {} {} -- {}", "x".red(), name.cyan(), e);
                }
            }
        }

        all_posts
    }
}
