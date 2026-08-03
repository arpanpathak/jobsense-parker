//! # Scoring Algorithm
//!
//! Computes a compatibility score between a [`Resume`] and a [`JobPost`].
//! The score is a float between 0.0 (no match) and 1.0 (perfect match).
//!
//! ## Score Breakdown
//!
//! | Component | Weight | How it works |
//! |-----------|--------|-------------|
//! | **Title skill match** | 35% | Any skill from your resume appearing in the job TITLE ("Senior Rust Engineer" = full title credit) |
//! | **Skill coverage** | 30% | How many of your skills appear in the job text (saturates at min(10, total skills) — broad resumes aren't penalised for knowing more) |
//! | **Keyword ratio** | 15% | Fraction of resume keywords found in the job text |
//! | **Role-title match** | 10% | Job title contains one of your role titles ("software engineer", "developer") |
//! | **Location match** | 5% | Job location aligns with preferred location |
//! | **Job-type match** | 5% | Job type matches preferred type |
//!
//! ## Matching
//!
//! Skills are matched with **word boundaries** (so `"go"` doesn't match
//! `"google"` and `"rust"` doesn't match `"trust"`) and **aliases** (`k8s` ≈
//! `kubernetes`, `golang` ≈ `go`, `cpp` ≈ `c++`, `js` ≈ `javascript`).

use crate::models::{JobPost, Resume};
use regex::Regex;
use std::sync::OnceLock;
use strsim::jaro_winkler;

/// Token aliases expanded before matching. Job text and skill names are both
/// normalised so `"k8s"` counts as `"kubernetes"`, `"golang"` as `"go"`, etc.
const ALIASES: &[(&str, &str)] = &[
    ("k8s", "kubernetes"),
    ("kube", "kubernetes"),
    ("golang", "go"),
    ("cpp", "c++"),
    ("js", "javascript"),
    ("ts", "typescript"),
    ("node.js", "node"),
    ("nodejs", "node"),
    ("react.js", "react"),
    ("reactjs", "react"),
    ("postgres", "postgresql"),
    ("gcp", "google cloud"),
];

/// Compiled alias regexes, cached once. Compiling per call was the hot path:
/// `normalize_text` is invoked per-skill (100+ times per job) and each
/// `Regex::new` costs ~0.3ms.
fn alias_regexes() -> &'static [(Regex, &'static str)] {
    static REGEXES: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    REGEXES.get_or_init(|| {
        ALIASES
            .iter()
            .filter_map(|(from, to)| {
                Regex::new(&format!(r"\b{}\b", regex::escape(from)))
                    .ok()
                    .map(|re| (re, *to))
            })
            .collect()
    })
}

/// Expand aliases and lowercase a skill name or job text for matching.
pub fn normalize_text(text: &str) -> String {
    let mut out = text.to_lowercase();
    for (re, to) in alias_regexes() {
        out = re.replace_all(&out, *to).to_string();
    }
    out
}

/// Does a (already-normalised) skill appear in normalised job text?
///
/// Multi-word, long, or slash-containing skills ("machine learning",
/// "kubernetes", "http/2") are safe as substring matches. Short names and
/// names with special chars ("go", "c", "c++", "s3", "c#") use token matching
/// so `"go"` doesn't match `"google"` and `"rust"` doesn't match `"trust"`.
pub fn skill_in_normalized(skill: &str, normalized_text: &str) -> bool {
    let s = normalize_text(skill);
    if s.contains(' ') || s.len() >= 5 || s.contains('/') {
        return normalized_text.contains(&s);
    }
    normalized_text
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '+' && c != '#' && c != '.')
        .any(|tok| tok == s || (s.contains(['+', '#', '.']) && tok.starts_with(&s)))
}

/// Compute a composite match score between 0.0 and 1.0.
///
/// # Scoring breakdown
///
/// - **Title skill match** (35%): skills found in the job TITLE, saturating
///   at 2. A skill in the title is the strongest possible signal.
/// - **Skill coverage** (30%): fraction of resume skills found in the job
///   description, saturating at 12 matches so a broad resume isn't penalised.
/// - **Keyword ratio** (15%): fraction of resume keywords found in job text.
/// - **Role-title match** (10%): bonus if job title contains a role from resume.
/// - **Location match** (5%): bonus if job location aligns with preferred location.
/// - **Job-type match** (5%): bonus if job type matches preferred type.
///
/// # Why saturation?
///
/// The old formula divided by ALL resume skills. With 100+ real skills on a
/// broad engineer's resume, even a perfect match scored ~15%. Saturation makes
/// the score reflect "how many of MY core skills does this job need", not
/// "what fraction of my entire career does this job cover".
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
    let title_lower = normalize_text(&job.title.to_lowercase());

    // ── Title skill match (35%) ──────────────────────────────────────────
    // Skills in the job title are the strongest possible signal: if ANY of
    // your skills is literally in the title ("Rust Engineer", "Python
    // Developer"), the job is almost certainly a match.
    if !all_skills.is_empty() {
        let title_skills = all_skills
            .iter()
            .filter(|s| skill_in_normalized(s, &title_lower))
            .count();
        if title_skills >= 1 {
            score += 0.35;
        }
    }

    // ── Skill coverage (30%) ────────────────────────────────────────────
    // How many resume skills appear in the job text. Saturates at
    // min(10, total_skills) so a broad 100+ skill resume isn't penalised for
    // knowing more, while a 2-skill resume gets full credit when fully matched.
    let cov_denom = all_skills.len().min(10) as f64;
    if cov_denom > 0.0 {
        let coverage = (matched_skills.len() as f64 / cov_denom).min(1.0);
        score += coverage * 0.30;
    }

    // ── Keyword ratio (15%) ─────────────────────────────────────────────
    if !all_keywords.is_empty() {
        let kw_ratio = matched_keywords.len() as f64 / all_keywords.len() as f64;
        score += kw_ratio * 0.15;
    }

    // ── Role-title match (10%) ──────────────────────────────────────────
    let title_match = resume.role_titles.iter().any(|r| {
        let rl = r.to_lowercase();
        title_lower.contains(&rl) || fuzzy_match(&rl, &title_lower)
    });
    if title_match {
        score += 0.10;
    }

    // ── Location match (5%) ─────────────────────────────────────────────
    if let (Some(pref_loc), Some(job_loc)) = (&resume.preferred_location, &job.location) {
        let pl = pref_loc.to_lowercase();
        let jl = job_loc.to_lowercase();
        if pl.contains(&jl) || jl.contains(&pl) || fuzzy_match(&pl, &jl) {
            score += 0.05;
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
/// dump platform-level tag clouds onto every job listing.
///
/// Also strips marker sections like \"Tags:\" from descriptions so platform
/// tag dumps don't inflate skill matches. NOTE: \"Requirements:\" is NOT
/// stripped — that's where the actual required skills live.
pub fn build_job_text(job: &JobPost) -> String {
    let desc = strip_tag_cloud(&job.description);
    let mut parts = vec![job.title.clone(), desc];
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

/// Strip tag-cloud sections from job descriptions.
///
/// Some job boards append a comma-separated tag cloud of every keyword.
/// These inflate skill matching scores. We detect common tag-dump markers
/// and truncate before them. Real content markers like \"Requirements:\",
/// \"Skills:\" and \"Nice to have:\" are kept — they contain the skills we
/// actually want to match against.
fn strip_tag_cloud(text: &str) -> String {
    let lower = text.to_lowercase();
    let markers = ["tags:", "technologies:", "tech stack:"];

    let mut earliest = None;
    for marker in &markers {
        if let Some(pos) = lower.find(marker) {
            match earliest {
                None => earliest = Some(pos),
                Some(current) if pos < current => earliest = Some(pos),
                _ => {}
            }
        }
    }

    match earliest {
        Some(pos) => text[..pos].trim().to_string(),
        None => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_boundaries_prevent_false_positives() {
        assert!(!skill_in_normalized("go", &normalize_text("We use Google and strong engineering")));
        assert!(!skill_in_normalized("rust", &normalize_text("We trust the compiler")));
        assert!(skill_in_normalized("go", &normalize_text("Written in Go and Rust")));
        assert!(skill_in_normalized("c++", &normalize_text("C++17 systems programming")));
    }

    #[test]
    fn aliases_are_expanded() {
        assert!(skill_in_normalized("kubernetes", &normalize_text("k8s at scale")));
        assert!(skill_in_normalized("go", &normalize_text("golang backend")));
        assert!(skill_in_normalized("c++", &normalize_text("modern cpp")));
        assert!(skill_in_normalized("javascript", &normalize_text("node.js and js")));
        assert!(skill_in_normalized("postgresql", &normalize_text("postgres db")));
    }

    #[test]
    fn multiword_skills_match_phrases() {
        assert!(skill_in_normalized("machine learning", &normalize_text("machine learning engineer")));
        assert!(skill_in_normalized("distributed systems", &normalize_text("building distributed systems")));
    }
}
