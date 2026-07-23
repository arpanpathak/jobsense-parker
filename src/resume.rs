//! # Resume Intelligence Engine
//!
//! Extracts structured intelligence from resume text using:
//!
//! 1. **Comprehensive tech-skill dictionary** (500+ known technologies)
//! 2. **Context-aware extraction** (patterns like "experience with X", "proficient in Y")
//! 3. **Role/seniority inference** from title patterns
//! 4. **Domain detection** (backend, frontend, DevOps, data science, etc.)
//! 5. **Education extraction** (degrees, fields, universities)
//! 6. **Certification detection**
//!
//! This replaces the previous regex-only approach that produced noisy keywords.

use regex::Regex;
use serde::{Deserialize, Serialize};

// ─── Comprehensive Tech Skill Dictionary ──────────────────────────────────────
//
// Organized by domain so we can infer the candidate's area of expertise.
// Each entry maps (lowercase name) → domain category.

/// Categories of skills for domain inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillDomain {
    Language,
    Framework,
    Frontend,
    Backend,
    Database,
    CloudDevOps,
    DataMl,
    Mobile,
    Tools,
    Platform,
    Protocol,
    Concept,
    Other,
}

impl std::fmt::Display for SkillDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Language => write!(f, "Languages"),
            Self::Framework => write!(f, "Frameworks"),
            Self::Frontend => write!(f, "Frontend"),
            Self::Backend => write!(f, "Backend"),
            Self::Database => write!(f, "Databases"),
            Self::CloudDevOps => write!(f, "Cloud/DevOps"),
            Self::DataMl => write!(f, "Data/ML"),
            Self::Mobile => write!(f, "Mobile"),
            Self::Tools => write!(f, "Tools"),
            Self::Platform => write!(f, "Platforms"),
            Self::Protocol => write!(f, "Protocols"),
            Self::Concept => write!(f, "Concepts"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// A known tech skill with its domain classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownSkill {
    pub name: String,
    pub domain: SkillDomain,
}

/// Context-weighted skill extractor.
///
/// Instead of matching against a hardcoded dictionary (which is fragile and
/// misses anything new), this extracts tech skills from the resume text itself
/// using context cues:
///
/// 1. **Context phrases** — terms near "experience with", "proficient in", etc.
/// 2. **Section headers** — terms in "Technologies:", "Skills:" sections
/// 3. **Capitalisation** — capitalized proper nouns (likely tech names)
/// 4. **Special chars** — terms with +, #, ., digits (C++, Node.js, etc.)
/// 5. **Frequency** — terms appearing multiple times in the resume
///
/// No hardcoded list. No regex maintenance. It reads YOUR resume and figures
/// out what's relevant.
pub struct SkillDictionary {
    /// Context phrases that indicate the next word(s) are likely a skill.
    context_phrases: Vec<&'static str>,
    /// Section header markers that introduce a list of skills.
    section_markers: Vec<&'static str>,
    /// Common English words that should never be extracted as skills.
    stop_words: Vec<&'static str>,
}

impl SkillDictionary {
    /// Create a new extractor with context patterns.
    pub fn new() -> Self {
        Self {
            context_phrases: vec![
                "experience with", "experience in", "experience using",
                "proficient in", "proficient with",
                "knowledge of", "working knowledge of",
                "familiar with", "familiarity with",
                "expertise in", "expertise with",
                "skilled in", "skilled at", "skilled with",
                "strong in", "strong knowledge of",
                "background in", "hands-on experience with",
                "exposure to", "worked with", "worked on",
                "built with", "built using", "developed with",
                "developed using", "programming in", "programming with",
                "fluent in", "competent in",
                "deep experience in", "extensive experience in",
                "solid experience with", "practical experience with",
                "focus on", "specialized in", "specialize in",
                "passionate about",
            ],
            section_markers: vec![
                "technologies:", "technology:", "tech stack:", "tech:",
                "tools:", "tool:", "languages:", "language:",
                "frameworks:", "framework:", "platforms:", "platform:",
                "skills:", "skill:", "expertise:", "competencies:",
                "core competencies:", "technical skills:",
                "programming languages:", "environments:",
            ],
            stop_words: vec![
                "the", "and", "for", "are", "but", "not", "you", "all",
                "can", "was", "one", "our", "out", "has", "have", "been",
                "some", "same", "its", "than", "them", "into", "two",
                "more", "these", "like", "such", "that", "this", "with",
                "from", "your", "which", "each", "will", "about", "between",
                "very", "just", "their", "would", "could", "should",
                "then", "there", "where", "while", "because", "before",
                "does", "doing", "done", "much", "many", "most", "must",
                "need", "take", "make", "well", "also", "back", "still",
                "get", "use", "used", "using", "way", "new", "first",
                "last", "own", "see", "may", "though", "every", "good",
                "great", "best", "year", "years", "experience", "work",
                "team", "project", "projects", "including", "including",
                "various", "multiple", "different", "wide", "range",
                "etc", "etc.",
            ],
        }
    }

    /// Extract tech skills from text using context-weighted analysis.
    ///
    /// Strategy: scan the text for signals that a term is a tech skill,
    /// score each candidate, return the highest-confidence ones.
    pub fn find_skills(&self, text: &str) -> Vec<KnownSkill> {
        let lower = text.to_lowercase();
        let mut candidates: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

        // ── Signal 1: Context phrases (+3 confidence) ──────────────
        // "experience with Rust" → "Rust" is almost certainly a skill
        for phrase in &self.context_phrases {
            // Find each occurrence of the phrase, then grab the next 1-3 words
            let mut search_from = 0;
            while let Some(pos) = lower[search_from..].find(phrase) {
                let abs_pos = search_from + pos + phrase.len();
                let after = &lower[abs_pos..];

                // Get the next 1-3 words
                let words: Vec<&str> = after.split_whitespace()
                    .filter(|w| w.len() >= 2)
                    .take(3)
                    .collect();

                for word in &words {
                    let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '+' && c != '#' && c != '.');
                    if clean.len() >= 2 && !self.stop_words.iter().any(|s| *s == clean) {
                        *candidates.entry(clean.to_string()).or_insert(0) += 3;
                    }
                }

                // Also check for multi-word skill
                if words.len() >= 2 {
                    let bigram = format!("{} {}", words[0], words[1]);
                    let clean_bigram: String = bigram.chars()
                        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '+' || *c == '#' || *c == '.')
                        .collect();
                    if clean_bigram.len() >= 3 {
                        *candidates.entry(clean_bigram.to_lowercase()).or_insert(0) += 3;
                    }
                }

                search_from = abs_pos + 1;
                if search_from >= lower.len() { break; }
            }
        }

        // ── Signal 2: Section markers (+2 confidence) ──────────────
        // "Technologies: Rust, Python, Docker" → list items are skills
        for marker in &self.section_markers {
            if let Some(pos) = lower.find(marker) {
                let after = &lower[pos + marker.len()..];
                // Split by comma, newline, semicolon
                for item in after.split(|c: char| c == ',' || c == '\n' || c == ';' || c == '|') {
                    let clean = item.trim().trim_matches(|c: char| !c.is_alphanumeric() && c != '+' && c != '#' && c != '.');
                    if clean.len() >= 2 && !self.stop_words.iter().any(|s| *s == clean) {
                        *candidates.entry(clean.to_string()).or_insert(0) += 2;
                    }
                }
            }
        }

        // ── Signal 3: Capitalisation pattern (+2 confidence) ───────
        // Words that start with uppercase or have mixed case (like "TypeScript")
        // are often proper nouns = tech names
        for word in text.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '+' && c != '#' && c != '.');

            if clean.len() < 2 || self.stop_words.iter().any(|s| *s == clean.to_lowercase().as_str()) {
                continue;
            }

            let has_upper = clean.chars().any(|c| c.is_uppercase());
            let has_lower = clean.chars().any(|c| c.is_lowercase());
            let has_special = clean.contains(|c: char| c == '+' || c == '#' || c == '.' || c == '-');
            let has_digit = clean.chars().any(|c| c.is_ascii_digit());

            // Capitalized or mixed case → likely a tech term
            if has_upper && has_lower && clean.len() >= 3 {
                *candidates.entry(clean.to_lowercase()).or_insert(0) += 2;
            }

            // Has special chars (C++, .NET, Node.js) → likely tech
            if has_special && clean.len() >= 2 {
                *candidates.entry(clean.to_lowercase()).or_insert(0) += 2;
            }

            // Has digits (Python3, Webpack5) → likely tech
            if has_digit && clean.len() >= 2 {
                *candidates.entry(clean.to_lowercase()).or_insert(0) += 1;
            }
        }

        // ── Signal 4: Uppercase acronyms (AWS, GCP, API) ───────────
        for word in text.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.len() >= 2 && clean.len() <= 6
                && clean.chars().all(|c| c.is_uppercase() || c.is_ascii_digit())
            {
                *candidates.entry(clean.to_lowercase()).or_insert(0) += 1;
            }
        }

        // ── Signal 5: All-caps short words (Rust, Go, Vue) ────────
        for word in text.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.len() >= 2 && clean.len() <= 8
                && clean.chars().all(|c| c.is_uppercase())
            {
                *candidates.entry(clean.to_lowercase()).or_insert(0) += 1;
            }
        }

        // ── Filter and sort results ────────────────────────────────
        // Remove candidates with very low confidence
        candidates.retain(|_, score| *score >= 2);

        // Sort by score descending
        let mut sorted: Vec<(String, i32)> = candidates.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        // Return top N as KnownSkills
        sorted.into_iter()
            .take(80)
            .map(|(name, _)| KnownSkill {
                name,
                domain: SkillDomain::Other,
            })
            .collect()
    }

    /// Count how many skills from a given set appear in text.
    pub fn count_skills_in_text(&self, skills: &[String], text: &str) -> Vec<String> {
        let lower = text.to_lowercase();
        skills
            .iter()
            .filter(|s| lower.contains(&s.to_lowercase()))
            .cloned()
            .collect()
    }

    /// Return all skill names (from context patterns, not a fixed list).
    pub fn all_skill_names(&self) -> Vec<String> {
        self.context_phrases.iter().map(|s| s.to_string()).collect()
    }
}

impl Default for SkillDictionary {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Extraction Results ──────────────────────────────────────────────────────

/// Enriched intelligence extracted from a resume.
///
/// Focus areas are derived dynamically from the resume text, not from
/// hardcoded skill→domain mappings. This means no more "Cloud/DevOps"
/// labels if your resume never says "DevOps".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeIntelligence {
    /// Known tech skills found in the resume (with domain classification).
    pub known_skills: Vec<KnownSkill>,
    /// Skills inferred from context patterns (not in the dictionary but likely tech).
    pub inferred_skills: Vec<String>,
    /// Role/career level (e.g., "Senior Software Engineer", "Data Scientist").
    pub role_titles: Vec<String>,
    /// Seniority level inferred from titles.
    pub seniority: Option<SeniorityLevel>,
    /// Years of professional experience.
    pub experience_years: Option<f32>,
    /// Preferred location.
    pub preferred_location: Option<String>,
    /// Preferred job type.
    pub preferred_job_type: Option<String>,
    /// Focus areas derived from the resume content — what you actually work on.
    /// Dynamically computed from job titles, frequent tech terms, and role context.
    /// No hardcoded categories.
    pub focus_areas: Vec<String>,
    /// Degrees extracted.
    pub education: Vec<Education>,
    /// Certifications extracted.
    pub certifications: Vec<String>,
    /// Meaningful keywords (tech terms + domain terms, filtered).
    pub keywords: Vec<String>,
}

/// Seniority level inferred from role titles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeniorityLevel {
    Intern,
    Junior,
    Mid,
    Senior,
    Staff,
    Principal,
    Lead,
    Director,
    Executive,
}

impl std::fmt::Display for SeniorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Intern => write!(f, "Intern"),
            Self::Junior => write!(f, "Junior"),
            Self::Mid => write!(f, "Mid-level"),
            Self::Senior => write!(f, "Senior"),
            Self::Staff => write!(f, "Staff"),
            Self::Principal => write!(f, "Principal"),
            Self::Lead => write!(f, "Lead"),
            Self::Director => write!(f, "Director"),
            Self::Executive => write!(f, "Executive"),
        }
    }
}

/// Education entry extracted from resume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    pub degree: Option<String>,
    pub field: Option<String>,
    pub institution: Option<String>,
}

impl std::fmt::Display for Education {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = [&self.degree, &self.field, &self.institution]
            .iter()
            .filter_map(|s| s.as_ref().filter(|s| !s.is_empty()))
            .cloned()
            .collect();
        if parts.is_empty() {
            write!(f, "(unknown)")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

// ─── Parser ──────────────────────────────────────────────────────────────────

/// Parse raw resume text into structured intelligence.
pub fn parse_resume(text: &str) -> ResumeIntelligence {
    let dict = SkillDictionary::new();
    let lower = text.to_lowercase();

    // 1. Find known tech skills (the dictionary-driven approach)
    let known_skills = dict.find_skills(text);

    // 2. Find context-inferred skills (patterns like "experience with X")
    let inferred_skills = extract_context_skills(&lower, &dict);

    // 3. Extract role titles
    let role_titles = extract_roles(&lower);

    // 4. Infer seniority
    let seniority = infer_seniority(&role_titles, &lower);

    // 5. Extract experience years
    let experience_years = extract_experience_years(&lower);

    // 6. Extract location
    let preferred_location = extract_location(&lower);

    // 7. Extract job type
    let preferred_job_type = extract_job_type(&lower);

    // 8. Derive focus areas dynamically from resume text
    // (no hardcoded skill→domain categories — reads what the resume says)
    let focus_areas = derive_focus_areas(text, &role_titles);

    // 9. Extract education
    let education = extract_education(&lower);

    // 10. Extract certifications
    let certifications = extract_certifications(&lower);

    // 11. Build meaningful keywords (filtered to be useful)
    let keywords = build_keywords(&known_skills, &role_titles, &inferred_skills, &lower);

    ResumeIntelligence {
        known_skills: known_skills
            .iter()
            .map(|ks| KnownSkill {
                name: ks.name.clone(),
                domain: ks.domain,
            })
            .collect(),
        inferred_skills,
        role_titles,
        seniority,
        experience_years,
        preferred_location,
        preferred_job_type,
        focus_areas,
        education,
        certifications,
        keywords,
    }
}

/// Extract skills from context patterns (for skills not in the dictionary).
fn extract_context_skills(text: &str, dict: &SkillDictionary) -> Vec<String> {
    let mut skills: Vec<String> = Vec::new();

    // Pattern: "experience with X", "proficient in X", "knowledge of X", etc.
    let ctx_re = Regex::new(
        r"(?i)(?:experience|proficient|skills?|knowledge|worked|familiar|expertise|strong|fluent|background|expert|using|including|exposure|hands.?on|practical|extensive|solid|deep|working)\s*(?:with|in|at|using|of|on|including)\s+([a-z][a-z+#.\d]+(?:\s+[a-z][a-z+#.\d]+)?)"
    ).unwrap();

    for cap in ctx_re.captures_iter(text) {
        let s = cap.get(1).unwrap().as_str().trim().to_string();
        if s.len() >= 2 && !dict.all_skill_names().contains(&s.to_lowercase()) {
            // Only include if it looks like a tech term (not a generic word)
            if looks_like_tech_term(&s) {
                skills.push(s);
            }
        }
    }

    // Pattern: "Technologies: X, Y, Z"
    let list_re = Regex::new(
        r"(?i)(?:technolog(y|ies)|tools?|tech\s*stack|languages?|frameworks?|platforms?|skills?|expertise)[:\s]+(.+)"
    ).unwrap();

    if let Some(caps) = list_re.captures(text) {
        let list_part = caps.get(2).unwrap().as_str();
        for item in list_part.split([',', ';', '|', '\n']).filter_map(|s| {
            let t = s.trim().trim_matches(|c: char| c == '.' || c == ' ').to_string();
            (t.len() >= 2 && !t.contains(char::is_whitespace)).then_some(t)
        }) {
            if !dict.all_skill_names().contains(&item.to_lowercase()) && looks_like_tech_term(&item) {
                skills.push(item);
            }
        }
    }

    skills.sort_unstable();
    skills.dedup();
    skills
}

/// Heuristic: does a term look like a tech skill (not a generic English word)?
fn looks_like_tech_term(term: &str) -> bool {
    let generic_tech_adjacent = [
        "development", "engineering", "software", "hardware", "programming",
        "coding", "implementation", "integration", "deployment", "testing",
        "analysis", "design", "architecture", "management", "operations",
        "administration", "support", "maintenance", "optimization", "automation",
        "configuration", "monitoring", "troubleshooting", "debugging",
        "framework", "library", "platform", "infrastructure", "pipeline",
        "workflow", "pipeline", "toolchain", "middleware", "frontend",
        "backend", "fullstack", "full-stack", "database", "analytics",
        "insights", "visualization", "reporting", "dashboard",
    ];

    let lower = term.to_lowercase();
    if generic_tech_adjacent.contains(&lower.as_str()) {
        return true;
    }

    // Check for version numbers, camelCase, special chars
    lower.contains(|c: char| c.is_ascii_digit() || c == '+' || c == '#' || c == '.')
        || lower.contains("js")
        || lower.contains("ql")
        || lower.chars().filter(|c| c.is_uppercase()).count() >= 2
}

/// Extract role titles dynamically from resume text.
///
/// Instead of matching against a fixed list of valid titles (which misses
/// "Distributed Systems Engineer", "Systems Programming" or anything that
/// doesn't fit a predefined pattern), this finds job titles by looking at
/// the resume's natural structure:
///
/// 1. Lines near date ranges (the line above a date is almost always a title)
/// 2. Explicit labels ("Title: X", "Role: X")
/// 3. Lines with role-identifying words + capitalisation
fn extract_roles(text: &str) -> Vec<String> {
    let mut roles: Vec<String> = Vec::new();

    // ── Method 1: Explicit labels ───────────────────────────────────
    // "Role: Senior Engineer", "Position: Lead Developer"
    let label_re = Regex::new(
        r"(?i)(?:role|position|title|current|designation)[:\s]+([a-z][a-z\s\-/&]{3,60}?)(?:\s*[|\(]|\s*$)"
    ).unwrap();
    for cap in label_re.captures_iter(text) {
        let role = cap.get(1).unwrap().as_str().trim().to_string();
        if role.len() >= 5 && role.len() <= 70 {
            roles.push(role);
        }
    }

    // ── Method 2: Date-driven titles ────────────────────────────────
    // The line BEFORE a date range in a resume is typically a job title.
    // Uses a simple two-step scan (NO complex regex that can freeze):
    // Step 1: find lines that look like date ranges
    // Step 2: grab the line before each date range
    // This avoids catastrophic backtracking from chained lazy quantifiers.
    let simple_date_re = Regex::new(
        r"(?i)\b\d{4}\s*[–\-to]+\s*(?:\d{4}|present|current|now)\b"
    ).unwrap();

    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if i == 0 { continue; }
        if simple_date_re.is_match(line) {
            let candidate = lines[i - 1].trim().to_string();
            if looks_like_role_line(&candidate) && !roles.contains(&candidate) {
                roles.push(candidate);
            }
        }
    }

    // ── Method 3: Broad role keyword + capitalisation ───────────────
    // Catches any line that looks like it could be a job title,
    // without requiring specific words.
    let role_keywords = [
        "engineer", "developer", "architect", "programmer", "scientist",
        "analyst", "designer", "manager", "director", "lead", "head",
        "specialist", "consultant", "coordinator", "administrator",
        "intern", "researcher", "instructor", "associate",
    ];

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.len() > 80 {
            continue;
        }
        let lower_line = trimmed.to_lowercase();

        // Check if any word in the line is a role keyword
        let words: Vec<&str> = lower_line.split_whitespace().collect();
        let has_keyword = role_keywords.iter().any(|kw| words.contains(kw));

        if has_keyword && looks_like_role_line(trimmed) {
            let title = trimmed.to_string();
            if !roles.contains(&title) {
                roles.push(title);
            }
        }
    }

    roles.sort_unstable();
    roles.dedup();
    roles
}

/// Heuristic: does this line look like a job title?
/// Job titles have capitalised words, aren't section headers, aren't
/// bullet points, and have meaningful length.
fn looks_like_role_line(line: &str) -> bool {
    let lower = line.to_lowercase().trim().to_string();

    // Exclude section headers
    let section_headers = [
        "experience", "education", "skills", "summary", "objective",
        "projects", "publications", "certifications", "references",
        "volunteer", "leadership", "languages", "interests",
        "achievements", "awards", "honors", "contact", "profile",
        "technical", "work", "employment", "background", "qualifications",
        "training", "courses", "patents",
    ];
    if section_headers.iter().any(|h| lower == *h || lower.starts_with(&format!("{}:", h))) {
        return false;
    }

    if line.len() < 5 || line.len() > 80 {
        return false;
    }

    // Must have at least one uppercase letter or digit
    if !line.chars().any(|c| c.is_uppercase()) && !line.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }

    // Must have at least 2 words (or 1 word >= 5 chars)
    let word_count = line.split_whitespace().count();
    if word_count < 2 && line.len() < 5 {
        return false;
    }

    // Exclude bullet points / list items
    let trimmed = line.trim_start();
    if trimmed.starts_with('-') || trimmed.starts_with('•')
        || trimmed.starts_with('*') || trimmed.starts_with("·") {
        return false;
    }

    // At least one word has 3+ chars
    line.split_whitespace().any(|w| w.len() >= 3)
}

/// Infer seniority level from role titles and text.
///
/// Strategy: role titles are the PRIMARY signal (they're explicit about level).
/// Text is the FALLBACK. And we check highest levels first so "senior engineer"
/// doesn't get overridden by "intern" appearing somewhere irrelevant in the text.
fn infer_seniority(roles: &[String], text: &str) -> Option<SeniorityLevel> {
    let combined = format!("{} {}", roles.join(" "), text).to_lowercase();

    // ── Role-title-driven checks (strongest signal) ─────────────────
    // If any role title explicitly says the level, trust it first.
    for role in roles {
        let r = role.to_lowercase();
        if r.contains("vice president") || r.contains("vp ") || r.contains("chief") || r.contains("cto") || r.contains("ceo") {
            return Some(SeniorityLevel::Executive);
        }
        if r.contains("director") {
            return Some(SeniorityLevel::Director);
        }
        if r.contains("principal") {
            return Some(SeniorityLevel::Principal);
        }
        if r.contains("staff") {
            return Some(SeniorityLevel::Staff);
        }
        if r.contains("senior") || r.contains("sr ") {
            return Some(SeniorityLevel::Senior);
        }
        if r.contains("lead") {
            return Some(SeniorityLevel::Lead);
        }
        if r.contains("junior") || r.contains("jr ") || r.contains("jr,") {
            return Some(SeniorityLevel::Junior);
        }
        if r.contains("intern") {
            return Some(SeniorityLevel::Intern);
        }
    }

    // ── Text-fallback checks (weaker signal, check high→low) ────────
    // Use word-boundary checks to avoid "internal" → "intern" false positive
    if combined.contains("vp ") || combined.contains("vice president") || combined.contains("chief") || combined.contains("cto") || combined.contains("ceo") {
        return Some(SeniorityLevel::Executive);
    }
    if combined.contains("director") {
        return Some(SeniorityLevel::Director);
    }
    if combined.contains("principal") {
        return Some(SeniorityLevel::Principal);
    }
    if combined.contains("staff") {
        // "staff" is common in resumes ("staff engineer" vs "staff augmentation")
        // Only trust if near engineering/technical context
        let ctx = format!("staff {} {}", text, roles.join(" ")).to_lowercase();
        if ctx.contains("staff engineer") || ctx.contains("staff software") || ctx.contains("staff scientist") {
            return Some(SeniorityLevel::Staff);
        }
    }
    if combined.contains("senior") || combined.contains("sr ") {
        return Some(SeniorityLevel::Senior);
    }
    if combined.contains("lead") {
        return Some(SeniorityLevel::Lead);
    }
    if combined.contains("junior") || combined.contains("jr ") {
        return Some(SeniorityLevel::Junior);
    }
    // Only match "intern" as a standalone word, not "internal" or "internship"
    if combined.contains(" intern ") || combined.starts_with("intern ") || combined.ends_with(" intern") || combined.contains("\nintern") {
        return Some(SeniorityLevel::Intern);
    }

    None
}

/// Extract years of experience.
fn extract_experience_years(text: &str) -> Option<f32> {
    // Primary: "N years of experience", "N+ years exp"
    let re = Regex::new(r"(?i)(\d{1,2})\+?\s*(?:years?|yrs?)\s*(?:of\s+)?(?:experience|exp)").unwrap();
    if let Some(cap) = re.captures(text) {
        if let Ok(y) = cap.get(1)?.as_str().parse::<f32>() {
            if y > 0.0 && y < 50.0 {
                return Some(y);
            }
        }
    }

    // Fallback: "N+ years" in experience section context
    let re2 = Regex::new(r"(?i)(?:over|>|more\s+than)\s*(\d{1,2})\+?\s*years?").unwrap();
    if let Some(cap) = re2.captures(text) {
        if let Ok(y) = cap.get(1)?.as_str().parse::<f32>() {
            if y > 0.0 && y < 50.0 {
                return Some(y);
            }
        }
    }

    None
}

/// Extract preferred location.
fn extract_location(text: &str) -> Option<String> {
    let re = Regex::new(
        r"(?i)(?:based|located|living|situated|relocated|willing\s+to\s+relocate\s+to)\s+(?:in|at|near|to)\s+([a-z][a-z\s.-]{2,40}?)(?:[,.!]|$)"
    ).unwrap();
    re.captures(text).and_then(|c| {
        let loc = c.get(1)?.as_str().trim().to_string();
        (loc.len() <= 50 && loc.len() >= 2).then_some(loc)
    })
}

/// Extract preferred job type.
fn extract_job_type(text: &str) -> Option<String> {
    let re = Regex::new(
        r"(?i)(?:looking|seeking|want|prefer|open|available|interested)\s+(?:for|a|an|in)?\s*(full[- ]time|part[- ]time|contract|remote|hybrid|onsite|internship)"
    ).unwrap();
    if let Some(cap) = re.captures(text) {
        return cap.get(1).map(|m| m.as_str().to_string());
    }

    // Fallback: scan for job type keywords in preference context
    let section = Regex::new(r"(?i)(?:type|preference|work\s+mode)[:\s]+([^\\n]+)").unwrap();
    if let Some(cap) = section.captures(text) {
        let s = cap.get(1).unwrap().as_str().to_lowercase();
        for t in &["remote", "hybrid", "onsite", "full-time", "full time", "part-time", "part time", "contract", "internship"] {
            if s.contains(t) {
                return Some(t.to_string());
            }
        }
    }

    None
}

/// Detect domains from known skills — ranked by number of skills per domain.
///
/// Only returns domains that are meaningfully represented (at least 2 skills),
/// sorted by strength (most skills first). This prevents showing ALL domains
/// when someone has a diverse skill set — you only see your top areas.
/// Derive focus areas dynamically from the resume text.
///
/// This is NOT hardcoded skill→domain mapping. Instead, it looks at what
/// the resume actually talks about: job titles, frequently mentioned tech
/// areas, and role context. Whatever your resume emphasises, that's what
/// shows up — no "DevOps" label unless you actually write "DevOps".
fn derive_focus_areas(text: &str, role_titles: &[String]) -> Vec<String> {
    let mut areas: Vec<String> = Vec::new();

    // 1. Extract multi-word phrases that look like focus areas
    // Look for patterns like "distributed systems", "systems programming",
    // "machine learning", "data infrastructure", etc.
    let phrase_re = Regex::new(
        r"(?i)(?:focus|area|expertise|specializ|field|domain|discipline|responsib|work\s+on|work\s+in|work\s+with|build|design|develop|architect|lead|manage|drive)\s*(?:ed|ing|es)?\s*:?\s*([a-z][a-z\s\-/]{3,60}?)(?:[.!,]|$)"
    ).unwrap();

    for cap in phrase_re.captures_iter(text) {
        let phrase = cap.get(1).unwrap().as_str().trim().to_string();
        if phrase.len() >= 8 && phrase.len() <= 80 {
            let words: Vec<&str> = phrase.split_whitespace().collect();
            if words.len() >= 2 {
                areas.push(phrase);
            }
        }
    }

    // 2. Extract bigrams/trigrams that appear 2+ times in the resume
    // (high-frequency phrases indicate what the resume is about)
    let words: Vec<&str> = text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.len() >= 4 && !is_stop_word(w))
        .collect();

    let mut bigram_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for window in words.windows(2) {
        let bigram = format!("{} {}", window[0], window[1]);
        *bigram_counts.entry(bigram.to_lowercase()).or_insert(0) += 1;
    }

    let mut frequent: Vec<(String, usize)> = bigram_counts.into_iter()
        .filter(|(phrase, count)| *count >= 2 && !is_stop_phrase(phrase))
        .collect();
    frequent.sort_by(|a, b| b.1.cmp(&a.1));

    for (phrase, _) in frequent.iter().take(5) {
        if !areas.contains(phrase) {
            areas.push(phrase.clone());
        }
    }

    // 3. Extract technology groupings from role titles + context
    // If a role title has "systems" or "distributed" or "infrastructure",
    // those describe focus areas
    for role in role_titles {
        let r = role.to_lowercase();
        // Extract adjective/noun pairs that describe the work
        let desc_re = Regex::new(r"(?i)(distributed\s+systems|systems\s+\w+|data\s+\w+|platform\s+\w+|cloud\s+\w+|infrastructure\s+\w+|network\s+\w+|security\s+\w+|backend\s+\w+|frontend\s+\w+|full.?stack|machine\s+learning|deep\s+learning|site\s+reliability|software\s+\w+|embedded\s+\w+|mobile\s+\w+|web\s+\w+)").unwrap();
        for cap in desc_re.captures_iter(&r) {
            let area = cap.get(1).unwrap().as_str().to_string();
            if !areas.contains(&area) {
                areas.push(area);
            }
        }
    }

    // Clean up and dedup
    areas.retain(|a| a.len() >= 6);
    areas.sort_unstable();
    areas.dedup();

    // Take top 6 max
    areas.truncate(6);
    areas
}

fn is_stop_word(word: &str) -> bool {
    let stops = [
        "the", "and", "for", "are", "but", "not", "you", "all", "can",
        "was", "one", "our", "out", "has", "have", "been", "some", "same",
        "its", "than", "them", "into", "two", "more", "these", "like",
        "such", "that", "this", "with", "from", "your", "which", "each",
        "will", "about", "between", "very", "just", "their", "would",
        "could", "should", "then", "there", "where", "while", "because",
        "before", "does", "doing", "done", "much", "many", "most",
        "need", "take", "make", "well", "also", "back", "still", "get",
        "use", "used", "using", "way", "new", "first", "last", "own",
        "see", "may", "though", "every", "good", "great", "best",
    ];
    stops.contains(&word)
}

fn is_stop_phrase(phrase: &str) -> bool {
    let stops = [
        "work with", "work on", "work in", "work for", "work as",
        "experience with", "experience in", "proficient in",
        "strong experience", "solid experience", "extensive experience",
        "responsible for", "responsible the", "including the",
        "well as", "such as", "as well", "order to",
    ];
    stops.contains(&phrase.to_lowercase().as_str())
}

/// Extract education entries — strict mode.
///
/// Only matches well-formed degree names (no fragment matches like "ms la").
/// Full names ("Bachelor of Science") or abbreviations with "in" ("MS in CS").
fn extract_education(text: &str) -> Vec<Education> {
    let mut education = Vec::new();

    // Pattern 1: Full degree names like "Bachelor of Science", "Master of Engineering"
    let full_re = Regex::new(
        r"(?i)((?:bachelor\s+(?:of|in)|master\s+(?:of|in)|ph\.?d\.?(?:\s+in)?|doctorate(?:\s+in)?|associate\s+(?:of|in))\s+[a-z]{4,}(?:\s+[a-z]{2,}){0,3})"
    ).unwrap();

    for cap in full_re.captures_iter(text) {
        let deg = cap.get(1).unwrap().as_str().trim().to_string();
        if deg.len() >= 10 && deg.len() < 100 {
            let edu = Education {
                degree: Some(deg),
                field: None,
                institution: None,
            };
            education.push(edu);
        }
    }

    // Pattern 2: Abbreviations ONLY with "in" or "of" + field name
    // e.g. "B.S. in Computer Science", "MS in Data Science" — NOT "ms la"
    let abbr_re = Regex::new(
        r"(?i)\b(b\.?\s*s\.?|m\.?\s*s\.?|b\.?\s*a\.?|m\.?\s*a\.?|bs|ba|ms|ma|b\.?tech|m\.?tech)\s+(?:in|of)\s+([a-z]{4,}(?:\s+[a-z]{2,}){0,3})"
    ).unwrap();

    for cap in abbr_re.captures_iter(text) {
        let degree_prefix = cap.get(1).unwrap().as_str().trim().to_string();
        let field = cap.get(2).unwrap().as_str().trim().to_string();
        let degree_str = format!("{} in {}", degree_prefix, field);
        if field.len() >= 4 && degree_str.len() < 100 {
            education.push(Education {
                degree: Some(degree_str),
                field: Some(field),
                institution: None,
            });
        }
    }

    // Extract institution names — only if >= 8 chars (looks real)
    let institution_re = Regex::new(
        r"(?i)((?:university|college|institute|school)\s+of\s+[a-z]{4,}(?:\s+[a-z]{2,}){0,3}|[a-z]{4,}\s+(?:university|college|institute))"
    ).unwrap();

    for cap in institution_re.captures_iter(text) {
        let inst = cap.get(1).unwrap().as_str().trim().to_string();
        if inst.len() >= 8 && !education.is_empty() {
            if let Some(edu) = education.last_mut() {
                edu.institution = Some(inst);
            }
        }
    }

    education.dedup_by(|a, b| a.degree == b.degree);
    education
}

/// Extract certifications.
fn extract_certifications(text: &str) -> Vec<String> {
    let mut certs = Vec::new();

    let patterns = [
        r"(?i)(?:certified|certification|certificate|credential)\s+(?:in|:)?\s*([a-z][a-z\s]{2,50}(?:certification)?)",
        r"(?i)([a-z][a-z\s]{2,40})\s+(?:certification|certificate|credential)",
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            for cap in re.captures_iter(text) {
                let cert = cap.get(1).unwrap().as_str().trim().to_string();
                if cert.len() > 3 && !certs.contains(&cert) {
                    certs.push(cert);
                }
            }
        }
    }

    // Known certifications (case-insensitive match)
    let known_certs = [
        "aws certified", "aws certification", "azure certified", "azure certification",
        "gcp certified", "gcp certification", "google cloud certified",
        "cka", "ckad", "cks", "cissp", "ceh", "oscp", "pmp", "scrum master",
        "csm", "psm", "safe", "itil", "comptia", "ccna", "ccnp", "ccie",
        "ocp", "ocjp", "ocajp", "rhcsa", "rhce", "lfcs", "cfe", "cfa",
        "frm", "prince2", "six sigma", "lean", "cobit", "togaf",
    ];

    for cert in &known_certs {
        if text.to_lowercase().contains(cert) {
            certs.push(cert.to_string());
        }
    }

    certs.sort_unstable();
    certs.dedup();
    certs
}

/// Build meaningful keywords — filtered to be useful for job matching.
fn build_keywords(
    known_skills: &[KnownSkill],
    role_titles: &[String],
    inferred_skills: &[String],
    text: &str,
) -> Vec<String> {
    let mut keywords: Vec<String> = Vec::new();

    // 1. All known skill names
    for ks in known_skills {
        keywords.push(ks.name.to_lowercase());
    }

    // 2. All role titles (as multi-word keywords)
    for role in role_titles {
        keywords.push(role.to_lowercase());
        // Also add individual significant words from roles
        for word in role.split_whitespace() {
            let w = word.to_lowercase();
            if w.len() >= 3 && !is_junk_word(&w) {
                keywords.push(w);
            }
        }
    }

    // 3. Inferred skills
    for s in inferred_skills {
        keywords.push(s.to_lowercase());
    }

    // 4. Focus-area derived keywords from frequent resume phrases
    let focus_areas = derive_focus_areas(text, role_titles);
    for area in &focus_areas {
        keywords.push(area.to_lowercase());
        for word in area.split_whitespace() {
            if word.len() >= 3 && !is_junk_word(word) {
                keywords.push(word.to_lowercase());
            }
        }
    }

    // 5. Experience-seniority keywords
    let seniority_kws = ["senior", "staff", "principal", "lead", "junior", "experienced"];
    for sk in &seniority_kws {
        if text.contains(sk) && !keywords.contains(&sk.to_string()) {
            keywords.push(sk.to_string());
        }
    }

    keywords.sort_unstable();
    keywords.dedup();
    keywords
}

/// Check if a word is junk (common stop word, not useful for matching).
fn is_junk_word(word: &str) -> bool {
    let junk = [
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had",
        "was", "one", "our", "out", "has", "have", "been", "some", "same",
        "its", "than", "them", "into", "two", "more", "these", "like", "over",
        "such", "that", "this", "with", "from", "your", "which", "each", "will",
        "about", "between", "under", "very", "just", "their", "would", "after",
        "could", "should", "than", "then", "there", "where", "while", "because",
        "before", "does", "doing", "done", "much", "many", "most", "must",
        "need", "take", "make", "made", "well", "work", "year", "years",
        "also", "back", "still", "get", "use", "used", "using", "way", "new",
        "first", "last", "own", "see", "say", "said", "may", "though",
    ];
    junk.contains(&word)
}
