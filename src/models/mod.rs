//! Data models used throughout the application.
//!
//! This module defines the core types that flow through the system:
//! job posts from crawlers, the user's resume, match results, scan history,
//! user preferences, and CLI commands. All types are serialisable so they
//! can be persisted to disk between sessions.

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Where a job posting was discovered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobSource {
    /// Remote OK job board.
    RemoteOk,
    /// Reddit (r/forhire, r/jobbit, etc.).
    Reddit,
    /// Hacker News "Who is Hiring?" threads.
    HackerNews,
    /// A user-defined source label (e.g. "LinkedIn", "Indeed").
    Custom(String),
}

impl std::fmt::Display for JobSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RemoteOk => write!(f, "Remote OK"),
            Self::Reddit => write!(f, "Reddit"),
            Self::HackerNews => write!(f, "Hacker News"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// A single job posting collected by a crawler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPost {
    /// Unique identifier (UUID v4).
    pub id: String,
    /// Job title (e.g. "Senior Rust Engineer").
    pub title: String,
    /// Company name, if available.
    pub company: Option<String>,
    /// Geographic location, if available.
    pub location: Option<String>,
    /// Full description text (plain-text, HTML stripped).
    pub description: String,
    /// Link to the original posting.
    pub url: String,
    /// Which source this was collected from.
    pub source: JobSource,
    /// When the posting was published (if the source provides it).
    pub posted_at: Option<DateTime<Utc>>,
    /// When our crawler fetched it.
    pub crawled_at: DateTime<Utc>,
    /// Salary range as a display string (e.g. "$80k - $120k").
    pub salary: Option<String>,
    /// Employment type (e.g. "full-time", "contract") if known.
    pub job_type: Option<String>,
    /// Source-specific tags (e.g. subreddit name, tech tags).
    pub tags: Vec<String>,
}

/// The user's parsed resume.
///
/// Can be constructed from a plain-text string via [`Resume::from_text`],
/// from a JSON file, or from a YAML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resume {
    /// Recognised technical skills (e.g. "rust", "docker", "aws").
    pub skills: Vec<String>,
    /// Years of professional experience extracted from text.
    pub experience_years: Option<f32>,
    /// Role/career level indicators (e.g. "engineer", "senior", "lead").
    pub role_titles: Vec<String>,
    /// All significant keywords extracted from the resume text.
    pub keywords: Vec<String>,
    /// Preferred work location, if mentioned.
    pub preferred_location: Option<String>,
    /// Preferred employment type, if mentioned (e.g. "remote", "full-time").
    pub preferred_job_type: Option<String>,
    /// Minimum acceptable salary, if known.
    pub min_salary: Option<u64>,
}

impl Resume {
    /// Parse a plain-text resume and extract structured fields.
    ///
    /// Detection methods:
    /// * **Skills** — matched against a built-in keyword list (tech stacks, roles).
    /// * **Experience years** — regex for patterns like `"5 years"`, `"10+ years"`, `"3 yrs"`.
    /// * **Location** — regex for patterns like `"based in NYC"`, `"located in San Francisco"`.
    /// * **Job type** — regex for `"full-time"`, `"part-time"`, `"contract"`, `"remote"`.
    /// * **Keywords** — all significant alphanumeric tokens (3+ chars) not in a stop list.
    pub fn from_text(text: &str) -> Self {
        let lower = text.to_lowercase();

        // ── Skills ──────────────────────────────────────────────────────

        let skill_keywords = [
            "rust", "python", "go", "golang", "java", "typescript", "javascript",
            "react", "angular", "vue", "node", "c++", "c#", "kotlin", "swift",
            "sql", "postgresql", "mysql", "mongodb", "redis", "docker", "kubernetes",
            "k8s", "aws", "gcp", "azure", "devops", "ci/cd", "terraform", "ansible",
            "machine learning", "ml", "ai", "data science", "deep learning",
            "blockchain", "solidity", "web3", "frontend", "backend", "fullstack",
            "api", "rest", "graphql", "git", "linux", "agile", "scrum",
        ];

        let role_keywords = [
            "engineer", "developer", "architect", "manager", "lead", "senior",
            "staff", "principal", "intern", "junior", "sde", "swe",
            "data scientist", "analyst", "consultant", "devops",
        ];

        let mut skills: Vec<String> = skill_keywords
            .iter()
            .filter(|kw| lower.contains(*kw))
            .map(|&s| s.to_string())
            .collect();
        skills.sort();
        skills.dedup();

        let mut role_titles: Vec<String> = role_keywords
            .iter()
            .filter(|kw| lower.contains(*kw))
            .map(|&s| s.to_string())
            .collect();
        role_titles.sort();
        role_titles.dedup();

        // ── Experience years ────────────────────────────────────────────

        let exp_re = Regex::new(r"(?i)(\d+)\+?\s*(?:years?|yrs?)\s*(?:of\s+)?(?:experience|exp)").unwrap();
        let experience_years = exp_re.captures(&lower).and_then(|c| {
            c.get(1)?.as_str().parse::<f32>().ok()
        });

        // ── Preferred location ──────────────────────────────────────────

        let loc_re = Regex::new(
            r"(?i)(?:based|located|living|situated)\s+(?:in|at|near)\s+([a-z][a-z\s.-]+?)(?:[,.!]|$)"
        ).unwrap();
        let preferred_location = loc_re.captures(&lower).and_then(|c| {
            let loc = c.get(1)?.as_str().trim().to_string();
            if loc.len() > 50 { None } else { Some(loc) }
        });

        // ── Preferred job type ──────────────────────────────────────────

        let type_re = Regex::new(
            r"(?i)(?:looking|seeking|want|prefer|open|available)\s+(?:for|a|an)?\s*(full[- ]time|part[- ]time|contract|remote|hybrid|onsite)"
        ).unwrap();
        let preferred_job_type = type_re.captures(&lower).and_then(|c| {
            Some(c.get(1)?.as_str().to_string())
        });

        // If no explicit job-type sentence, scan for the keywords directly
        let preferred_job_type = preferred_job_type.or_else(|| {
            for t in &["full-time", "full time", "part-time", "part time", "contract", "remote", "hybrid", "onsite"] {
                if lower.contains(t) {
                    return Some(t.to_string());
                }
            }
            None
        });

        // ── Keywords (significant tokens) ───────────────────────────────

        let stop_words = [
            "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her",
            "was", "one", "our", "out", "has", "have", "been", "some", "same", "also",
            "its", "than", "them", "into", "two", "more", "these", "like", "over",
            "such", "that", "this", "with", "from", "your", "which", "each", "will",
            "about", "between", "under", "very", "just", "their", "would", "after",
            "could", "should", "other", "than", "then", "there", "where", "while",
            "because", "before", "does", "doing", "done", "much", "many", "most",
            "must", "need", "take", "make", "made", "well", "work", "year", "years",
        ];

        let mut keywords: Vec<String> = text
            .split(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
            .filter(|w| w.len() >= 3)
            .map(|w| w.to_lowercase())
            .filter(|w| !stop_words.contains(&w.as_str()))
            .collect();
        keywords.sort();
        keywords.dedup();

        Self {
            skills,
            experience_years,
            role_titles,
            keywords,
            preferred_location,
            preferred_job_type,
            min_salary: None,
        }
    }
}

/// Configuration for a single crawl/search operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Keywords to search for across sources.
    pub keywords: Vec<String>,
    /// Which sources to include in the search.
    pub sources: Vec<JobSource>,
    /// Maximum number of job posts to collect per source.
    pub max_results: usize,
    /// Maximum age of job posts in days (None = no limit).
    pub max_days_old: Option<i64>,
    /// Geographic preference to narrow results.
    pub location: Option<String>,
    /// Whether to filter for remote-only positions.
    pub remote_only: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            keywords: vec![],
            sources: vec![
                JobSource::RemoteOk,
                JobSource::Reddit,
                JobSource::HackerNews,
            ],
            max_results: 50,
            max_days_old: Some(7),
            location: None,
            remote_only: false,
        }
    }
}

/// The output of matching a job post against a resume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    /// The job post that was matched.
    pub job: JobPost,
    /// Overall compatibility score (0.0 – 1.0).
    pub score: f64,
    /// Skills from the resume that were found in the job description.
    pub matched_skills: Vec<String>,
    /// Keywords from the resume that were found in the job description.
    pub matched_keywords: Vec<String>,
    /// Skills from the resume that were NOT found in the job description.
    pub missing_skills: Vec<String>,
}

/// Represents a single scan session — what was searched, when, and the results.
/// This is the "impact profile" that persists across runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRecord {
    /// Unique scan identifier (UUID v4).
    pub id: String,
    /// When the scan was performed.
    pub timestamp: DateTime<Utc>,
    /// The query string used during the scan.
    pub query: String,
    /// Number of sources crawled.
    pub source_count: usize,
    /// Total raw jobs found before matching.
    pub total_jobs_found: usize,
    /// The highest match score across all results.
    pub top_score: f64,
    /// Number of results that passed the threshold.
    pub result_count: usize,
}

/// A command issued by the user via the interactive menu or CLI arguments.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Run a scan against all configured sources.
    Scan,
    /// Search with a custom query string.
    Search(String),
    /// Display the last scan results.
    ViewResults,
    /// Load a resume from a file path or raw text.
    LoadResume(String),
    /// Show the currently loaded resume.
    ShowResume,
    /// Filter/sort the current results.
    FilterResults,
    /// Exit the application.
    Quit,
}

/// Persisted user preferences that survive between sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Preferred work location.
    pub preferred_location: Option<String>,
    /// Preferred employment type (e.g. "remote", "full-time").
    pub preferred_job_type: Option<String>,
    /// Minimum acceptable salary.
    pub min_salary: Option<u64>,
    /// Which sources to search by default.
    pub active_sources: Vec<JobSource>,
    /// Maximum results per scan.
    pub max_results: usize,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_location: None,
            preferred_job_type: None,
            min_salary: None,
            active_sources: vec![
                JobSource::RemoteOk,
                JobSource::Reddit,
                JobSource::HackerNews,
            ],
            max_results: 50,
        }
    }
}
