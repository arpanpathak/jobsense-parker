use anyhow::Result;
use chrono::Utc;
use scraper::{Html, Selector};
use uuid::Uuid;

use super::fetcher::Fetcher;
use crate::models::{JobPost, JobSource, SearchConfig};
use super::SourceCrawler;

pub struct IndeedCrawler;

#[async_trait::async_trait]
impl SourceCrawler for IndeedCrawler {
    fn name(&self) -> &str {
        "Indeed"
    }

    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let mut fetcher = Fetcher::new()?;
        let query = config.keywords.join("+");
        let loc = config.location.as_deref().unwrap_or("remote");
        let url = format!(
            "https://www.indeed.com/jobs?q={query}&l={loc}&fromage=7&sort=date"
        );

        let html = fetcher.fetch(&url).await?;
        let document = Html::parse_document(&html);

        let card_sel = Selector::parse("div.job_seen_beacon").unwrap();
        let title_sel = Selector::parse("h2.jobTitle a, a.jobTitle").unwrap();
        let company_sel = Selector::parse("span.companyName").unwrap();
        let loc_sel = Selector::parse("div.companyLocation").unwrap();
        let desc_sel = Selector::parse("div.job-snippet").unwrap();
        let link_sel = Selector::parse("h2.jobTitle a").unwrap();

        let mut posts = Vec::new();
        for card in document.select(&card_sel).take(config.max_results) {
            let title = card
                .select(&title_sel).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default()
                .trim().to_string();

            let company = card
                .select(&company_sel).next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            let location = card
                .select(&loc_sel).next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            let description = card
                .select(&desc_sel).next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let url_suffix = card
                .select(&link_sel).next()
                .and_then(|e| e.value().attr("href"))
                .unwrap_or("");
            let url = if url_suffix.starts_with("http") {
                url_suffix.to_string()
            } else {
                format!("https://www.indeed.com{url_suffix}")
            };

            if title.is_empty() {
                continue;
            }

            posts.push(JobPost {
                id: Uuid::new_v4().to_string(),
                title,
                company,
                location,
                description,
                url,
                source: JobSource::Indeed,
                posted_at: None,
                crawled_at: Utc::now(),
                salary: None,
                job_type: None,
                tags: vec![],
            });
        }

        Ok(posts)
    }
}
