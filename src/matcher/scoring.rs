//! # Scoring Algorithm
//!
//! Computes a compatibility score between a [`Resume`] and a [`JobPost`]
//! using weighted fuzzy matching. The score is a float between 0.0 (no match)
//! and 1.0 (perfect match).
//!
//! ## Score Breakdown
//!
//! | Component | Weight | How it works |
//! |-----------|--------|-------------|
//! | **Skill ratio** | 50% | `matched_skills / total_skills` — what fraction of your skills appear in the job description? |
//! | **Keyword ratio** | 25% | `matched_keywords / total_keywords` — broad keyword overlap |
//! | **Role-title match** | 10% | Does the job title contain one of your role titles (e.g. "engineer")? Uses Jaro-Winkler fuzzy match. |
//! | **Location match** | 10% | Does the job location contain your preferred location (or vice versa)? Fuzzy matched. |
//! | **Job-type match** | 5% | Does the job type match your preferred type (e.g. "remote", "full-time")? |
//!
//! ## Fuzzy Matching
//!
//! Where exact substring matching fails, we fall back to
//! [Jaro-Winkler similarity](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance)
//! with a threshold of 0.85. This catches typos and small variations:
//!
//! | Input | Match | Score |
//! |-------|-------|-------|
//! | `"kubernetes"` | `"k8s"` | High (fuzzy) |
//! | `"typescript"` | `"TypeScript"` | Exact (case-insensitive) |
//! | `"javascript"` | `"js"` | No (too short for fuzzy) |

use crate::models::{JobPost, Resume};
use strsim::jaro_winkler;

/// Compute a composite match score between 0.0 and 1.0.
///
/// # Scoring breakdown
///
/// - **Skill ratio** (50%): fraction of resume skills found in job text
/// - **Keyword ratio** (25%): fraction of resume keywords found in job text
/// - **Role-title match** (10%): bonus if job title contains a role from resume
/// - **Location match** (10%): bonus if job location aligns with preferred location
/// - **Job-type match** (5%): bonus if job type matches preferred type
///
/// # Example
///
/// If your resume has skills `["rust", "python", "docker"]` and a job
/// description mentions "rust" and "docker":
///
/// ```text
/// skill_ratio  = 2/3 × 0.50 = 0.333
/// keyword_ratio = ...        = 0.125
/// title_match  = bonus       = 0.100  (if job title says "Engineer")
///                             ─────
///                             0.558  (55.8% match)
/// ```
pub fn compute_score(
    matched_skills: &[String],
    all_skills: &[String],
    matched_keywords: &[String],
    all_keywords: &[String],
    job: &JobPost,
    resume: &Resume,
) -> f64 {
    if all_skills.is_empty() && all_keywords.is_empty() {
        return 0.5;
    }

    let mut score = 0.0;

    // ── Skill ratio (50%) ───────────────────────────────────────────────
    let skill_ratio = if all_skills.is_empty() {
        0.0
    } else {
        matched_skills.len() as f64 / all_skills.len() as f64
    };
    score += skill_ratio * 0.50;

    // ── Keyword ratio (25%) ─────────────────────────────────────────────
    let kw_ratio = if all_keywords.is_empty() {
        0.0
    } else {
        matched_keywords.len() as f64 / all_keywords.len() as f64
    };
    score += kw_ratio * 0.25;

    // ── Role-title match (10%) ──────────────────────────────────────────
    let title_lower = job.title.to_lowercase();
    let title_match = resume.role_titles.iter().any(|r| {
        let rl = r.to_lowercase();
        title_lower.contains(&rl) || fuzzy_match(&rl, &title_lower)
    });
    if title_match {
        score += 0.10;
    }

    // ── Location match (10%) ────────────────────────────────────────────
    if let (Some(pref_loc), Some(job_loc)) = (&resume.preferred_location, &job.location) {
        let pl = pref_loc.to_lowercase();
        let jl = job_loc.to_lowercase();
        if pl.contains(&jl) || jl.contains(&pl) || fuzzy_match(&pl, &jl) {
            score += 0.10;
        }
    }

    // ── Job-type match (5%) ─────────────────────────────────────────────
    if let (Some(pref_type), Some(job_type)) = (&resume.preferred_job_type, &job.job_type) {
        let pt = pref_type.to_lowercase();
        let jt = job_type.to_lowercase();
        if pt == jt || jt.contains(&pt) || pt.contains(&jt) {
            score += 0.05;
        }
    }

    score.clamp(0.0, 1.0)
}

/// Check whether `keyword` approximately matches any word in `text`
/// using the [Jaro-Winkler distance](https://en.wikipedia.org/wiki/Jaro%E2%80%93Winkler_distance)
/// with a threshold of 0.85.
///
/// # How it works
///
/// 1. Splits `text` into whitespace-separated words
/// 2. Strips non-alphanumeric characters from each word
/// 3. Compares each word (case-insensitive) against `keyword` using
///    [`jaro_winkler`] from the `strsim` crate
/// 4. Returns `true` if any word scores ≥ 0.85
///
/// # Examples
///
/// ```ignore
/// assert!(fuzzy_match("kubernetes", "We use k8s at scale"));
/// assert!(!fuzzy_match("rust", "We use ruby and java"));
/// ```
///
/// # Limitations
///
/// - Requires keywords to be at least 3 characters long (avoids false
///   positives on short strings like "js" or "go")
/// - Threshold of 0.85 is fairly strict — catches typos but not synonyms
pub fn fuzzy_match(keyword: &str, text: &str) -> bool {
    let threshold = 0.85;
    let keyword = keyword.trim();
    if keyword.is_empty() || keyword.len() < 3 {
        return false;
    }

    text.split_whitespace().any(|word| {
        let w = word.trim_matches(|c: char| !c.is_alphanumeric());
        if w.is_empty() {
            return false;
        }
        jaro_winkler(keyword, &w.to_lowercase()) >= threshold
    })
}

/// Concatenate relevant fields of a job post into a single searchable string.
///
/// Combines `title`, `description`, `company`, `location`, `salary`,
/// and `job_type` into one space-separated string for skill matching.
///
/// **`tags` are deliberately excluded** because job boards like Remote OK
/// dump platform-level tag clouds onto every job listing. A "Senior Vice
/// President" role can match "ai, angular, c#, python, react, rust..." purely
/// from tags, making skill matching meaningless.
///
/// # Example output
///
/// ```text
/// "Senior Rust Engineer We are looking for a Rust engineer... Stripe San Francisco $200k full-time"
/// ```
pub fn build_job_text(job: &JobPost) -> String {
    let mut parts = vec![job.title.clone(), job.description.clone()];
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
    parts.join(" ").trim().to_string()
}
