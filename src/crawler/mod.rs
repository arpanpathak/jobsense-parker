pub mod fetcher;
pub mod indeed;
pub mod google;
pub mod reddit;
pub mod hackernews;

use anyhow::Result;
use colored::Colorize;

use crate::models::{JobPost, SearchConfig};

#[async_trait::async_trait]
pub trait SourceCrawler: Send + Sync {
    fn name(&self) -> &str;
    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>>;
}

pub struct CrawlerCoordinator {
    crawlers: Vec<Box<dyn SourceCrawler>>,
}

impl CrawlerCoordinator {
    pub fn new() -> Self {
        Self {
            crawlers: vec![
                Box::new(indeed::IndeedCrawler),
                Box::new(google::GoogleJobsCrawler),
                Box::new(reddit::RedditCrawler),
                Box::new(hackernews::HackerNewsCrawler),
            ],
        }
    }

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
