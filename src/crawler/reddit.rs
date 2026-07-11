use anyhow::Result;
use chrono::Utc;
use scraper::{Html, Selector};
use uuid::Uuid;

use super::fetcher::Fetcher;
use crate::models::{JobPost, JobSource, SearchConfig};
use super::SourceCrawler;

pub struct RedditCrawler;

#[async_trait::async_trait]
impl SourceCrawler for RedditCrawler {
    fn name(&self) -> &str {
        "Reddit"
    }

    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let mut fetcher = Fetcher::new()?;
        let query = urlencode(&config.keywords.join(" "));

        let subreddits = ["forhire", "jobbit", "remotejobs", "jobs", "cscareerquestions"];
        let mut posts = Vec::new();

        for sub in subreddits {
            let url = format!(
                "https://old.reddit.com/r/{sub}/search?q={query}&sort=new&restrict_sr=on&t=week"
            );

            let html = match fetcher.fetch(&url).await {
                Ok(h) => h,
                Err(_) => continue,
            };

            let document = Html::parse_document(&html);
            let post_sel = Selector::parse("div.thing").unwrap();
            let title_sel = Selector::parse("a.title").unwrap();
            let link_sel = Selector::parse("a.title").unwrap();

            for post in document.select(&post_sel).take(config.max_results / subreddits.len()) {
                let title = post
                    .select(&title_sel).next()
                    .map(|e| e.text().collect::<String>())
                    .unwrap_or_default()
                    .trim().to_string();

                let url_suffix = post
                    .select(&link_sel).next()
                    .and_then(|e| e.value().attr("href"))
                    .unwrap_or("");
                let url = if url_suffix.starts_with("http") {
                    url_suffix.to_string()
                } else {
                    format!("https://old.reddit.com{url_suffix}")
                };

                if title.is_empty() {
                    continue;
                }

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

                posts.push(JobPost {
                    id: Uuid::new_v4().to_string(),
                    title,
                    company: None,
                    location: None,
                    description: String::new(),
                    url,
                    source: JobSource::Reddit,
                    posted_at: None,
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

fn urlencode(s: &str) -> String {
    s.replace(' ', "+")
}
