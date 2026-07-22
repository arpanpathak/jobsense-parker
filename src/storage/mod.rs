//! # JSON Persistence Layer
//!
//! All user data is stored as JSON files under `~/.jobsense-parker/`. This
//! module handles reading and writing every data type with proper error
//! handling and automatic directory creation.
//!
//! ## File Layout
//!
//! ```text
//! ~/.jobsense-parker/
//! ├── resume.json          ← Parsed resume (skills, roles, keywords)
//! ├── preferences.json     ← User preferences (sources, max results)
//! ├── companies.json       ← Cached companies with careers-page URLs
//! ├── queries.json         ← Recent search queries (capped at 50)
//! ├── scan_history.json    ← Scan records (capped at 100)
//! └── last_results.json    ← Most recent match results
//! ```
//!
//! ## First-Run Behaviour
//!
//! On first run, the directory is created automatically and:
//! - `companies.json` is seeded with 80+ major tech companies
//! - All other files are created on first write

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::models::{CompanyDatabase, MatchResult, Resume, ScanRecord, UserPreferences};

/// Directory name under `$HOME` for storing app data.
const DATA_DIR: &str = ".jobsense-parker";

/// Returns the path to the data directory (does not guarantee it exists).
pub fn data_dir() -> Result<PathBuf> {
    let home = dirs_next::home_dir().context("Cannot determine home directory")?;
    Ok(home.join(DATA_DIR))
}

/// Returns the path to the data directory, creating it if necessary.
pub fn ensure_dir() -> Result<PathBuf> {
    let dir = data_dir()?;
    std::fs::create_dir_all(&dir).context("Failed to create data directory")?;
    Ok(dir)
}

// ─── Resume ───────────────────────────────────────────────────────────

/// Persist the user's resume to `~/.jobsense-parker/resume.json`.
pub fn save_resume(resume: &Resume) -> Result<()> {
    let dir = ensure_dir()?;
    let path = dir.join("resume.json");
    let json = serde_json::to_string_pretty(resume)?;
    std::fs::write(&path, json).context("Failed to write resume.json")
}

/// Load the previously persisted resume from `~/.jobsense-parker/resume.json`.
///
/// Returns `Ok(None)` if no resume has been saved yet.
pub fn load_resume() -> Result<Option<Resume>> {
    let dir = data_dir()?;
    let path = dir.join("resume.json");
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(&path).context("Failed to read resume.json")?;
    serde_json::from_str(&json)
        .map(Some)
        .context("Failed to parse resume.json")
}

// ─── Preferences ──────────────────────────────────────────────────────

/// Persist user preferences to `~/.jobsense-parker/preferences.json`.
pub fn save_preferences(prefs: &UserPreferences) -> Result<()> {
    let dir = ensure_dir()?;
    let path = dir.join("preferences.json");
    let json = serde_json::to_string_pretty(prefs)?;
    std::fs::write(&path, json).context("Failed to write preferences.json")
}

/// Load user preferences, returning defaults if no file exists.
pub fn load_preferences() -> Result<UserPreferences> {
    let dir = data_dir()?;
    let path = dir.join("preferences.json");
    if !path.exists() {
        return Ok(UserPreferences::default());
    }
    let json = std::fs::read_to_string(&path).context("Failed to read preferences.json")?;
    serde_json::from_str(&json).context("Failed to parse preferences.json")
}

// ─── Query History ────────────────────────────────────────────────────

/// Persist the query history list to `~/.jobsense-parker/queries.json`.
pub fn save_query_history(queries: &[String]) -> Result<()> {
    let dir = ensure_dir()?;
    let path = dir.join("queries.json");
    let json = serde_json::to_string_pretty(queries)?;
    std::fs::write(&path, json).context("Failed to write queries.json")
}

/// Load previously saved query history (empty vec if no file exists).
pub fn load_query_history() -> Result<Vec<String>> {
    let dir = data_dir()?;
    let path = dir.join("queries.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = std::fs::read_to_string(&path).context("Failed to read queries.json")?;
    serde_json::from_str(&json).context("Failed to parse queries.json")
}

/// Add a query to the front of the history (deduped, capped at 50).
///
/// If the query already exists in history it is moved to the front.
pub fn push_query(query: &str) -> Result<()> {
    let mut history = load_query_history().unwrap_or_default();
    history.retain(|q| q != query);
    history.insert(0, query.to_string());
    if history.len() > 50 {
        history.truncate(50);
    }
    save_query_history(&history)
}

// ─── Scan History / Impact Profile ────────────────────────────────────

/// Persist all scan records to `~/.jobsense-parker/scan_history.json`.
pub fn save_scan_history(records: &[ScanRecord]) -> Result<()> {
    let dir = ensure_dir()?;
    let path = dir.join("scan_history.json");
    let json = serde_json::to_string_pretty(records)?;
    std::fs::write(&path, json).context("Failed to write scan_history.json")
}

/// Load the full scan history (empty vec if no file exists).
pub fn load_scan_history() -> Result<Vec<ScanRecord>> {
    let dir = data_dir()?;
    let path = dir.join("scan_history.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = std::fs::read_to_string(&path).context("Failed to read scan_history.json")?;
    serde_json::from_str(&json).context("Failed to parse scan_history.json")
}

/// Insert a scan record at the front of the history list (capped at 100).
pub fn push_scan_record(record: &ScanRecord) -> Result<()> {
    let mut history = load_scan_history().unwrap_or_default();
    history.insert(0, record.clone());
    if history.len() > 100 {
        history.truncate(100);
    }
    save_scan_history(&history)
}

// ─── Last Results Cache ───────────────────────────────────────────────

/// Persist the most recent match results to `~/.jobsense-parker/last_results.json`.
///
/// This allows `--results` and the "View results" menu option to work
/// across restarts without re-running a scan.
pub fn save_last_results(results: &[MatchResult]) -> Result<()> {
    let dir = ensure_dir()?;
    let path = dir.join("last_results.json");
    let json = serde_json::to_string_pretty(results)?;
    std::fs::write(&path, json).context("Failed to write last_results.json")
}

/// Load the most recent match results from disk (empty vec if no file exists).
pub fn load_last_results() -> Result<Vec<MatchResult>> {
    let dir = data_dir()?;
    let path = dir.join("last_results.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = std::fs::read_to_string(&path).context("Failed to read last_results.json")?;
    serde_json::from_str(&json).context("Failed to parse last_results.json")
}

// ─── Company Database ─────────────────────────────────────────────────

/// Persist the company database to `~/.jobsense-parker/companies.json`.
pub fn save_company_database(db: &CompanyDatabase) -> Result<()> {
    let dir = ensure_dir()?;
    let path = dir.join("companies.json");
    let json = serde_json::to_string_pretty(db)?;
    std::fs::write(&path, json).context("Failed to write companies.json")
}

/// Load the company database.
///
/// On first run the file won't exist, so we seed it with a curated list of
/// well-known tech companies and their careers-page URLs. See the
/// [`seed_companies`] function for the full list.
///
/// After seeding, the database is persisted immediately so subsequent runs
/// load from disk.
pub fn load_company_database() -> Result<CompanyDatabase> {
    let dir = data_dir()?;
    let path = dir.join("companies.json");
    if !path.exists() {
        let mut db = CompanyDatabase::new();
        seed_companies(&mut db);
        save_company_database(&db)?;
        eprintln!("  + Seeded {} companies into database.", db.companies.len());
        return Ok(db);
    }
    let json = std::fs::read_to_string(&path).context("Failed to read companies.json")?;
    serde_json::from_str(&json).context("Failed to parse companies.json")
}

/// Guess a careers-page URL from a company name (used for auto-discovery).
///
/// Takes the company name, lowercases it, removes non-alphanumeric
/// characters, and constructs `https://careers.{slug}.com/`.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(guess_careers_url("Google"), "https://careers.google.com/");
/// assert_eq!(guess_careers_url("Stripe"), "https://careers.stripe.com/");
/// ```
///
/// This won't always work (e.g. Meta → `careers.meta.com` is wrong),
/// but it's a reasonable starting guess. Users can override with
/// `--add-company` or the interactive menu.
pub fn guess_careers_url(name: &str) -> String {
    let slug = name
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '.', "")
        .trim()
        .to_string();
    if slug.is_empty() {
        return String::new();
    }
    format!("https://careers.{slug}.com/")
}

/// Seed the database with a curated list of major companies.
///
/// This list includes big tech, fintech, enterprise/SaaS, e-commerce,
/// gaming, and notable startups. Each entry has a manually verified
/// careers-page URL that the [`CompanyCrawler`](crate::crawler::company::CompanyCrawler)
/// will attempt to parse with heuristics.
fn seed_companies(db: &mut CompanyDatabase) {
    let entries: &[(&str, &str)] = &[
        // ── Big Tech ───────────────────────────────────────────────────
        ("Google", "https://careers.google.com/jobs"),
        ("Meta", "https://www.metacareers.com/jobs"),
        ("Apple", "https://jobs.apple.com/en-us"),
        ("Amazon", "https://www.amazon.jobs/en"),
        ("Microsoft", "https://careers.microsoft.com/us/en"),
        ("Netflix", "https://jobs.netflix.com"),
        ("Spotify", "https://www.lifeatspotify.com/jobs"),
        ("Stripe", "https://stripe.com/jobs"),
        ("Shopify", "https://www.shopify.com/careers"),
        ("GitLab", "https://about.gitlab.com/jobs"),
        ("Uber", "https://www.uber.com/us/en/careers"),
        ("Airbnb", "https://www.airbnb.com/careers"),
        ("Twitter/X", "https://about.twitter.com/en/careers"),
        ("LinkedIn", "https://careers.linkedin.com"),
        ("Pinterest", "https://www.pinterestcareers.com"),
        ("Square", "https://squareup.com/us/en/careers"),
        ("Slack", "https://slack.com/careers"),
        ("Dropbox", "https://www.dropbox.com/jobs"),
        ("Salesforce", "https://www.salesforce.com/company/careers"),
        ("Adobe", "https://www.adobe.com/careers"),
        ("Atlassian", "https://www.atlassian.com/company/careers"),
        ("Reddit", "https://www.redditinc.com/careers"),
        ("Discord", "https://discord.com/jobs"),
        ("Figma", "https://www.figma.com/careers"),
        ("Notion", "https://www.notion.so/careers"),
        ("Canva", "https://www.canva.com/careers"),
        ("Palantir", "https://www.palantir.com/careers"),
        ("Datadog", "https://www.datadoghq.com/careers"),
        ("Snowflake", "https://www.snowflake.com/careers"),
        ("Cloudflare", "https://www.cloudflare.com/careers"),
        ("Cisco", "https://jobs.cisco.com"),
        ("Oracle", "https://www.oracle.com/careers"),
        ("IBM", "https://www.ibm.com/careers"),
        ("Intel", "https://www.intel.com/content/www/us/en/jobs"),
        ("AMD", "https://www.amd.com/en/corporate/careers"),
        ("NVIDIA", "https://nvidia.wd5.myworkdayjobs.com/NVIDIAExternalSite"),
        ("Tesla", "https://www.tesla.com/careers"),
        ("SpaceX", "https://www.spacex.com/careers"),
        // ── Finance / Fintech ──────────────────────────────────────────
        ("Goldman Sachs", "https://www.goldmansachs.com/careers"),
        ("JPMorgan Chase", "https://jpmc.fa.oraclecloud.com/careers"),
        ("Morgan Stanley", "https://www.morganstanley.com/careers"),
        ("Citadel", "https://www.citadel.com/careers"),
        ("Jane Street", "https://www.janestreet.com/join-jane-street"),
        ("Two Sigma", "https://www.twosigma.com/careers"),
        ("Bridgewater", "https://www.bridgewater.com/careers"),
        ("Coinbase", "https://www.coinbase.com/careers"),
        ("Robinhood", "https://careers.robinhood.com"),
        ("Chime", "https://www.chime.com/careers"),
        ("Plaid", "https://plaid.com/careers"),
        ("Brex", "https://www.brex.com/careers"),
        ("Rippling", "https://www.rippling.com/careers"),
        ("Deel", "https://www.deel.com/careers"),
        // ── Enterprise / Cloud ─────────────────────────────────────────
        ("VMware", "https://careers.vmware.com"),
        ("ServiceNow", "https://www.servicenow.com/careers"),
        ("Workday", "https://www.workday.com/en-us/careers.html"),
        ("SAP", "https://www.sap.com/careers"),
        ("Databricks", "https://www.databricks.com/careers"),
        ("Confluent", "https://www.confluent.io/careers"),
        ("Elastic", "https://www.elastic.co/about/careers"),
        ("MongoDB", "https://www.mongodb.com/careers"),
        ("Redis", "https://redis.com/company/careers"),
        ("HashiCorp", "https://www.hashicorp.com/careers"),
        ("New Relic", "https://newrelic.com/about/careers"),
        ("Splunk", "https://www.splunk.com/en_us/careers.html"),
        // ── E-commerce / Retail ────────────────────────────────────────
        ("eBay", "https://www.ebayinc.com/careers"),
        ("Walmart", "https://careers.walmart.com"),
        ("Target", "https://corporate.target.com/careers"),
        ("Wayfair", "https://www.wayfair.com/careers"),
        ("Doordash", "https://careers.doordash.com"),
        ("Instacart", "https://instacart.careers"),
        // ── Gaming ─────────────────────────────────────────────────────
        ("Riot Games", "https://www.riotgames.com/en/work-with-us"),
        ("Blizzard", "https://www.blizzard.com/en-us/careers"),
        ("Epic Games", "https://www.epicgames.com/site/en-US/careers"),
        ("Unity", "https://careers.unity.com"),
        ("Roblox", "https://corp.roblox.com/careers"),
        ("Electronic Arts", "https://www.ea.com/careers"),
        // ── Other notable ──────────────────────────────────────────────
        ("Samsara", "https://www.samsara.com/careers"),
        ("Vercel", "https://vercel.com/careers"),
        ("Netlify", "https://www.netlify.com/careers"),
        ("Railway", "https://railway.app/careers"),
        ("Supabase", "https://supabase.com/careers"),
        ("Fly.io", "https://fly.io/jobs"),
    ];
    for (name, url) in entries {
        db.add(name, url);
    }
}
