//! Crawler for Hacker News — fetches the monthly "Who is Hiring?" thread
//! and extracts individual job comments that match the user's keywords.

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
        let (story_id, story_title) = Self::find_story(&fetcher, config).await;
        if story_id.is_empty() {
            return Ok(vec![]);
        }

        let item_url = format!("https://hacker-news.firebaseio.com/v0/item/{story_id}.json");
        let item_json = fetcher.fetch(&item_url).await?;
        let item: serde_json::Value = serde_json::from_str(&item_json)?;

        let kids = item["kids"].as_array().cloned().unwrap_or_default();
        let kw_matcher = KeywordMatcher::new(&config.keywords);

        // Fetch all kid comments concurrently — this is way faster than sequential.
        let kid_futures: Vec<_> = kids
            .iter()
            .take(config.max_results)
            .filter_map(|kid| kid.as_i64())
            .map(|comment_id| {
                let fetcher = Fetcher::new().ok();
                async move {
                    let fetcher = fetcher?;
                    let comment_url =
                        format!("https://hacker-news.firebaseio.com/v0/item/{comment_id}.json");
                    let body = fetcher.fetch(&comment_url).await.ok()?;
                    let comment: serde_json::Value = serde_json::from_str(&body).ok()?;
                    Some((comment_id, comment))
                }
            })
            .collect();

        let comments: Vec<_> = futures::future::join_all(kid_futures)
            .await
            .into_iter()
            .flatten()
            .collect();

        let mut posts = Vec::new();

        for (comment_id, comment) in comments {
            let text = comment["text"].as_str().unwrap_or("").to_string();
            let by = comment["by"].as_str().unwrap_or("anonymous");

            if text.is_empty() {
                continue;
            }

            // Only keep comments that mention at least one search keyword
            if !kw_matcher.matches(&text) {
                continue;
            }

            let first_line = text.lines().next().unwrap_or("");
            let company = if first_line.contains('|') {
                first_line.split('|').next().map(|s| s.trim().to_string())
            } else {
                None
            };

            let hn_url = format!("https://news.ycombinator.com/item?id={comment_id}");

            posts.push(JobPost {
                id: Uuid::new_v4().to_string(),
                title: format!("{story_title} -- {by}"),
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

        Ok(posts)
    }
}

impl HackerNewsCrawler {
    /// Find the most recent "Who is Hiring?" thread by searching Algolia,
    /// trying the current month and up to 2 months back.
    async fn find_story(fetcher: &Fetcher, config: &SearchConfig) -> (String, String) {
        let now = Utc::now();
        let keyword_q = if config.keywords.is_empty() {
            String::new()
        } else {
            format!("%20{}", config.keywords.join(" "))
        };

        for offset in 0..=2 {
            let dt = now - chrono::Months::new(offset);
            let month = dt.format("%B").to_string();
            let year = dt.format("%Y").to_string();
            let url = format!(
                "https://hn.algolia.com/api/v1/search?query=Who%20is%20Hiring%20{month}%20{year}{keyword_q}&tags=story&hitsPerPage=5"
            );

            let json = match fetcher.fetch(&url).await {
                Ok(j) => j,
                Err(_) => continue,
            };
            let parsed: serde_json::Value = match serde_json::from_str(&json) {
                Ok(p) => p,
                Err(_) => continue,
            };
            if let Some(hit) = parsed["hits"].as_array().and_then(|h| h.first()) {
                let id = hit["objectID"].as_str().unwrap_or("").to_string();
                if !id.is_empty() {
                    let title = hit["title"].as_str().unwrap_or("Who is Hiring?").to_string();
                    return (id, title);
                }
            }
        }
        (String::new(), String::new())
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
