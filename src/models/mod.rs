//! # Models
//!
//! Core data types shared across the application. Everything from job-post
//! representations and resume data to search configuration and match results
//! lives here.
//!
//! ## Key types
//!
//! | Type               | Purpose                                         |
//! |--------------------|-------------------------------------------------|
//! | [`JobSource`]      | Enumeration of supported job-board sources.     |
//! | [`JobPost`]        | A single scraped job listing.                   |
//! | [`Resume`]         | Structured resume extracted from user input.    |
//! | [`SearchConfig`]   | Parameters controlling a crawl session.         |
//! | [`MatchResult`]    | Output of scoring one job post against a resume.|
//! | [`Command`]        | CLI commands emitted by the menu.               |

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Job Source ───────────────────────────────────────────────────────────────

/// Identifies which job-board or site a [`JobPost`] was scraped from.
///
/// Each variant maps to a specific crawler implementation in the [`crawler`]
/// module. The [`Custom`] variant allows ad-hoc sources to be tagged without
/// adding a new variant.
///
/// [`crawler`]: crate::crawler
/// [`Custom`]: JobSource::Custom
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobSource {
    /// Scraped from `indeed.com`.
    Indeed,
    /// Scraped from Google's job-search results page.
    GoogleJobs,
    /// Scraped from subreddits such as `r/forhire` and `r/jobbit`.
    Reddit,
    /// Scraped from X / Twitter (reserved for future use).
    Twitter,
    /// Scraped from the Hacker News "Who is Hiring?" monthly thread.
    HackerNews,
    /// Scraped from `craigslist.org` (reserved for future use).
    Craigslist,
    /// Scraped from `glassdoor.com` (reserved for future use).
    Glassdoor,
    /// Any source not covered by the enum variants above.
    Custom(String),
}

impl std::fmt::Display for JobSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Indeed => write!(f, "Indeed"),
            Self::GoogleJobs => write!(f, "Google Jobs"),
            Self::Reddit => write!(f, "X / Twitter"),
            Self::Twitter => write!(f, "X / Twitter"),
            Self::HackerNews => write!(f, "Hacker News"),
            Self::Craigslist => write!(f, "Craigslist"),
            Self::Glassdoor => write!(f, "Glassdoor"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

// ─── Job Post ─────────────────────────────────────────────────────────────────

/// A single job listing scraped from one of the supported sources.
///
/// Contains all the metadata a crawler can extract: title, company, location,
/// description, posting date, salary, job type, and free-form tags.
///
/// Every post is assigned a unique [`Uuid`] at crawl time and is timestamped
/// with the moment it was fetched.
///
/// [`Uuid`]: https://docs.rs/uuid/latest/uuid/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPost {
    /// Unique identifier generated at crawl time.
    pub id: String,
    /// Job title (e.g. "Senior Rust Engineer").
    pub title: String,
    /// Name of the hiring company, if detected.
    pub company: Option<String>,
    /// Location string (e.g. "San Francisco, CA" or "Remote").
    pub location: Option<String>,
    /// Full or snippet-level job description text.
    pub description: String,
    /// URL pointing to the original listing.
    pub url: String,
    /// Which site this post was scraped from.
    pub source: JobSource,
    /// When the job was originally posted, if available.
    pub posted_at: Option<DateTime<Utc>>,
    /// When our crawler fetched this post.
    pub crawled_at: DateTime<Utc>,
    /// Salary range as a free-form string (e.g. "$120k-$160k").
    pub salary: Option<String>,
    /// Employment type (e.g. "Full-time", "Contract", "Remote").
    pub job_type: Option<String>,
    /// Arbitrary tags attached by the crawler (e.g. subreddit name).
    pub tags: Vec<String>,
}

// ─── Resume ───────────────────────────────────────────────────────────────────

/// Structured representation of a user's resume.
///
/// Can be deserialised from JSON / YAML (for structured resumes) or built
/// from free text via [`Resume::from_text`], which performs simple keyword
/// extraction to populate skills and role titles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resume {
    /// Technical skills and technologies (e.g. "Rust", "Kubernetes").
    pub skills: Vec<String>,
    /// Total years of professional experience, if known.
    pub experience_years: Option<f32>,
    /// Role / job titles the user has held or is targeting.
    pub role_titles: Vec<String>,
    /// Additional free-form keywords for matching.
    pub keywords: Vec<String>,
    /// Preferred job location (e.g. "San Francisco" or "Remote").
    pub preferred_location: Option<String>,
    /// Preferred employment type (e.g. "Full-time", "Contract").
    pub preferred_job_type: Option<String>,
    /// Minimum acceptable salary.
    pub min_salary: Option<u64>,
}

impl Resume {
    /// Build a minimal [`Resume`] from a free-text description via simple
    /// keyword extraction.
    ///
    /// The method scans the input text for a hard-coded list of common tech
    /// skills and role keywords. Any matches are deduplicated, sorted, and
    /// stored as [`skills`] and [`role_titles`], respectively.
    ///
    /// All optional fields (`experience_years`, `preferred_location`, etc.)
    /// are left as `None` — they must be provided via structured JSON/YAML
    /// input.
    ///
    /// [`skills`]: Resume::skills
    /// [`role_titles`]: Resume::role_titles
    pub fn from_text(text: &str) -> Self {
        let lower = text.to_lowercase();
        let _words: Vec<&str> = lower.split_whitespace().collect();

        // Common tech skills to look for
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

        Self {
            skills,
            experience_years: None,
            role_titles,
            keywords: vec![],
            preferred_location: None,
            preferred_job_type: None,
            min_salary: None,
        }
    }
}

// ─── Search Config ────────────────────────────────────────────────────────────

/// Configuration that controls a crawl session.
///
/// Determines which keywords to search for, which sources to crawl, how many
/// results to collect, and basic filters like recency, location, and remote
/// preference.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Keywords to search for across all sources.
    pub keywords: Vec<String>,
    /// Which job sources to include in the crawl.
    pub sources: Vec<JobSource>,
    /// Maximum number of raw job posts to collect per source.
    pub max_results: usize,
    /// Only include posts newer than this many days.
    pub max_days_old: Option<i64>,
    /// Geographic filter (e.g. "San Francisco").
    pub location: Option<String>,
    /// If `true`, only return remote / work-from-home listings.
    pub remote_only: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            keywords: vec![],
            sources: vec![
                JobSource::Indeed,
                JobSource::GoogleJobs,
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

// ─── Match Result ─────────────────────────────────────────────────────────────

/// The result of scoring a single [`JobPost`] against the user's [`Resume`].
///
/// Contains the overall relevance score (0.0 – 1.0), which skills from the
/// resume were found in the post, which are missing, and which broader
/// keywords matched.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The job post that was scored.
    pub job: JobPost,
    /// Overall match score between 0.0 and 1.0.
    pub score: f64,
    /// Skills from the resume that were found in the job post.
    pub matched_skills: Vec<String>,
    /// Role-title or custom keywords from the resume that were found.
    pub matched_keywords: Vec<String>,
    /// Skills from the resume that were **not** found in the job post.
    pub missing_skills: Vec<String>,
}

// ─── CLI Command ──────────────────────────────────────────────────────────────

/// Commands that the interactive CLI menu can emit.
///
/// Each variant corresponds to a top-level action the user can take. The
/// application's main loop matches on these and dispatches to the appropriate
/// handler method.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Kick off a scan across all configured job sources.
    Scan,
    /// Search using a custom query string provided by the user.
    Search(String),
    /// Paginate through the cached match results.
    ViewResults,
    /// Load a resume from a file path (PDF, JSON, YAML, or plain text).
    LoadResume(String),
    /// Display the currently loaded resume summary.
    ShowResume,
    /// Filter or sort the current match results in-place.
    FilterResults,
    /// Exit the application.
    Quit,
}
