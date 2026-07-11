//! Career-site crawler — fetches each known company's careers page and
//! extracts job listings using heuristic HTML parsing.
//!
//! Since every company has a different career-page layout, we use a
//! pattern-based approach:
//!
//! 1. Fetch the careers URL
//! 2. Find every `<a>` link on the page
//! 3. Filter links whose href or text suggests a job posting
//! 4. Extract title, optional location, build a [`JobPost`]
//!
//! This will never be perfect for every site, but it covers a wide range
//! of company career pages without per-site configuration.

use anyhow::Result;
use chrono::Utc;
use colored::Colorize;
use scraper::{Html, Selector};
use uuid::Uuid;

use crate::models::{Company, CompanyDatabase, JobPost, JobSource, SearchConfig};

use super::fetcher::Fetcher;

/// Crawls the career pages of all known companies in the database.
pub struct CompanyCrawler;

impl CompanyCrawler {
    /// Crawl every company in `db` **concurrently** whose careers page
    /// hasn't been crawled recently (or at all).
    ///
    /// With 80+ companies, sequential crawling would take minutes.
    /// Concurrently, most finish in the time of the slowest single page.
    ///
    /// Returns discovered job posts and updates `db.last_crawled` /
    /// `db.failed` on each company.
    pub async fn crawl_all(db: &mut CompanyDatabase, config: &SearchConfig) -> Vec<JobPost> {
        // Crawl ALL companies every time. No hour-skip — users expect career
        // sites to be searched on every scan to find new postings.
        let companies: Vec<Company> = db.companies.clone();

        if companies.is_empty() {
            return Vec::new();
        }

        // Fetch all company career pages concurrently
        let futures: Vec<_> = companies
            .iter()
            .map(|company| {
                let company = company.clone();
                let config = config.clone();
                async move {
                    let result = Self::crawl_company(&company, &config).await;
                    (company, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;
        let mut all_posts = Vec::new();
        let mut ok_count = 0usize;
        let mut err_count = 0usize;

        for (company, result) in results {
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

        // One summary line instead of 80+ lines of noise
        let total = ok_count + err_count;
        if all_posts.is_empty() && err_count > 0 {
            eprintln!(
                "  {} {} company pages fetched ({} ok, {} failed) — 0 job listings found",
                "-".yellow(),
                total,
                ok_count,
                err_count
            );
        } else if all_posts.is_empty() {
            eprintln!(
                "  {} {} company pages fetched — 0 job listings (most use JS rendering)",
                "-".yellow(),
                total
            );
        } else {
            eprintln!(
                "  {} {} jobs from {} company pages ({} ok, {} failed)",
                "+".green(),
                all_posts.len(),
                total,
                ok_count,
                err_count,
            );
        }

        all_posts
    }

    /// Fetch a single company's careers page and extract job listings.
    async fn crawl_company(company: &Company, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let fetcher = Fetcher::new()?;
        let html = fetcher.fetch(&company.careers_url).await?;
        let document = Html::parse_document(&html);

        // Find all links on the page
        let link_sel = Selector::parse("a[href]").unwrap();
        let mut posts = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();

        for element in document.select(&link_sel) {
            let href = element
                .value()
                .attr("href")
                .unwrap_or("")
                .trim()
                .to_string();

            if href.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
                continue;
            }

            let text: String = element
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();

            if text.is_empty() {
                continue;
            }

            // Check if this link looks like a job posting
            if !Self::is_job_link(&href, &text) {
                continue;
            }

            // Resolve relative URLs
            let full_url = if href.starts_with("http") {
                href.clone()
            } else {
                let base = company.careers_url.trim_end_matches('/');
                let path = href.trim_start_matches('/');
                format!("{base}/{path}")
            };

            // Deduplicate
            if !seen_urls.insert(full_url.clone()) {
                continue;
            }

            // Extract a simple location heuristic from text
            let location = Self::extract_location(&text, &html);

            // Clean up the title
            let title = text
                .trim()
                .trim_matches(|c: char| c == '"' || c == '\'' || c == ' ')
                .to_string();

            if title.len() < 3 {
                continue;
            }

            // Apply keyword filter if the user specified search keywords
            if !config.keywords.is_empty() {
                let lower = format!("{} {}", title, company.name).to_lowercase();
                let matches = config
                    .keywords
                    .iter()
                    .any(|kw| lower.contains(&kw.to_lowercase()));
                if !matches {
                    continue;
                }
            }

            posts.push(JobPost {
                id: Uuid::new_v4().to_string(),
                title,
                company: Some(company.name.clone()),
                location,
                description: String::new(), // We'd need a second fetch for detail pages
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

        Ok(posts)
    }

    /// Heuristic: does this link look like a job posting?
    fn is_job_link(href: &str, text: &str) -> bool {
        let lower_href = href.to_lowercase();
        let lower_text = text.to_lowercase();

        // Keywords in the URL that indicate a job listing
        let job_url_patterns = [
            "/job/",
            "/jobs/",
            "/career/",
            "/careers/",
            "/position/",
            "/positions/",
            "/opening/",
            "/openings/",
            "/apply/",
            "/vacancy/",
            "/vacancies/",
            "/requisition/",
            "/opportunity/",
            "/opportunities/",
            "job=",
            "jobid=",
            "job-id=",
            "job_id=",
            "req_id=",
            "position=",
            "gh_jid",      // Greenhouse
            "lever.co/",   // Lever
            "workday.com", // Workday
            "jobdetails",  // Common job detail page
        ];

        // Keywords in the link text that suggest a job title
        let job_text_keywords = [
            "engineer",
            "developer",
            "manager",
            "analyst",
            "scientist",
            "designer",
            "architect",
            "intern",
            "internship",
            "consultant",
            "specialist",
            "coordinator",
            "associate",
            "director",
            "lead",
            "senior",
            "junior",
            "staff",
            "principal",
            "software",
            "data",
            "product",
            "program",
            "project",
            "technical",
            "solution",
            "support",
            "devops",
            "platform",
            "infrastructure",
            "security",
            "site reliability",
            "full-stack",
            "frontend",
            "backend",
            "full stack",
            "front end",
            "back end",
            "machine learning",
            "ai",
            "ml",
            "qa",
            "test",
            "quality",
            "operations",
            "hr",
            "recruiter",
            "people",
            "marketing",
            "finance",
            "legal",
            "sales",
            "success",
            "customer",
            "application",
        ];

        // Exclude non-job links
        let exclude_patterns = [
            "login",
            "signin",
            "register",
            "faq",
            "help",
            "support",
            "contact",
            "about",
            "privacy",
            "terms",
            "cookie",
            "press",
            "news",
            "blog",
            "events",
            "location",
            "department",
            "team",
            "culture",
            "benefits",
            "life",
        ];

        // A link is a job if its URL contains a job-related path
        let url_match = job_url_patterns.iter().any(|p| lower_href.contains(p));
        if url_match {
            // But exclude clearly non-job pages
            let excluded = exclude_patterns.iter().any(|p| lower_href.contains(p));
            return !excluded;
        }

        // Also check if the link text contains a job title keyword
        // (catches career pages that link directly with descriptive text)
        let text_match = job_text_keywords.iter().any(|k| lower_text.contains(k));
        if text_match {
            // Skip if href is clearly non-job
            let excluded = exclude_patterns.iter().any(|p| lower_href.contains(p));
            return !excluded && text.len() < 120; // reasonable title length
        }

        false
    }

    /// Crude location extraction from text (look for common patterns).
    fn extract_location(text: &str, _html: &str) -> Option<String> {
        let lower = text.to_lowercase();
        let patterns = [
            (r"(?i)(?:in|at|near|location:?)\s*([a-z][a-z\s.-]{2,30}?)(?:[,.]|$)", "remote"),
        ];

        for (pat, _) in &patterns {
            if let Ok(re) = regex::Regex::new(pat) {
                if let Some(cap) = re.captures(&lower) {
                    let loc = cap.get(1).map(|m| m.as_str().trim().to_string())?;
                    if loc.len() > 2 && loc.len() < 40 {
                        return Some(loc);
                    }
                }
            }
        }

        // Check if "remote" appears in the text
        if lower.contains("remote") {
            return Some("Remote".to_string());
        }

        None
    }
}
