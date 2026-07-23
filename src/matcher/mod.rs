//! Resume-to-job scoring engine.
//!
//! The [`Matcher`] struct holds a loaded resume and compares it against
//! job posts, producing a [`MatchResult`] with a score between 0.0 and 1.0.
//!
//! ## How Scoring Works (ML)
//!
//! Scoring uses **TF-IDF vectorization + cosine similarity** — an unsupervised
//! ML technique. The resume text and each job text are converted into term
//! vectors weighted by term frequency × inverse document frequency. The cosine
//! of the angle between vectors is the match score.
//!
//! This replaces the old heuristic weight-tuning approach ("slope") that used
//! manually set percentages for skills, keywords, role titles, etc.

pub mod scoring;

use std::collections::HashSet;

use crate::models::{JobPost, MatchResult, Resume};
use scoring::{build_job_text, score_batch};

/// Compares a resume against job posts and produces scored results.
pub struct Matcher {
    resume: Option<Resume>,
    threshold: f64,
}

impl Matcher {
    /// Create a new matcher with no loaded resume.
    pub fn new() -> Self {
        Self {
            resume: None,
            threshold: 0.15,
        }
    }

    /// Load (or replace) the resume to match against.
    pub fn load_resume(&mut self, resume: Resume) {
        self.resume = Some(resume);
    }

    /// Whether a resume has been loaded.
    pub fn has_resume(&self) -> bool {
        self.resume.is_some()
    }

    /// Returns a reference to the loaded resume, if any.
    pub fn resume(&self) -> Option<&Resume> {
        self.resume.as_ref()
    }

    /// Score all job posts against the loaded resume using TF-IDF + cosine
    /// similarity (ML), and return results sorted by score descending.
    ///
    /// Results below [`threshold`] are filtered out.
    ///
    /// Returns an empty vec if no resume is loaded.
    pub fn score_all(&self, jobs: &[JobPost]) -> Vec<MatchResult> {
        let resume = match self.resume.as_ref() {
            Some(r) => r,
            None => return vec![],
        };

        if jobs.is_empty() {
            return vec![];
        }

        // ── Step 1: Compute ML scores via TF-IDF + cosine similarity ──
        // This fits a vectorizer on the full corpus and scores every job
        // in one batch. It's the core ML step.
        let ml_scores = score_batch(jobs, resume);

        // ── Step 2: Build display details (matched/missing skills) ─────
        // These are for the UI only — the score itself comes from ML.
        let mut results: Vec<MatchResult> = jobs
            .iter()
            .zip(ml_scores.iter())
            .filter(|(_, score)| **score >= self.threshold)
            .map(|(job, score)| {
                let job_text = build_job_text(job);
                let job_lower = job_text.to_lowercase();

                let (matched_skills, missing_skills) = find_skill_matches(&resume.skills, &job_lower);
                let matched_keywords = find_keyword_matches(
                    &resume.keywords,
                    &resume.role_titles,
                    &job_lower,
                );

                MatchResult {
                    job: job.clone(),
                    score: *score,
                    matched_skills: dedup_ordered(matched_skills),
                    matched_keywords: dedup_ordered(matched_keywords),
                    missing_skills: dedup_ordered(missing_skills),
                }
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }
}

/// Find which skills from the resume match in the job text (case-insensitive).
/// Returns (matched, missing).
fn find_skill_matches(skills: &[String], job_lower: &str) -> (Vec<String>, Vec<String>) {
    let mut matched = Vec::new();
    let mut missing = Vec::new();

    for skill in skills {
        let skill_lower = skill.to_lowercase();
        if job_lower.contains(&skill_lower) {
            matched.push(skill.clone());
        } else {
            missing.push(skill.clone());
        }
    }

    (matched, missing)
}

/// Find which keywords from the resume match in the job text.
fn find_keyword_matches(
    keywords: &[String],
    role_titles: &[String],
    job_lower: &str,
) -> Vec<String> {
    let all_keywords: Vec<String> = keywords
        .iter()
        .chain(role_titles.iter())
        .cloned()
        .collect();

    all_keywords
        .into_iter()
        .filter(|kw| job_lower.contains(&kw.to_lowercase()))
        .collect()
}

/// Deduplicate items in a vector while preserving insertion order.
fn dedup_ordered<T: std::hash::Hash + Eq + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::JobSource;
    use chrono::Utc;

    fn make_job(title: &str, desc: &str) -> JobPost {
        JobPost {
            id: "test".into(),
            title: title.into(),
            company: None,
            location: None,
            description: desc.into(),
            url: "https://example.com/job".into(),
            source: JobSource::Custom("Indeed".into()),
            posted_at: None,
            crawled_at: Utc::now(),
            salary: None,
            job_type: None,
            tags: vec![],
        }
    }

    fn make_resume(skills: &[&str], roles: &[&str]) -> Resume {
        Resume {
            skills: skills.iter().map(|&s| s.to_string()).collect(),
            experience_years: None,
            role_titles: roles.iter().map(|&r| r.to_string()).collect(),
            keywords: vec![],
            preferred_location: None,
            preferred_job_type: None,
            min_salary: None,
            seniority: None,
            focus_areas: vec![],
            education: vec![],
            certifications: vec![],
        }
    }

    #[test]
    fn test_score_rust_job() {
        let mut matcher = Matcher::new();
        matcher.load_resume(make_resume(&["rust", "python"], &["engineer"]));
        let job = make_job("Rust Engineer", "We need a Rust engineer with Python experience");
        let results = matcher.score_all(&[job]);
        assert!(!results.is_empty());
        assert!(results[0].score > 0.3);
        assert!(results[0].matched_skills.contains(&"rust".to_string()));
    }

    #[test]
    fn test_no_resume_returns_empty() {
        let matcher = Matcher::new();
        let job = make_job("Engineer", "some job");
        let results = matcher.score_all(&[job]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_partial_match() {
        let mut matcher = Matcher::new();
        matcher.load_resume(make_resume(
            &["rust", "kubernetes", "react"],
            &["engineer"],
        ));
        let job = make_job("Frontend Engineer", "React developer position");
        let results = matcher.score_all(&[job]);
        assert!(!results.is_empty());
        assert!(results[0].matched_skills.contains(&"react".to_string()));
        assert!(!results[0].matched_skills.contains(&"rust".to_string()));
    }

    #[test]
    fn test_ranks_rust_higher() {
        let mut matcher = Matcher::new();
        matcher.load_resume(make_resume(
            &["rust", "python", "kubernetes"],
            &["software engineer"],
        ));
        let jobs = vec![
            make_job("Frontend Python Developer", "Python Django React TypeScript web development"),
            make_job("Senior Rust Engineer", "Rust Python Kubernetes distributed systems"),
            make_job("Barista", "Coffee shop experience making and serving"),
        ];
        let results = matcher.score_all(&jobs);
        // Rust job should rank first (most technical overlap)
        assert!(!results.is_empty());
        assert!(results[0].job.title.contains("Rust"));
    }

    #[test]
    fn test_threshold_filters_unrelated() {
        let mut matcher = Matcher::new();
        matcher.threshold = 0.9; // between barista (0.82) and rust (0.98)
        matcher.load_resume(make_resume(
            &["rust", "python", "kubernetes"],
            &["software engineer"],
        ));
        let jobs = vec![
            make_job("Barista", "Coffee shop experience making coffee"),
            make_job("Senior Rust Engineer", "Rust Python Kubernetes Go systems"),
        ];
        let results = matcher.score_all(&jobs);
        // Only the highly-relevant Rust job should pass the high threshold
        assert!(results.iter().any(|r| r.job.title.contains("Rust")));
        assert!(!results.iter().any(|r| r.job.title.contains("Barista")));
    }

    #[test]
    fn test_empty_jobs_returns_empty() {
        let mut matcher = Matcher::new();
        matcher.load_resume(make_resume(&["rust"], &["engineer"]));
        let results = matcher.score_all(&[]);
        assert!(results.is_empty());
    }
}
