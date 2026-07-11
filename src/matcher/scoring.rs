//! Scoring logic used by the [`Matcher`](super::Matcher) to compute match scores.
//!
//! The final score is a weighted combination of skill matches, keyword matches,
//! title overlap, location match, and job-type match.

use crate::models::{JobPost, Resume};
use strsim::jaro_winkler;

/// Compute a composite match score between 0.0 and 1.0.
///
/// Scoring weights:
/// * Skill ratio: 50%
/// * Keyword ratio: 25%
/// * Role-title match: 10%
/// * Location match: 10%
/// * Job-type match: 5%
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

    let skill_ratio = if all_skills.is_empty() {
        0.0
    } else {
        matched_skills.len() as f64 / all_skills.len() as f64
    };
    score += skill_ratio * 0.50;

    let kw_ratio = if all_keywords.is_empty() {
        0.0
    } else {
        matched_keywords.len() as f64 / all_keywords.len() as f64
    };
    score += kw_ratio * 0.25;

    let title_lower = job.title.to_lowercase();
    let title_match = resume.role_titles.iter().any(|r| {
        let rl = r.to_lowercase();
        title_lower.contains(&rl) || fuzzy_match(&rl, &title_lower)
    });
    if title_match {
        score += 0.10;
    }

    if let (Some(pref_loc), Some(job_loc)) = (&resume.preferred_location, &job.location) {
        let pl = pref_loc.to_lowercase();
        let jl = job_loc.to_lowercase();
        if pl.contains(&jl) || jl.contains(&pl) || fuzzy_match(&pl, &jl) {
            score += 0.10;
        }
    }

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
/// using the Jaro-Winkler distance (threshold: 0.85).
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

/// Concatenate all relevant fields of a job post into a single searchable string.
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
    parts.extend(job.tags.clone());
    parts.join(" ")
}
