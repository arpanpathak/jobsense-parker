//! Crawler for Hacker News — fetches the monthly "Who is Hiring?" thread
//! and extracts individual job comments.

use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use super::fetcher::Fetcher;
use crate::models::{JobPost, JobSource, SearchConfig};
use super::SourceCrawler;

/// Crawler that extracts job listings from the Hacker News "Who is Hiring?" thread
/// and filters them against the search keywords.
pub struct HackerNewsCrawler;

#[async_trait::async_trait]
impl SourceCrawler for HackerNewsCrawler {
    fn name(&self) -> &str {
        "Hacker News"
    }

    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let fetcher = Fetcher::new()?;

        let now = Utc::now();
        let month = now.format("%B").to_string();
        let year = now.format("%Y").to_string();

        // Build a query that includes the user's keywords so Algolia
        // returns only relevant stories in the first place.
        let keyword_q = if config.keywords.is_empty() {
            String::new()
        } else {
            let joined = config.keywords.join(" ");
            format!("%20{joined}")
        };
        let search_url = format!(
            "https://hn.algolia.com/api/v1/search?query=Who%20is%20Hiring%20{month}%20{year}{keyword_q}&tags=story&hitsPerPage=5"
        );

        let json = fetcher.fetch(&search_url).await?;
        let parsed: serde_json::Value = serde_json::from_str(&json)?;

        let mut posts = Vec::new();
        if let Some(hits) = parsed["hits"].as_array() {
            for hit in hits.iter().take(1) {
                let story_id = hit["objectID"].as_str().unwrap_or("0");
                let item_url =
                    format!("https://hacker-news.firebaseio.com/v0/item/{story_id}.json");
                let item_json = fetcher.fetch(&item_url).await?;
                let item: serde_json::Value = serde_json::from_str(&item_json)?;

                let title = item["title"]
                    .as_str()
                    .unwrap_or("Who is Hiring?")
                    .to_string();

                let kids = item["kids"].as_array().cloned().unwrap_or_default();
                // Pre-compile keyword matcher so we only keep relevant comments.
                let kw_matcher = KeywordMatcher::new(&config.keywords);

                for kid in kids.iter().take(config.max_results) {
                    let comment_id = kid.as_i64().unwrap_or(0);
                    let comment_url =
                        format!("https://hacker-news.firebaseio.com/v0/item/{comment_id}.json");
                    let comment_json = fetcher.fetch(&comment_url).await?;
                    let comment: serde_json::Value =
                        serde_json::from_str(&comment_json)?;

                    let text = comment["text"].as_str().unwrap_or("").to_string();
                    let by = comment["by"].as_str().unwrap_or("anonymous");

                    if text.is_empty() {
                        continue;
                    }

                    // *** KEY FIX: skip comments that don't mention any keyword ***
                    if !kw_matcher.matches(&text) {
                        continue;
                    }

                    let first_line = text.lines().next().unwrap_or("");
                    let company = if first_line.contains('|') {
                        first_line.split('|').next().map(|s| s.trim().to_string())
                    } else {
                        None
                    };

                    let hn_url =
                        format!("https://news.ycombinator.com/item?id={comment_id}");

                    posts.push(JobPost {
                        id: Uuid::new_v4().to_string(),
                        title: format!("{title} -- {by}"),
                        company,
                        location: None,
                        description: text,
                        url: hn_url,
                        source: JobSource::HackerNews,
                        posted_at: None,
                        crawled_at: Utc::now(),
                        salary: None,
                        job_type: None,
                        tags: vec!["hacker-news".to_string(), "who-is-hiring".to_string()],
                    });
                }
            }
        }

        Ok(posts)
    }
}

/// Efficient keyword checker — returns true if the text contains at least
/// one of the user's search keywords (case-insensitive).
struct KeywordMatcher {
    keywords: Vec<String>,
}

impl KeywordMatcher {
    fn new(keywords: &[String]) -> Self {
        Self {
            keywords: keywords.iter().map(|k| k.to_lowercase()).collect(),
        }
    }

    fn matches(&self, text: &str) -> bool {
        if self.keywords.is_empty() {
            return true; // no filter = everything passes
        }
        let lower = text.to_lowercase();
        self.keywords.iter().any(|kw| lower.contains(kw))
    }
}
