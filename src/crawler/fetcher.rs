//! # HTTP Fetcher
//!
//! A lightweight, concurrent-safe HTTP client for crawling job sources.
//!
//! ## Design
//!
//! - [`Fetcher`] wraps `reqwest::Client` which internally uses `Arc` —
//!   cloning is cheap and safe across threads.
//! - `fetch()` takes `&self` (not `&mut self`), so a single `Fetcher` can
//!   be shared across concurrent tasks.
//! - No artificial delays. Requests fire immediately with a 15-second timeout.
//! - Single realistic User-Agent (no rotation — modern APIs don't care).

use anyhow::Result;
use reqwest::Client;

/// HTTP client wrapper — just GETs URLs with a sensible timeout.
///
/// `Fetcher` is [`Clone`] (internally `Client` uses `Arc`), and
/// `fetch()` takes `&self` so it can be shared across concurrent tasks.
///
/// # Example
///
/// ```ignore
/// let fetcher = Fetcher::new()?;
/// let body = fetcher.fetch("https://api.example.com/data").await?;
/// ```
#[derive(Clone)]
pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    /// Create a new fetcher with a standard browser User-Agent and a
    /// 15-second request timeout.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
            .build()?;
        Ok(Self { client })
    }

    /// Fetch a URL and return the response body as a string.
    ///
    /// - Timeout: 15 seconds
    /// - No artificial delay
    /// - Returns an error on non-2xx status codes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The request times out (>15s)
    /// - The server returns a non-success status (e.g. 404, 500)
    /// - The response body is not valid UTF-8
    pub async fn fetch(&self, url: &str) -> Result<String> {
        let resp = self.client.get(url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("HTTP {status} for {url}");
        }
        let body = resp.text().await?;
        Ok(body)
    }
}
