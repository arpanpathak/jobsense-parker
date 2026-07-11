//! Crawler for Reddit — searches hiring-related subreddits for job posts
//! using Reddit's JSON API (more reliable than HTML scraping).

use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::fetcher::Fetcher;
use crate::models::{JobPost, JobSource, SearchConfig};
use super::SourceCrawler;

/// Crawler that searches multiple hiring subreddits for job posts via JSON API.
pub struct RedditCrawler;

#[async_trait::async_trait]
impl SourceCrawler for RedditCrawler {
    fn name(&self) -> &str {
        "Reddit"
    }

    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let fetcher = Fetcher::new()?;
        let query = urlencode(&config.keywords.join(" "));

        let subreddits = ["forhire", "jobbit", "remotejobs", "jobs", "cscareerquestions"];
        let mut posts = Vec::new();
        let per_sub = config.max_results / subreddits.len();

        for sub in subreddits {
            // Use Reddit's JSON API instead of HTML scraping.
            // The .json endpoint is faster and doesn't break when HTML changes.
            let url = format!(
                "https://www.reddit.com/r/{sub}/search.json?q={query}&sort=new&restrict_sr=on&t=week&limit={per_sub}"
            );

            let body = match fetcher.fetch(&url).await {
                Ok(b) => b,
                Err(_) => continue,
            };

            let parsed: serde_json::Value = match serde_json::from_str(&body) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let children = parsed["data"]["children"]
                .as_array()
                .map(|c| c.to_vec())
                .unwrap_or_default();

            for child in children {
                let data = &child["data"];
                let title = data["title"].as_str().unwrap_or("").trim().to_string();
                if title.is_empty() {
                    continue;
                }

                // Skip non-hiring posts
                let lower = title.to_lowercase();
                let is_hiring = lower.contains("hiring") || lower.contains("[hiring]")
                    || lower.contains("job") || lower.contains("position")
                    || lower.contains("opening") || lower.contains("opportunity")
                    || lower.contains("looking for") || lower.contains("need a")
                    || lower.contains("we are") || lower.contains("full-time")
                    || lower.contains("part-time") || lower.contains("contract")
                    || lower.contains("remote");

                if !is_hiring {
                    continue;
                }

                let url = data["url"].as_str().unwrap_or("").to_string();
                let url = if url.starts_with('/') {
                    format!("https://www.reddit.com{url}")
                } else {
                    url
                };

                let selftext = data["selftext"].as_str().unwrap_or("").to_string();
                let created = data["created_utc"].as_f64()
                    .and_then(|ts| DateTime::from_timestamp(ts as i64, 0));

                posts.push(JobPost {
                    id: Uuid::new_v4().to_string(),
                    title,
                    company: None,
                    location: None,
                    description: selftext,
                    url,
                    source: JobSource::Reddit,
                    posted_at: created,
                    crawled_at: Utc::now(),
                    salary: None,
                    job_type: None,
                    tags: vec![sub.to_string()],
                });
            }
        }

        Ok(posts)
    }
}

/// Simple URL encoding that replaces spaces with `+`.
fn urlencode(s: &str) -> String {
    s.replace(' ', "+")
}
