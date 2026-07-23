//! Career-site crawler — fetches each known company's career page and extracts
//! job listings using either:
//!
//! 1. **Dedicated API** (Greenhouse, Lever) — clean JSON, always works
//! 2. **Heuristic HTML scraping** — falls back for all other sites
//!
//! ## Known Limitations
//!
//! Most modern career sites (Workday, MyWorkDay, SuccessFactors, etc.) use
//! JavaScript rendering (SPA). The `scraper` crate can only parse static HTML,
//! so these sites will be detected and reported as "JS-rendered" rather than
//! returning empty results silently.

use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use futures::pin_mut;
use futures::stream::StreamExt;
use scraper::{Html, Selector};
use uuid::Uuid;

use crate::models::{Company, CompanyDatabase, JobPost, JobSource, SearchConfig};

use super::fetcher::Fetcher;

/// Crawls the career pages of all known companies in the database.
pub struct CompanyCrawler;

impl CompanyCrawler {
    /// Crawl every company in `db` concurrently (10 at a time).
    pub async fn crawl_all(db: &mut CompanyDatabase, config: &SearchConfig) -> Vec<JobPost> {
        let companies: Vec<Company> = db.companies.clone();

        if companies.is_empty() {
            return Vec::new();
        }

        let stream = futures::stream::iter(companies.into_iter().map(|company| {
            let config = config.clone();
            async move {
                let result = Self::crawl_company(&company, &config).await;
                (company, result)
            }
        }))
        .buffer_unordered(10);

        let mut all_posts = Vec::new();
        let mut ok_count = 0usize;
        let mut err_count = 0usize;

        pin_mut!(stream);
        while let Some((company, result)) = stream.next().await {
            match result {
                Ok(posts) => {
                    ok_count += 1;
                    db.mark_crawled(&company.name);
                    all_posts.extend(posts);
                }
                Err(e) => {
                    err_count += 1;
                    db.mark_failed(&company.name, &e.to_string());
                }
            }
        }

        let total = ok_count + err_count;
        if all_posts.is_empty() && err_count > 0 {
            eprintln!(
                "  {} {} company pages fetched ({} ok, {} failed) — 0 jobs found",
                "-".yellow(),
                total, ok_count, err_count
            );
        } else if all_posts.is_empty() {
            eprintln!(
                "  {} {} company pages fetched — 0 jobs (most use JS rendering)",
                "-".yellow(),
                total
            );
        } else {
            eprintln!(
                "  {} {} jobs from {} company pages ({} ok, {} failed)",
                "+".green(),
                all_posts.len(),
                total, ok_count, err_count,
            );
        }

        all_posts
    }

    // ─── Greenhouse API ─────────────────────────────────────────────────

    fn extract_greenhouse_slug(url: &str) -> Option<String> {
        let marker = "boards.greenhouse.io/";
        if let Some(pos) = url.find(marker) {
            let after = &url[pos + marker.len()..];
            let slug = after.split('/').next().unwrap_or("");
            if !slug.is_empty() {
                return Some(slug.to_string());
            }
        }
        let host_start = url.find("://").map(|p| p + 3).unwrap_or(0);
        let domain = &url[host_start..];
        if let Some(dot) = domain.find('.') {
            let subdomain = &domain[..dot];
            if !subdomain.is_empty() && subdomain != "boards" && !subdomain.contains('/') {
                return Some(subdomain.to_string());
            }
        }
        None
    }

    async fn crawl_greenhouse(
        fetcher: &Fetcher,
        slug: &str,
        company: &Company,
        config: &SearchConfig,
    ) -> Result<Vec<JobPost>> {
        let url = format!("https://boards-api.greenhouse.io/v1/boards/{slug}/jobs?content=true");
        let body = fetcher.fetch(&url).await?;
        let parsed: serde_json::Value = serde_json::from_str(&body)?;

        let jobs = parsed["jobs"].as_array().ok_or_else(|| anyhow::anyhow!("no jobs array"))?;
        let mut posts = Vec::new();

        for job in jobs.iter() {
            let title = job["title"].as_str().unwrap_or("").to_string();
            if title.is_empty() { continue; }

            if !config.keywords.is_empty() {
                let lower = title.to_lowercase();
                if !config.keywords.iter().any(|kw| lower.contains(&kw.to_lowercase())) {
                    continue;
                }
            }

            let url = job["absolute_url"].as_str().unwrap_or("").to_string();
            let location = job["offices"].as_array()
                .and_then(|o| o.first())
                .and_then(|o| o["location"].as_str().map(|s| s.to_string()));
            let updated_at = job["updated_at"].as_str()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            posts.push(JobPost {
                id: Uuid::new_v4().to_string(),
                title,
                company: Some(company.name.clone()),
                location,
                description: job["content"].as_str().unwrap_or("").to_string(),
                url,
                source: JobSource::Custom(company.name.clone()),
                posted_at: updated_at,
                crawled_at: Utc::now(),
                salary: None,
                job_type: None,
                tags: vec!["greenhouse".to_string(), company.name.to_lowercase()],
            });
        }

        Ok(posts)
    }

    // ─── Lever API ──────────────────────────────────────────────────────

    fn extract_lever_slug(url: &str) -> Option<String> {
        if let Some(pos) = url.find("jobs.lever.co/") {
            let rest = &url[pos + "jobs.lever.co/".len()..];
            return rest.split('/').next().filter(|s| !s.is_empty()).map(|s| s.to_string());
        }
        None
    }

    async fn crawl_lever(
        fetcher: &Fetcher,
        slug: &str,
        company: &Company,
        config: &SearchConfig,
    ) -> Result<Vec<JobPost>> {
        let url = format!("https://api.lever.co/v0/postings/{slug}?mode=json");
        let body = fetcher.fetch(&url).await?;
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&body)?;

        let mut posts = Vec::new();
        for job in parsed.iter() {
            let title = job["text"].as_str().unwrap_or("").to_string();
            if title.is_empty() { continue; }

            if !config.keywords.is_empty() {
                let lower = title.to_lowercase();
                if !config.keywords.iter().any(|kw| lower.contains(&kw.to_lowercase())) {
                    continue;
                }
            }

            let url = job["hostedUrl"].as_str().unwrap_or("").to_string();
            let location = job["categories"]["location"].as_str().map(|s| s.to_string());
            let description = job["descriptionPlain"].as_str().unwrap_or("").to_string();
            let updated_at = job["createdAt"].as_i64()
                .and_then(|ts| DateTime::from_timestamp(ts / 1000, 0));

            posts.push(JobPost {
                id: Uuid::new_v4().to_string(),
                title,
                company: Some(company.name.clone()),
                location,
                description,
                url,
                source: JobSource::Custom(company.name.clone()),
                posted_at: updated_at,
                crawled_at: Utc::now(),
                salary: None,
                job_type: None,
                tags: vec!["lever".to_string(), company.name.to_lowercase()],
            });
        }

        Ok(posts)
    }

    // ─── Generic Crawler ────────────────────────────────────────────────

    /// Fetch a single company's careers page and extract job listings.
    async fn crawl_company(company: &Company, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let fetcher = Fetcher::new()?;

        // ── 1. Dedicated API support (Greenhouse, Lever) ──────────────
        let url_lower = company.careers_url.to_lowercase();

        if let Some(slug) = Self::extract_greenhouse_slug(&url_lower) {
            if let Ok(posts) = Self::crawl_greenhouse(&fetcher, &slug, company, config).await {
                return Ok(posts);
            }
        }

        if let Some(slug) = Self::extract_lever_slug(&url_lower) {
            if let Ok(posts) = Self::crawl_lever(&fetcher, &slug, company, config).await {
                return Ok(posts);
            }
        }

        // ── 2. HTML fallback ──────────────────────────────────────────
        let html = fetcher.fetch(&company.careers_url).await?;

        // Detect JS-rendered SPAs early — most career sites use React/Angular
        let body_start = html.find("<body").unwrap_or(0);
        let body_end = html.rfind("</body>").unwrap_or(html.len());
        let body_content = &html[body_start..body_end];
        let text_chars: usize = body_content.chars().filter(|c| c.is_ascii_alphabetic()).count();

        if text_chars < 200 {
            return Err(anyhow::anyhow!(
                "JS-rendered page ({} readable chars). Site uses SPA framework.",
                text_chars
            ));
        }

        let document = Html::parse_document(&html);
        let mut posts = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();
        let role_words = [
            "engineer", "developer", "manager", "analyst", "scientist",
            "designer", "architect", "intern", "specialist", "director",
            "lead", "senior", "software", "product",
        ];

        if let Ok(link_sel) = Selector::parse("a[href]") {
            for element in document.select(&link_sel) {
                let href = element.value().attr("href").unwrap_or("").trim().to_string();
                if href.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
                    continue;
                }

                let text: String = element.text().collect::<Vec<_>>().join(" ")
                    .split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string();

                if text.is_empty() || text.len() < 5 {
                    continue;
                }

                let lower_href = href.to_lowercase();
                let lower_text = text.to_lowercase();

                let skip = ["login", "register", "faq", "privacy", "terms", "blog", "press"];
                if skip.iter().any(|s| lower_href.contains(s)) {
                    continue;
                }

                let job_url = lower_href.contains("/job/") || lower_href.contains("/jobs/")
                    || lower_href.contains("/careers/") || lower_href.contains("/career/")
                    || lower_href.contains("/position/") || lower_href.contains("/positions/")
                    || lower_href.contains("/opening/") || lower_href.contains("/openings/")
                    || lower_href.contains("requisition") || lower_href.contains("gh_jid")
                    || lower_href.contains("lever.co/") || lower_href.contains("jobdetails");

                let role_text = role_words.iter().any(|w| lower_text.contains(w));

                if !job_url && !role_text {
                    continue;
                }

                if text.len() > 120 {
                    continue;
                }

                let full_url = if href.starts_with("http") {
                    href.clone()
                } else {
                    format!("{}/{}", company.careers_url.trim_end_matches('/'), href.trim_start_matches('/'))
                };

                if !seen_urls.insert(full_url.clone()) {
                    continue;
                }

                posts.push(JobPost {
                    id: Uuid::new_v4().to_string(),
                    title: text.trim().trim_matches(|c: char| c == '"' || c == '\'').to_string(),
                    company: Some(company.name.clone()),
                    location: None,
                    description: String::new(),
                    url: full_url,
                    source: JobSource::Custom(company.name.clone()),
                    posted_at: None,
                    crawled_at: Utc::now(),
                    salary: None,
                    job_type: None,
                    tags: vec!["careers-site".to_string(), company.name.to_lowercase()],
                });

                if posts.len() >= config.max_results {
                    break;
                }
            }
        }

        if posts.is_empty() {
            Err(anyhow::anyhow!(
                "No job listings found. Site may use JS rendering or non-standard HTML."
            ))
        } else {
            Ok(posts)
        }
    }
}
