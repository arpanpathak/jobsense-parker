pub mod scoring;

use std::collections::HashSet;

use crate::models::{JobPost, MatchResult, Resume};
use scoring::{build_job_text, compute_score, fuzzy_match};

pub struct Matcher {
    resume: Option<Resume>,
    threshold: f64,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            resume: None,
            threshold: 0.15,
        }
    }

    pub fn load_resume(&mut self, resume: Resume) {
        self.resume = Some(resume);
    }

    pub fn has_resume(&self) -> bool {
        self.resume.is_some()
    }

    pub fn resume(&self) -> Option<&Resume> {
        self.resume.as_ref()
    }

    pub fn score(&self, job: &JobPost) -> Option<MatchResult> {
        let resume = self.resume.as_ref()?;

        let job_text = build_job_text(job);
        let job_lower = job_text.to_lowercase();

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

    pub fn score_all(&self, jobs: &[JobPost]) -> Vec<MatchResult> {
        let mut results: Vec<MatchResult> = jobs
            .iter()
            .filter_map(|j| self.score(j))
            .filter(|r| r.score >= self.threshold)
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

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
            source: JobSource::Indeed,
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
        }
    }

    #[test]
    fn test_exact_skill_match() {
        let mut matcher = Matcher::new();
        let resume = make_resume(&["rust", "python", "docker"], &["engineer"]);
        matcher.load_resume(resume);
    }

    #[test]
    fn test_score_rust_job() {
        let mut matcher = Matcher::new();
        matcher.load_resume(make_resume(&["rust", "python"], &["engineer"]));
        let job = make_job("Rust Engineer", "We need a Rust engineer with Python experience");
        let result = matcher.score(&job).unwrap();
        assert!(result.score > 0.5);
        assert!(result.matched_skills.contains(&"rust".to_string()));
    }

    #[test]
    fn test_no_resume_returns_none() {
        let matcher = Matcher::new();
        let job = make_job("Engineer", "some job");
        assert!(matcher.score(&job).is_none());
    }

    #[test]
    fn test_partial_match() {
        let mut matcher = Matcher::new();
        matcher.load_resume(make_resume(
            &["rust", "kubernetes", "react"],
            &["engineer"],
        ));
        let job = make_job("Frontend Engineer", "React developer position");
        let result = matcher.score(&job).unwrap();
        assert!(result.matched_skills.contains(&"react".to_string()));
        assert!(!result.matched_skills.contains(&"rust".to_string()));
    }
}
