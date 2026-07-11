//! Simple HTTP fetcher with random User-Agent rotation and polite delays.

use std::time::Duration;
use anyhow::Result;
use rand::Rng;
use reqwest::Client;

/// Pool of User-Agent strings rotated on each request to avoid detection.
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
];

/// HTTP client that wraps `reqwest` with timeouts and polite delays.
pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    /// Create a new fetcher with a randomly chosen User-Agent.
    pub fn new() -> Result<Self> {
        let idx = rand::thread_rng().gen_range(0..USER_AGENTS.len());
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(USER_AGENTS[idx])
            .build()?;
        Ok(Self { client })
    }

    /// Replace the underlying client with a new one using a different UA.
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

    /// Fetch a URL and return the response body as a string.
    ///
    /// Adds a random delay (1–3 s) before each request to be polite to
    /// upstream servers. The User-Agent is rotated after each request.
    pub async fn fetch(&mut self, url: &str) -> Result<String> {
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
