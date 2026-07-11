use anyhow::Result;
use chrono::Utc;
use scraper::{Html, Selector};
use uuid::Uuid;

use super::fetcher::Fetcher;
use crate::models::{JobPost, JobSource, SearchConfig};
use super::SourceCrawler;

pub struct GoogleJobsCrawler;

#[async_trait::async_trait]
impl SourceCrawler for GoogleJobsCrawler {
    fn name(&self) -> &str {
        "Google Jobs"
    }

    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let mut fetcher = Fetcher::new()?;
        let query = config.keywords.join(" ");
        let url = format!(
            "https://www.google.com/search?q={query}+jobs&ibp=htl;jobs&sa=X"
        );

        let html = fetcher.fetch(&url).await?;
        let document = Html::parse_document(&html);

        let job_sel =
            Selector::parse("div[jsname='VjXuE'], div.iFjolb, div.gws-jobs__result").unwrap();
        let title_sel = Selector::parse("h3, div[role='heading'], span[jsname]").unwrap();
        let desc_sel = Selector::parse("div[jsname] span, div.gws-jobs__description").unwrap();

        let mut posts = Vec::new();
        for card in document.select(&job_sel).take(config.max_results) {
            let title = card
                .select(&title_sel).next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default()
                .trim().to_string();

            let description = card
                .select(&desc_sel).next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            posts.push(JobPost {
                id: Uuid::new_v4().to_string(),
                title,
                company: None,
                location: None,
                description,
                url: format!("https://www.google.com/search?q={query}+jobs"),
                source: JobSource::GoogleJobs,
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
