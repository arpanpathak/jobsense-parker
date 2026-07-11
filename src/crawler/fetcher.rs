//! Simple HTTP fetcher — fast, no artificial delays, safe to share across threads.

use anyhow::Result;
use reqwest::Client;

/// HTTP client wrapper — just GETs URLs. No sleeping, no UA rotation.
///
/// `Fetcher` is cheap to clone (internally `Client` uses `Arc`), and
/// `fetch()` takes `&self` so it can be used from concurrent contexts.
#[derive(Clone)]
pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    /// Create a new fetcher with a standard browser User-Agent.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
            .build()?;
        Ok(Self { client })
    }

    /// Fetch a URL and return the response body as a string.
    ///
    /// No artificial delay. Timeout is 15 seconds.
    /// Takes `&self` so multiple URLs can be fetched concurrently.
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
