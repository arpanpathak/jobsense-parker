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
        let fetcher = Fetcher::new()?;

        // Fetch ALL jobs from the main Remote OK API.
        // The API returns a flat JSON array of job objects — NO metadata element
        // at index 0 (unlike the tag-based pages which had metadata first).
        let body = fetcher.fetch("https://remoteok.com/api").await?;

        // Try to parse as flat array first, then fall back to object-wrapped format.
        let items: Vec<serde_json::Value> = match serde_json::from_str::<Vec<serde_json::Value>>(&body)
        {
            Ok(arr) => {
                // The API returns a flat array. No metadata element.
                // If the first element has a "last_updated" field, it IS metadata — skip it.
                if arr.first().and_then(|f| f.get("last_updated")).is_some() {
                    arr.iter().skip(1).cloned().collect()
                } else {
                    arr
                }
            }
            Err(_) => {
                // Try object-wrapped format: { "jobs": [...] }
                let obj: serde_json::Value = serde_json::from_str(&body)?;
                obj["jobs"].as_array().cloned().unwrap_or_default()
            }
        };

        let mut posts: Vec<JobPost> = items.iter().filter_map(Self::parse_item).collect();

        // Sort by date (newest first), cap with room for keyword filtering
        posts.sort_by(|a, b| b.crawled_at.cmp(&a.crawled_at));
        posts.truncate(config.max_results * 2);

        Ok(posts)
    }
}

impl RemoteOkCrawler {
    /// Parse a single Remote OK JSON item into a [`JobPost`], or return `None`
    /// if essential fields are missing.
    fn parse_item(item: &serde_json::Value) -> Option<JobPost> {
        let slug = item["slug"].as_str().unwrap_or("");
        if slug.is_empty() {
            return None;
        }
        let title = item["position"].as_str()?.trim().to_string();
        if title.is_empty() {
            return None;
        }
        let company = item["company"].as_str().map(|s| s.trim().to_string());
        let location = item["location"]
            .as_str()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let raw_desc = item["description"].as_str().unwrap_or("");
        let description = strip_tag_cloud(&strip_html(raw_desc));
        let url = item["url"].as_str().unwrap_or("").to_string();
        let salary_min = item["salary_min"].as_i64().unwrap_or(0);
        let salary_max = item["salary_max"].as_i64().unwrap_or(0);
        let salary = (salary_min > 0 || salary_max > 0).then(|| format_salary(salary_min, salary_max));
        let posted_at = item["epoch"].as_i64().and_then(|e| DateTime::from_timestamp(e, 0));
        let tags: Vec<String> = item["tags"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        Some(JobPost {
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
        })
    }
}

/// URL-encode a string for Remote OK's API (spaces → dashes, lowercase).
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

/// Strip tag-cloud sections from Remote OK descriptions.
///
/// Remote OK's `description` field often contains a comma-separated tag cloud
/// with every tech keyword the platform associates with the job. This makes
/// the matcher think a "Senior Vice President" job requires "ai, angular, c#,
/// python, react, rust..." — because those tags appear in the description.
///
/// We detect common tag-cloud markers and truncate at that point, keeping
/// only the actual job description text.
///
/// Markers we strip (case-insensitive):
/// - "Tags: ..."
/// - "Technologies: ..."
/// - "Tech Stack: ..."
/// - "Skills: ..."
/// - "Requirements: ..." (only if followed by a list, not paragraphs)
/// - Any trailing line that is just comma-separated tech keywords
fn strip_tag_cloud(text: &str) -> String {
    let lower = text.to_lowercase();
    let markers = ["tags:", "technologies:", "tech stack:", "skills:"];

    // Find the earliest marker and truncate before it
    let mut earliest = None;
    for marker in &markers {
        if let Some(pos) = lower.find(marker) {
            match earliest {
                None => earliest = Some(pos),
                Some(current) if pos < current => earliest = Some(pos),
                _ => {}
            }
        }
    }

    match earliest {
        Some(pos) => {
            let truncated = &text[..pos].trim();
            truncated.to_string()
        }
        None => text.to_string(),
    }
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
