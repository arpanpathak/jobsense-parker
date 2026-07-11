//! Crawler for [Remote OK](https://remoteok.com) — a JSON API returning
//! remote job listings by tag/keyword.

use anyhow::Result;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::fetcher::Fetcher;
use crate::models::{JobPost, JobSource, SearchConfig};
use super::SourceCrawler;

/// Crawler that fetches jobs from Remote OK's JSON endpoint.
pub struct RemoteOkCrawler;

#[async_trait::async_trait]
impl SourceCrawler for RemoteOkCrawler {
    fn name(&self) -> &str {
        "Remote OK"
    }

    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let mut fetcher = Fetcher::new()?;

        // Remote OK has tag-based pages. Try each keyword to get relevant results.
        // First element in each response is metadata, rest are job posts.
        let keywords = if config.keywords.is_empty() {
            vec!["software".to_string()]
        } else {
            config.keywords.clone()
        };

        let mut seen = std::collections::HashSet::new();
        let mut posts = Vec::new();

        for kw in &keywords {
            let kw_lower = kw.to_lowercase().trim().to_string();
            if kw_lower.is_empty() {
                continue;
            }

            let url = format!(
                "https://remoteok.com/remote-{}-jobs.json",
                urlencode(&kw_lower)
            );

            let body = match fetcher.fetch(&url).await {
                Ok(b) => b,
                Err(_) => continue,
            };

            let parsed: Vec<serde_json::Value> = match serde_json::from_str(&body) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // First element is metadata (has "last_updated"), skip it
            for item in parsed.iter().skip(1) {
                let slug = item["slug"].as_str().unwrap_or("");
                if slug.is_empty() || seen.contains(slug) {
                    continue;
                }
                seen.insert(slug.to_string());

                let title = item["position"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if title.is_empty() {
                    continue;
                }

                let company = item["company"].as_str().map(|s| s.trim().to_string());

                let location = item["location"]
                    .as_str()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());

                let raw_desc = item["description"].as_str().unwrap_or("");
                // Strip HTML tags for a plain-text description
                let description = strip_html(raw_desc);

                let url = item["url"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();

                let salary_min = item["salary_min"].as_i64().unwrap_or(0);
                let salary_max = item["salary_max"].as_i64().unwrap_or(0);
                let salary = if salary_min > 0 || salary_max > 0 {
                    Some(format_salary(salary_min, salary_max))
                } else {
                    None
                };

                let posted_at = item["epoch"]
                    .as_i64()
                    .and_then(|epoch| {
                        DateTime::from_timestamp(epoch, 0)
                    });

                let tags: Vec<String> = item["tags"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                posts.push(JobPost {
                    id: Uuid::new_v4().to_string(),
                    title,
                    company,
                    location,
                    description,
                    url,
                    source: JobSource::RemoteOk,
                    posted_at,
                    crawled_at: Utc::now(),
                    salary,
                    job_type: None,
                    tags,
                });

                if posts.len() >= config.max_results {
                    break;
                }
            }

            if posts.len() >= config.max_results {
                break;
            }
        }

        // De-duplicate by keeping unique jobs sorted by date (newest first)
        posts.sort_by(|a, b| b.crawled_at.cmp(&a.crawled_at));
        posts.truncate(config.max_results);

        Ok(posts)
    }
}

/// URL-encode a string for Remote OK's API (spaces → dashes, lowercase).
fn urlencode(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => result.push(ch),
            ' ' => result.push('-'),
            _ => {
                for byte in ch.to_string().bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result.to_lowercase()
}

/// Remove HTML tags and decode common entities from a string.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_entity = false;
    let mut entity_buf = String::new();

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            '&' if !in_tag => {
                in_entity = true;
                entity_buf.clear();
            }
            ';' if in_entity => {
                in_entity = false;
                let decoded = match entity_buf.as_str() {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "#39" => "'",
                    "nbsp" => " ",
                    _ => "",
                };
                result.push_str(decoded);
            }
            _ if in_entity => entity_buf.push(ch),
            _ if !in_tag && !in_entity => {
                result.push(ch);
            }
            _ => {}
        }
    }

    // Collapse multiple whitespace into single space
    let mut cleaned = String::with_capacity(result.len());
    let mut prev_space = false;
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                cleaned.push(' ');
                prev_space = true;
            }
        } else {
            cleaned.push(ch);
            prev_space = false;
        }
    }

    cleaned.trim().to_string()
}

/// Format a salary range for display.
fn format_salary(min: i64, max: i64) -> String {
    match (min, max) {
        (0, 0) => String::new(),
        (0, m) => format!("Up to ${}/year", m),
        (n, 0) => format!("From ${}/year", n),
        (n, m) if n == m => format!("${}/year", n),
        (n, m) => format!("${} - ${}/year", n, m),
    }
}
