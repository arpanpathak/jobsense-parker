//! # Matcher
//!
//! Scores job posts against a loaded resume using a combination of exact
//! keyword matching, fuzzy string comparison ([Jaro-Winkler]), and configurable
//! bonuses (title alignment, location preference, job-type match).
//!
//! ## Scoring breakdown
//!
//! | Component          | Weight | Description                                     |
//! |--------------------|--------|-------------------------------------------------|
//! | Skill match ratio  | 50%    | Fraction of resume skills found in the job post |
//! | Keyword match ratio| 25%    | Fraction of resume keywords / roles found       |
//! | Title bonus        | 10%    | Job title contains a resume role title           |
//! | Location bonus     | 10%    | Job location overlaps with preferred location   |
//! | Job-type bonus     |  5%    | Job type matches preferred type                 |
//!
//! [Jaro-Winkler]: https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance

use std::collections::HashSet;

use crate::models::{JobPost, MatchResult, Resume};
use strsim::jaro_winkler;

// ─── Matcher ──────────────────────────────────────────────────────────────────

/// The core matching engine that compares job posts against a loaded resume.
///
/// Holds an optional [`Resume`] and a configurable score threshold. Posts
/// whose final score falls below the threshold are discarded.
///
/// ## Example
///
/// ```no_run
/// use jobsense_parker::matcher::Matcher;
/// use jobsense_parker::models::Resume;
///
/// let mut matcher = Matcher::new();
/// matcher.load_resume(Resume::from_text("Senior Rust engineer, 5 years"));
/// // … then score jobs …
/// ```
pub struct Matcher {
    /// The loaded resume, if any.
    resume: Option<Resume>,
    /// Minimum score (0.0 – 1.0) for a result to be retained.
    threshold: f64,
}

impl Matcher {
    /// Create a new [`Matcher`] with no resume loaded and the default
    /// threshold of `0.15`.
    pub fn new() -> Self {
        Self {
            resume: None,
            threshold: 0.15,
        }
    }

    /// Create a new [`Matcher`] with a custom score threshold.
    ///
    /// The threshold is clamped to the range `[0.0, 1.0]`.
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            resume: None,
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Replace the current resume with a new one.
    ///
    /// Any previously loaded resume is discarded.
    pub fn load_resume(&mut self, resume: Resume) {
        self.resume = Some(resume);
    }

    /// Returns `true` if a resume has been loaded.
    pub fn has_resume(&self) -> bool {
        self.resume.is_some()
    }

    /// Override the minimum score threshold.
    ///
    /// Values are clamped to the range `[0.0, 1.0]`.
    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Score a single job post against the loaded resume.
    ///
    /// Returns `None` if no resume is loaded.
    ///
    /// See the [module-level documentation](self) for the scoring breakdown.
    pub fn score(&self, job: &JobPost) -> Option<MatchResult> {
        let resume = self.resume.as_ref()?;

        let job_text = build_job_text(job);
        let job_lower = job_text.to_lowercase();

        // ── Skill matching ────────────────────────────────────────────────
        let mut matched_skills = Vec::new();
        let mut missing_skills = Vec::new();

        for skill in &resume.skills {
            let skill_lower = skill.to_lowercase();
            if job_lower.contains(&skill_lower) || fuzzy_match(&skill_lower, &job_lower) {
                matched_skills.push(skill.clone());
            } else {
                missing_skills.push(skill.clone());
            }
        }

        // ── Keyword matching ──────────────────────────────────────────────
        let mut matched_keywords = Vec::new();
        let all_keywords: Vec<String> = resume
            .keywords
            .iter()
            .chain(resume.role_titles.iter())
            .cloned()
            .collect();

        for kw in &all_keywords {
            let kw_lower = kw.to_lowercase();
            if job_lower.contains(&kw_lower) || fuzzy_match(&kw_lower, &job_lower) {
                matched_keywords.push(kw.clone());
            }
        }

        // ── Compute score ─────────────────────────────────────────────────
        let score = compute_score(
            &matched_skills,
            &resume.skills,
            &matched_keywords,
            &all_keywords,
            job,
            resume,
        );

        Some(MatchResult {
            job: job.clone(),
            score,
            matched_skills: dedup_ordered(matched_skills),
            matched_keywords: dedup_ordered(matched_keywords),
            missing_skills: dedup_ordered(missing_skills),
        })
    }

    /// Score all jobs and return results sorted by score (descending).
    ///
    /// Results with a score below `self.threshold` are filtered out.
    pub fn score_all(&self, jobs: &[JobPost]) -> Vec<MatchResult> {
        let mut results: Vec<MatchResult> = jobs
            .iter()
            .filter_map(|j| self.score(j))
            .filter(|r| r.score >= self.threshold)
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Borrow the loaded resume, if any.
    pub fn resume(&self) -> Option<&Resume> {
        self.resume.as_ref()
    }
}

// ─── Scoring Logic ────────────────────────────────────────────────────────────

/// Compute a composite match score (0.0 – 1.0) from skill overlaps, keyword
/// overlaps, title alignment, location preference, and job-type preference.
///
/// Each component is weighted (see the [module docs](self) for the breakdown)
/// and the final value is clamped to `[0.0, 1.0]`.
fn compute_score(
    matched_skills: &[String],
    all_skills: &[String],
    matched_keywords: &[String],
    all_keywords: &[String],
    job: &JobPost,
    resume: &Resume,
) -> f64 {
    if all_skills.is_empty() && all_keywords.is_empty() {
        return 0.5; // neutral score when no resume data to match against
    }

    let mut score = 0.0;

    // Skill match ratio (weighted heavily)
    let skill_ratio = if all_skills.is_empty() {
        0.0
    } else {
        matched_skills.len() as f64 / all_skills.len() as f64
    };
    score += skill_ratio * 0.50;

    // Keyword / role title match ratio
    let kw_ratio = if all_keywords.is_empty() {
        0.0
    } else {
        matched_keywords.len() as f64 / all_keywords.len() as f64
    };
    score += kw_ratio * 0.25;

    // Title bonus: boost if the job title contains any of the resume role titles
    let title_lower = job.title.to_lowercase();
    let title_match = resume.role_titles.iter().any(|r| {
        let rl = r.to_lowercase();
        title_lower.contains(&rl) || fuzzy_match(&rl, &title_lower)
    });
    if title_match {
        score += 0.10;
    }

    // Location bonus
    if let (Some(pref_loc), Some(job_loc)) = (&resume.preferred_location, &job.location) {
        let pl = pref_loc.to_lowercase();
        let jl = job_loc.to_lowercase();
        if pl.contains(&jl) || jl.contains(&pl) || fuzzy_match(&pl, &jl) {
            score += 0.10;
        }
    }

    // Job type bonus
    if let (Some(pref_type), Some(job_type)) = (&resume.preferred_job_type, &job.job_type) {
        let pt = pref_type.to_lowercase();
        let jt = job_type.to_lowercase();
        if pt == jt || jt.contains(&pt) || pt.contains(&jt) {
            score += 0.05;
        }
    }

    score.clamp(0.0, 1.0)
}

// ─── Text Helpers ─────────────────────────────────────────────────────────────

/// Concatenate all meaningful fields of a [`JobPost`] into a single
/// lower-case-friendly search blob.
///
/// Combines title, description, company, location, salary, job type, and tags
/// into one space-separated string so that keyword matching can search across
/// all fields at once.
fn build_job_text(job: &JobPost) -> String {
    let mut parts = vec![
        job.title.clone(),
        job.description.clone(),
    ];
    if let Some(c) = &job.company {
        parts.push(c.clone());
    }
    if let Some(l) = &job.location {
        parts.push(l.clone());
    }
    if let Some(s) = &job.salary {
        parts.push(s.clone());
    }
    if let Some(jt) = &job.job_type {
        parts.push(jt.clone());
    }
    parts.extend(job.tags.clone());
    parts.join(" ")
}

/// Check whether a short keyword approximately matches any word in a larger
/// text body using the [Jaro-Winkler distance].
///
/// Returns `true` if any whitespace-delimited word in `text` has a
/// Jaro-Winkler similarity of at least `0.85` to `keyword`. Words shorter
/// than 3 characters are ignored.
///
/// [Jaro-Winkler distance]: https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance
fn fuzzy_match(keyword: &str, text: &str) -> bool {
    let threshold = 0.85;
    let keyword = keyword.trim();
    if keyword.is_empty() || keyword.len() < 3 {
        return false;
    }

    text.split_whitespace()
        .any(|word| {
            let w = word.trim_matches(|c: char| !c.is_alphanumeric());
            if w.is_empty() {
                return false;
            }
            jaro_winkler(keyword, &w.to_lowercase()) >= threshold
        })
}

/// Remove duplicates from a vector while preserving insertion order.
///
/// Uses a [`HashSet`] for O(1) lookups. The first occurrence of each element
/// is kept; subsequent duplicates are dropped.
fn dedup_ordered<T: std::hash::Hash + Eq + Clone>(items: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::JobSource;
    use chrono::Utc;

    /// Build a minimal [`JobPost`] for use in tests.
    fn make_job(title: &str, desc: &str) -> JobPost {
        JobPost {
            id: "test".into(),
            title: title.into(),
            company: None,
            location: None,
            description: desc.into(),
            url: "https://example.com/job".into(),
            source: JobSource::Indeed,
            posted_at: None,
            crawled_at: Utc::now(),
            salary: None,
            job_type: None,
            tags: vec![],
        }
    }

    /// Build a minimal [`Resume`] from skills and role-title slices.
    fn make_resume(skills: &[&str], roles: &[&str]) -> Resume {
        Resume {
            skills: skills.iter().map(|&s| s.to_string()).collect(),
            experience_years: None,
            role_titles: roles.iter().map(|&r| r.to_string()).collect(),
            keywords: vec![],
            preferred_location: None,
            preferred_job_type: None,
            min_salary: None,
        }
    }

    /// Verify that a matcher can be created and a resume loaded without
    /// panicking.
    #[test]
    fn test_exact_skill_match() {
        let mut matcher = Matcher::new();
        let resume = make_resume(&["rust", "python", "docker"], &["engineer"]);
        matcher.load_resume(resume);
    }

    /// Verify that a job mentioning matching skills gets a score > 0.5.
    #[test]
    fn test_score_rust_job() {
        let mut matcher = Matcher::new();
        matcher.load_resume(make_resume(&["rust", "python"], &["engineer"]));
        let job = make_job("Rust Engineer", "We need a Rust engineer with Python experience");
        let result = matcher.score(&job).unwrap();
        assert!(result.score > 0.5);
        assert!(result.matched_skills.contains(&"rust".to_string()));
    }

    /// Verify that scoring returns `None` when no resume is loaded.
    #[test]
    fn test_no_resume_returns_none() {
        let matcher = Matcher::new();
        let job = make_job("Engineer", "some job");
        assert!(matcher.score(&job).is_none());
    }

    /// Verify that only present skills are marked as matched.
    #[test]
    fn test_partial_match() {
        let mut matcher = Matcher::new();
        matcher.load_resume(make_resume(
            &["rust", "kubernetes", "react"],
            &["engineer"],
        ));
        let job = make_job("Frontend Engineer", "React developer position");
        let result = matcher.score(&job).unwrap();
        // Only "react" and "engineer" should match
        assert!(result.matched_skills.contains(&"react".to_string()));
        assert!(!result.matched_skills.contains(&"rust".to_string()));
    }
}
