//! Persistent JSON storage for resumes, preferences, query history, scan records,
//! and cached results. All data is stored under `$HOME/.jobsense-parker/`.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::models::{MatchResult, Resume, ScanRecord, UserPreferences};

/// Directory name under `$HOME` for storing app data.
const DATA_DIR: &str = ".jobsense-parker";

/// Returns the path to the data directory (no guarantee it exists).
fn data_dir() -> Result<PathBuf> {
    let home = dirs_next::home_dir().context("Cannot determine home directory")?;
    Ok(home.join(DATA_DIR))
}

/// Returns the path to the data directory, creating it if necessary.
fn ensure_dir() -> Result<PathBuf> {
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
pub fn push_query(query: &str) -> Result<()> {
    let mut history = load_query_history().unwrap_or_default();
    // Avoid duplicates (recent-first)
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
