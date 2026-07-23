//! # Auto-Apply Module
//!
//! Automates the job application process:
//!
//! 1. **Cover letter generation** — crafts a personalised cover letter from resume
//!    intelligence + job match details
//! 2. **Application tracking** — persists applied jobs so you never double-apply
//! 3. **Quick open** — opens the application URL in your browser
//!
//! # Usage
//!
//! From the results viewer, press `a` to auto-apply to the selected job.
//! The tool will:
//! 1. Generate a tailored cover letter
//! 2. Save it to `~/.jobsense-parker/applications/{job_id}.md`
//! 3. Open the job URL in your browser
//! 4. Track that you applied (persisted in `~/.jobsense-parker/applied.json`)

use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::models::{JobPost, MatchResult, Resume};

// ─── Applied Job Record ──────────────────────────────────────────────────────

/// A record of a job we've applied to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedJob {
    /// Job post ID (matches `JobPost.id`).
    pub job_id: String,
    /// Job title for display.
    pub title: String,
    /// Company name.
    pub company: Option<String>,
    /// URL of the job posting.
    pub url: String,
    /// When we applied.
    pub applied_at: DateTime<Utc>,
    /// Match score at time of application.
    pub score: f64,
    /// Path to the generated cover letter on disk.
    pub cover_letter_path: Option<String>,
}

/// The persisted list of applied jobs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApplicationDatabase {
    pub applied: Vec<AppliedJob>,
}

impl ApplicationDatabase {
    /// Load applications from disk.
    pub fn load() -> Self {
        let dir = crate::storage::data_dir().ok();
        let path = dir.map(|d| d.join("applied.json"));
        if let Some(p) = path {
            if p.exists() {
                if let Ok(json) = std::fs::read_to_string(&p) {
                    if let Ok(db) = serde_json::from_str(&json) {
                        return db;
                    }
                }
            }
        }
        Self::default()
    }

    /// Save applications to disk.
    pub fn save(&self) {
        let dir = match crate::storage::ensure_dir() {
            Ok(d) => d,
            Err(_) => return,
        };
        let path = dir.join("applied.json");
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Check if we've already applied to a job.
    pub fn already_applied(&self, job_id: &str) -> bool {
        self.applied.iter().any(|a| a.job_id == job_id)
    }

    /// Record a new application.
    pub fn record(&mut self, job: &JobPost, score: f64, cover_path: Option<String>) {
        self.applied.push(AppliedJob {
            job_id: job.id.clone(),
            title: job.title.clone(),
            company: job.company.clone(),
            url: job.url.clone(),
            applied_at: Utc::now(),
            score,
            cover_letter_path: cover_path,
        });
        self.save();
    }

    /// List all applied jobs (newest first).
    pub fn list(&self) -> Vec<&AppliedJob> {
        let mut list: Vec<&AppliedJob> = self.applied.iter().collect();
        list.sort_by(|a, b| b.applied_at.cmp(&a.applied_at));
        list
    }
}

// ─── Cover Letter Generator ───────────────────────────────────────────────────

/// Generate a personalised cover letter based on resume + job match.
///
/// The letter is tailored:
/// - Opens with the job title and company
/// - Highlights matched skills
/// - Acknowledges missing skills (turns them into a learning opportunity)
/// - References seniority level and experience years
/// - Closes with enthusiasm
pub fn generate_cover_letter(result: &MatchResult, resume: &Resume) -> String {
    let job = &result.job;
    let company = job.company.as_deref().unwrap_or("the company");
    let seniority = resume
        .seniority
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Professional".to_string());
    let exp = resume
        .experience_years
        .map(|y| format!("{} years of experience", y))
        .unwrap_or_else(|| "strong industry experience".to_string());

    let matched = if result.matched_skills.is_empty() {
        "your required qualifications".to_string()
    } else {
        result.matched_skills.join(", ")
    };

    let missing = if result.missing_skills.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nWhile my direct experience doesn't cover {} yet, \
             I'm a fast learner and eager to develop these skills. \
             I've consistently picked up new technologies throughout my career.",
            result.missing_skills.join(", ")
        )
    };

    let focus = if resume.focus_areas.is_empty() {
        String::new()
    } else {
        format!("\n\nMy expertise focuses on {}.", resume.focus_areas.join(", "))
    };

    format!(
        r#"Subject: Application for {title} — {name}

Dear Hiring Team at {company},

I am writing to express my strong interest in the {title} position at {company}. \
As a {seniority} professional with {exp}, I am confident that my background \
aligns well with what you are looking for.

My relevant skills include: {matched}.{missing}{focus}

I am excited about the opportunity to contribute to {company}'s mission \
and would welcome the chance to discuss how my experience can add value to your team.

Thank you for your time and consideration.

Best regards,
[Your Name]
"#,
        title = job.title,
        name = resume.role_titles.first().map(|s| s.as_str()).unwrap_or("Candidate"),
        company = company,
        seniority = seniority,
        exp = exp,
        matched = matched,
        missing = missing,
        focus = focus,
    )
}

/// Save a cover letter to disk and return the file path.
pub fn save_cover_letter(job_id: &str, letter: &str) -> Option<String> {
    let dir = crate::storage::ensure_dir().ok()?;
    let app_dir = dir.join("applications");
    std::fs::create_dir_all(&app_dir).ok()?;
    let path = app_dir.join(format!("{}.md", job_id));
    std::fs::write(&path, letter).ok()?;
    Some(path.to_string_lossy().to_string())
}

/// Open a job URL in the browser and track the application.
pub fn apply_to_job(result: &MatchResult, resume: &Resume) {
    let job = &result.job;

    // Check if already applied
    let mut db = ApplicationDatabase::load();
    if db.already_applied(&job.id) {
        println!(
            "  {} Already applied to '{}' ({})",
            "·".yellow(),
            job.title,
            job.company.as_deref().unwrap_or("unknown")
        );
        return;
    }

    // Generate cover letter
    let letter = generate_cover_letter(result, resume);
    let cover_path = save_cover_letter(&job.id, &letter);

    // Open URL
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&job.url)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&job.url)
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", &job.url])
            .spawn();
    }

    // Track
    db.record(job, result.score, cover_path.clone());

    println!(
        "  {} Applied to '{}' @ {}",
        "✓".green(),
        job.title.bright_white(),
        job.company.as_deref().unwrap_or("unknown").cyan()
    );
    if let Some(path) = cover_path {
        println!(
            "    Cover letter saved: {}",
            path.dimmed()
        );
    }
    println!("    Application URL opened in browser.");
}
