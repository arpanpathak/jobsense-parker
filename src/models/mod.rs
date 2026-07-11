use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Job Source ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobSource {
    Indeed,
    GoogleJobs,
    Reddit,
    HackerNews,
    Custom(String),
}

impl std::fmt::Display for JobSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Indeed => write!(f, "Indeed"),
            Self::GoogleJobs => write!(f, "Google Jobs"),
            Self::Reddit => write!(f, "Reddit"),
            Self::HackerNews => write!(f, "Hacker News"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

// ─── Job Post ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPost {
    pub id: String,
    pub title: String,
    pub company: Option<String>,
    pub location: Option<String>,
    pub description: String,
    pub url: String,
    pub source: JobSource,
    pub posted_at: Option<DateTime<Utc>>,
    pub crawled_at: DateTime<Utc>,
    pub salary: Option<String>,
    pub job_type: Option<String>,
    pub tags: Vec<String>,
}

// ─── Resume ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resume {
    pub skills: Vec<String>,
    pub experience_years: Option<f32>,
    pub role_titles: Vec<String>,
    pub keywords: Vec<String>,
    pub preferred_location: Option<String>,
    pub preferred_job_type: Option<String>,
    pub min_salary: Option<u64>,
}

impl Resume {
    pub fn from_text(text: &str) -> Self {
        let lower = text.to_lowercase();

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

// ─── Search Config ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub keywords: Vec<String>,
    pub sources: Vec<JobSource>,
    pub max_results: usize,
    pub max_days_old: Option<i64>,
    pub location: Option<String>,
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

// ─── Match Result ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub job: JobPost,
    pub score: f64,
    pub matched_skills: Vec<String>,
    pub matched_keywords: Vec<String>,
    pub missing_skills: Vec<String>,
}

// ─── Scan Record (impact profile) ─────────────────────────────────────────

/// Represents a single scan session — what was searched, when, and the results.
/// This is the "impact profile" that persists across runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub query: String,
    pub source_count: usize,
    pub total_jobs_found: usize,
    pub top_score: f64,
    pub result_count: usize,
}

// ─── CLI Command ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Scan,
    Search(String),
    ViewResults,
    LoadResume(String),
    ShowResume,
    FilterResults,
    Quit,
}

// ─── User Preferences ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_location: Option<String>,
    pub preferred_job_type: Option<String>,
    pub min_salary: Option<u64>,
    pub active_sources: Vec<JobSource>,
    pub max_results: usize,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_location: None,
            preferred_job_type: None,
            min_salary: None,
            active_sources: vec![
                JobSource::Indeed,
                JobSource::GoogleJobs,
                JobSource::Reddit,
                JobSource::HackerNews,
            ],
            max_results: 50,
        }
    }
}
