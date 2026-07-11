//! # Crawler
//!
//! Web scrapers that fetch job posts from multiple sources. Each source is
//! modelled as a separate struct implementing the [`SourceCrawler`] trait.
//!
//! ## Supported sources
//!
//! | Source         | Struct               | Strategy                                      |
//! |----------------|----------------------|-----------------------------------------------|
//! | Indeed         | [`IndeedCrawler`]    | HTML scraping of `indeed.com` search results  |
//! | Google Jobs    | [`GoogleJobsCrawler`]| HTML scraping of Google job-search results    |
//! | Reddit         | [`RedditCrawler`]    | HTML scraping of `old.reddit.com` search      |
//! | Hacker News    | [`HackerNewsCrawler`]| Firebase API → Algolia search → HN item API   |
//!
//! All crawlers share a common [`Fetcher`] that handles HTTP requests with
//! polite delays, user-agent rotation, and timeout enforcement.

use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use colored::Colorize;
use rand::Rng;
use reqwest::Client;
use scraper::{Html, Selector};
use uuid::Uuid;

use crate::models::{JobPost, JobSource, SearchConfig};

// ─── HTTP Fetcher ─────────────────────────────────────────────────────────────

/// Rotating pool of browser user-agent strings used to disguise requests.
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
];

/// Lightweight HTTP client with polite delays and user-agent rotation.
///
/// Built on [`reqwest::Client`]. Designed to minimise the chance of being
/// rate-limited or blocked by job-board sites.
struct Fetcher {
    client: Client,
}

impl Fetcher {
    /// Create a new [`Fetcher`] with a randomly selected user-agent.
    fn new() -> Result<Self> {
        let idx = rand::thread_rng().gen_range(0..USER_AGENTS.len());
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(USER_AGENTS[idx])
            .build()?;
        Ok(Self { client })
    }

    /// Swap the current user-agent for a different one at random.
    fn rotate_ua(&mut self) {
        let idx = rand::thread_rng().gen_range(0..USER_AGENTS.len());
        if let Ok(c) = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(USER_AGENTS[idx])
            .build()
        {
            self.client = c;
        }
    }

    /// Fetch the HTML body at `url`.
    ///
    /// Adds a random delay (1–3 seconds) before each request, checks for a
    /// successful HTTP status, then rotates the user-agent for the next call.
    async fn fetch(&mut self, url: &str) -> Result<String> {
        // Polite delay to avoid rate-limiting
        let delay_ms = rand::thread_rng().gen_range(1000..3000);
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;

        let resp = self.client.get(url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("HTTP {status} for {url}");
        }
        let body = resp.text().await?;
        self.rotate_ua();
        Ok(body)
    }
}

// ─── Crawler Trait ────────────────────────────────────────────────────────────

/// Common interface for all job-source scrapers.
///
/// Each source implements this trait so that the [`CrawlerCoordinator`] can
/// iterate over them uniformly.
#[async_trait::async_trait]
pub trait SourceCrawler: Send + Sync {
    /// Human-readable name of the source (e.g. "Indeed").
    fn name(&self) -> &str;

    /// Scrape job posts from this source using the given [`SearchConfig`].
    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>>;
}

// ─── Indeed Scraper ───────────────────────────────────────────────────────────

/// Scraper for `indeed.com` job-search result pages.
///
/// Parses the HTML of a Indeed search results page, extracting job cards with
/// title, company, location, description snippet, and a link to the full post.
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
                .select(&title_sel)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default()
                .trim()
                .to_string();

            let company = card
                .select(&company_sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            let location = card
                .select(&loc_sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            let description = card
                .select(&desc_sel)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let url_suffix = card
                .select(&link_sel)
                .next()
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

// ─── Google Jobs Scraper ──────────────────────────────────────────────────────

/// Scraper for Google's job-search integration.
///
/// Parses the HTML of a Google search results page that includes the job
/// listing widget (`ibp=htl;jobs` parameter).
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
                .select(&title_sel)
                .next()
                .map(|e| e.text().collect::<String>())
                .unwrap_or_default()
                .trim()
                .to_string();

            let description = card
                .select(&desc_sel)
                .next()
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

// ─── Reddit Scraper ───────────────────────────────────────────────────────────

/// Scraper for Reddit "For Hire" and job-related subreddits.
///
/// Searches subreddits like `r/forhire`, `r/jobbit`, `r/remotejobs`,
/// `r/jobs`, and `r/cscareerquestions` on `old.reddit.com` for recent posts
/// that look like hiring opportunities.
pub struct RedditCrawler;

#[async_trait::async_trait]
impl SourceCrawler for RedditCrawler {
    fn name(&self) -> &str {
        "Reddit"
    }

    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let mut fetcher = Fetcher::new()?;
        let query = urlencode(config.keywords.join(" ").as_str());

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
                    .select(&title_sel)
                    .next()
                    .map(|e| e.text().collect::<String>())
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                let url_suffix = post
                    .select(&link_sel)
                    .next()
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

                // Only keep posts that look like they're about hiring
                let lower = title.to_lowercase();
                let is_hiring_post = lower.contains("hiring")
                    || lower.contains("[hiring]")
                    || lower.contains("job")
                    || lower.contains("position")
                    || lower.contains("opening")
                    || lower.contains("opportunity")
                    || lower.contains("looking for")
                    || lower.contains("need a")
                    || lower.contains("we are")
                    || lower.contains("full-time")
                    || lower.contains("part-time")
                    || lower.contains("contract")
                    || lower.contains("remote");

                if !is_hiring_post {
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

// ─── Hacker News "Who is Hiring" Crawler ──────────────────────────────────────

/// Scraper for the monthly Hacker News "Who is Hiring?" thread.
///
/// Uses the Algolia API to find the current month's thread, then fetches
/// individual comments via the Firebase API and converts them into job posts.
pub struct HackerNewsCrawler;

#[async_trait::async_trait]
impl SourceCrawler for HackerNewsCrawler {
    fn name(&self) -> &str {
        "Hacker News"
    }

    async fn crawl(&self, config: &SearchConfig) -> Result<Vec<JobPost>> {
        let mut fetcher = Fetcher::new()?;
        let _query = config.keywords.join(" ");

        let now = Utc::now();
        let month = now.format("%B").to_string();
        let year = now.format("%Y").to_string();
        let search_url = format!(
            "https://hn.algolia.com/api/v1/search?query=Who%20is%20Hiring%20{month}%20{year}&tags=story&hitsPerPage=5"
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

                // Fetch comments (the actual job posts inside the thread)
                let kids = item["kids"].as_array().cloned().unwrap_or_default();
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

                    // Try to extract company from the first line (common format: "Company | Role | Location")
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
                        title: format!("{title} — {by}"),
                        company,
                        location: None,
                        description: text,
                        url: hn_url,
                        source: JobSource::HackerNews,
                        posted_at: None,
                        crawled_at: Utc::now(),
                        salary: None,
                        job_type: None,
                        tags: vec![
                            "hacker-news".to_string(),
                            "who-is-hiring".to_string(),
                        ],
                    });
                }
            }
        }

        Ok(posts)
    }
}

// ─── Crawler Coordinator ──────────────────────────────────────────────────────

/// Orchestrates multiple [`SourceCrawler`] implementations and aggregates
/// their results.
///
/// The coordinator runs each enabled crawler sequentially, collecting all
/// job posts into a single flattened list. Errors from individual crawlers
/// are logged to stderr but do not abort the crawl.
pub struct CrawlerCoordinator {
    crawlers: Vec<Box<dyn SourceCrawler>>,
}

impl CrawlerCoordinator {
    /// Create a new coordinator with the default set of crawlers (Indeed,
    /// Google Jobs, Reddit, Hacker News).
    pub fn new() -> Self {
        Self {
            crawlers: vec![
                Box::new(IndeedCrawler),
                Box::new(GoogleJobsCrawler),
                Box::new(RedditCrawler),
                Box::new(HackerNewsCrawler),
            ],
        }
    }

    /// Run all enabled crawlers and aggregate their results.
    ///
    /// Only crawlers whose names match an entry in `config.sources` are
    /// executed. Results from each crawler are printed to stderr as they
    /// arrive. Failed crawlers are logged but do not stop the remaining ones.
    pub async fn crawl_all(&self, config: &SearchConfig) -> Vec<JobPost> {
        let mut all_posts = Vec::new();

        for crawler in &self.crawlers {
            let name = crawler.name();
            if !config
                .sources
                .iter()
                .any(|s| name.to_lowercase().contains(&s.to_string().to_lowercase()))
            {
                continue;
            }

            match crawler.crawl(config).await {
                Ok(posts) => {
                    eprintln!(
                        "  {} {} posts from {}",
                        "✓".green(),
                        posts.len(),
                        name.cyan()
                    );
                    all_posts.extend(posts);
                }
                Err(e) => {
                    eprintln!("  {} {} — {e}", "✗".red(), name.cyan());
                }
            }
        }

        all_posts
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Simple URL-encoding: replaces spaces with `+`.
///
/// This is sufficient for basic query parameters. Full percent-encoding is
/// not required because Reddit's search endpoint handles `+` natively.
fn urlencode(s: &str) -> String {
    s.replace(' ', "+")
}
