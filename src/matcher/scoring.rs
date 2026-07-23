//! # ML-Powered Scoring Engine
//!
//! Computes a compatibility score between a [`Resume`] and a [`JobPost`]
//! using **TF-IDF vectorization + Cosine Similarity** — an unsupervised
//! machine learning technique from Information Retrieval.
//!
//! ## How It Works
//!
//! 1. **Tokenization** — text is lowercased, split into tokens, filtered
//!    for stop words and short/noisy terms.
//! 2. **TF-IDF Vectorization** — each document (resume + each job) is
//!    converted into a vector where each dimension is a term weighted by
//!    how important it is to that document vs. the whole corpus.
//! 3. **Cosine Similarity** — the angle between the resume vector and each
//!    job vector measures semantic alignment. Closer vectors = better match.
//!
//! ## Why TF-IDF over Hardcoded Weights?
//!
//! The old scoring had manually tuned weights (title skills 35%, desc skills
//! 25%, etc.) that were brittle and domain-specific. TF-IDF learns term
//! importance from the data itself:
//!
//! - A rare skill mentioned in both resume and job → high weight (high IDF)
//! - A common word like "experience" → low weight (low IDF)
//! - Skills in the job title naturally get more weight because the title is
//!   shorter (higher TF density)
//!
//! ## Bonus Features
//!
//! After the ML similarity, small bonuses are added for exact/preference
//! matches (location, job type) that TF-IDF might not capture well.

use std::collections::{HashMap, HashSet};

use crate::models::{JobPost, Resume};

// ─── Stop Words ──────────────────────────────────────────────────────────────

/// Common English words excluded from the vocabulary.
/// These carry no signal for resume-job matching.
const STOP_WORDS: &[&str] = &[
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
    "every", "good", "great", "best", "away", "here", "there", "when",
    "what", "who", "why", "how", "where", "which",
    // Domain-agnostic resume noise
    "experience", "company", "team", "project", "projects", "including",
    "various", "multiple", "different", "wide", "range", "etc",
    "looking", "seek", "seeking", "want", "wanted", "need", "needed",
    "work", "working", "built", "building", "develop", "developing",
    "developed", "design", "designed", "designing", "implement",
    "implemented", "implementing", "manage", "managed", "managing",
];

// ─── Tokenizer ───────────────────────────────────────────────────────────────

/// Tokenize text into clean, meaningful terms for ML vectorization.
///
/// Pipeline:
/// 1. Lowercase
/// 2. Split on whitespace
/// 3. Strip non-alphanumeric characters (except +, #, . for tech names)
/// 4. Filter out stop words, short words (< 2 chars), pure numbers
fn tokenize(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    lower
        .split_whitespace()
        .filter_map(|word| {
            // Clean the token but preserve tech-relevant special chars
            let clean: String = word
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '+' || *c == '#' || *c == '.')
                .collect();

            if clean.len() < 2 {
                return None;
            }

            // Skip pure numbers/dates
            if clean.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }

            // Skip stop words
            if STOP_WORDS.contains(&clean.as_str()) {
                return None;
            }

            Some(clean)
        })
        .collect()
}

// ─── TF-IDF Vectorizer ──────────────────────────────────────────────────────

/// A fitted TF-IDF vectorizer that transforms documents into term vectors.
///
/// This is an **unsupervised ML model** — it learns term importance (IDF)
/// from the corpus of documents it's trained on.
struct TfIdfVectorizer {
    /// Ordered vocabulary: index → term string
    vocabulary: Vec<String>,
    /// Precomputed IDF for each term in the vocabulary
    idf: Vec<f64>,
}

impl TfIdfVectorizer {
    /// Fit the vectorizer on a set of tokenized documents.
    ///
    /// Learns:
    /// - Vocabulary: all unique terms across all documents
    /// - IDF: inverse document frequency for each term
    fn fit(tokenized_docs: &[Vec<String>]) -> Self {
        let n_docs = tokenized_docs.len() as f64;

        // Build vocabulary and document frequency
        let mut term_df: HashMap<&str, usize> = HashMap::new();
        let mut vocab_set: HashSet<&str> = HashSet::new();

        for doc in tokenized_docs {
            let mut seen_in_doc: HashSet<&str> = HashSet::new();
            for term in doc {
                vocab_set.insert(term);
                if seen_in_doc.insert(term) {
                    *term_df.entry(term).or_insert(0) += 1;
                }
            }
        }

        // Sort vocabulary for deterministic order
        let mut vocabulary: Vec<String> = vocab_set.iter().map(|s| (*s).to_string()).collect();
        vocabulary.sort();

        // Compute IDF for each term
        // IDF(t) = 1.0 + log((N + 1) / (df(t) + 1))
        // This is the "smooth" IDF variant that prevents division by zero
        let idf: Vec<f64> = vocabulary
            .iter()
            .map(|term| {
                let df = term_df.get(term.as_str()).copied().unwrap_or(0) as f64;
                1.0 + ((n_docs + 1.0) / (df + 1.0)).ln()
            })
            .collect();

        Self { vocabulary, idf }
    }

    /// Transform a tokenized document into a TF-IDF vector.
    ///
    /// TF-IDF(t, d) = TF(t, d) × IDF(t)
    /// where TF is the raw count of term t in document d.
    fn transform(&self, tokens: &[String]) -> Vec<f64> {
        // Compute TF (raw counts)
        let mut tf: HashMap<&str, f64> = HashMap::new();
        for token in tokens {
            *tf.entry(token).or_insert(0.0) += 1.0;
        }

        // Build the vector: TF-IDF for each vocabulary term
        self.vocabulary
            .iter()
            .enumerate()
            .map(|(i, term)| {
                let tf_val = tf.get(term.as_str()).copied().unwrap_or(0.0);
                tf_val * self.idf[i]
            })
            .collect()
    }
}

// ─── Cosine Similarity ──────────────────────────────────────────────────────

/// Compute cosine similarity between two vectors.
///
/// Returns a value in [0.0, 1.0] where 1.0 = identical direction.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot / (norm_a * norm_b)).clamp(0.0, 1.0)
}

// ─── Text Builders ──────────────────────────────────────────────────────────

/// Concatenate relevant fields of a job post into a single searchable string.
///
/// Combines `title`, `description`, `company`, `location`, `salary`,
/// and `job_type` into one space-separated string for skill matching.
///
/// **`tags` are deliberately excluded** because job boards like Remote OK
/// dump platform-level tag clouds onto every job listing.
///
/// Also strips marker sections like "Tags:", "Technologies:" etc. from
/// descriptions so platform tag dumps don't inflate skill matches.
///
/// # Example output
///
/// ```text
/// "Senior Rust Engineer We are looking for a Rust engineer... Stripe San Francisco $200k full-time"
/// ```
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

/// Build the resume text for ML comparison — combines all relevant fields.
///
/// Includes skills, role titles, keywords, focus areas, and preferences
/// into a single document that we can vectorize alongside job texts.
pub fn build_resume_text(resume: &Resume) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Role titles (weighted implicitly by TF — they're shorter, so each
    // term gets higher TF density naturally)
    for role in &resume.role_titles {
        parts.push(role.clone());
    }

    // Focus areas — what the candidate actually works on
    for area in &resume.focus_areas {
        parts.push(area.clone());
    }

    // Skills — the tech stack
    for skill in &resume.skills {
        parts.push(skill.clone());
    }

    // Keywords — extra matching terms
    for kw in &resume.keywords {
        parts.push(kw.clone());
    }

    // Preferences (location and type)
    if let Some(loc) = &resume.preferred_location {
        parts.push(format!("location: {}", loc));
    }
    if let Some(jt) = &resume.preferred_job_type {
        parts.push(format!("type: {}", jt));
    }

    // Experience years as a soft signal
    if let Some(yrs) = resume.experience_years {
        let yrs_str = format!("{} years", yrs as u32);
        parts.push(yrs_str);
    }

    parts.join(" ").trim().to_string()
}

// ─── Core ML Scorer (Batch) ────────────────────────────────────────────────

/// Score a batch of jobs against a resume using TF-IDF + cosine similarity.
///
/// This is more efficient than scoring one-at-a-time because we fit the
/// vectorizer once on the entire corpus instead of refitting for each job.
pub fn score_batch(jobs: &[JobPost], resume: &Resume) -> Vec<f64> {
    if jobs.is_empty() {
        return vec![];
    }

    // Build all job texts first
    let job_texts: Vec<String> = jobs.iter().map(|j| build_job_text(j)).collect();

    // Build corpus: resume + all jobs
    let resume_text = build_resume_text(resume);
    let resume_tokens = tokenize(&resume_text);

    let all_job_tokens: Vec<Vec<String>> = job_texts.iter().map(|t| tokenize(t)).collect();

    let mut corpus: Vec<Vec<String>> = Vec::with_capacity(1 + all_job_tokens.len());
    corpus.push(resume_tokens.clone());
    corpus.extend(all_job_tokens.clone());

    // Fit TF-IDF model on the full corpus
    let vectorizer = TfIdfVectorizer::fit(&corpus);

    // Transform resume into vector
    let resume_vec = vectorizer.transform(&resume_tokens);

    // Score each job
    jobs.iter()
        .map(|job| {
            let job_text = build_job_text(job);
            let tokens = tokenize(&job_text);
            let job_vec = vectorizer.transform(&tokens);

            let mut score = cosine_similarity(&resume_vec, &job_vec);

            // Bonuses
            let title_lower = job.title.to_lowercase();
            let desc_lower = job.description.to_lowercase();

            let has_role_match = resume.role_titles.iter().any(|r| {
                let rl = r.to_lowercase();
                title_lower.contains(&rl) || desc_lower.contains(&rl)
            });
            if has_role_match {
                score += 0.05;
            }

            if let (Some(pref_loc), Some(job_loc)) = (&resume.preferred_location, &job.location) {
                let pl = pref_loc.to_lowercase();
                let jl = job_loc.to_lowercase();
                if pl.contains(&jl) || jl.contains(&pl) {
                    score += 0.03;
                }
            }

            if let (Some(pref_type), Some(job_type)) = (&resume.preferred_job_type, &job.job_type) {
                let pt = pref_type.to_lowercase();
                let jt = job_type.to_lowercase();
                if pt == jt || jt.contains(&pt) || pt.contains(&jt) {
                    score += 0.02;
                }
            }

            score.clamp(0.0, 1.0)
        })
        .collect()
}

/// Strip tag-cloud sections from job descriptions.
///
/// Many job boards append a comma-separated tag cloud of every keyword.
/// These inflate skill matching scores. We detect common section markers
/// and truncate before them.
fn strip_tag_cloud(text: &str) -> String {
    let lower = text.to_lowercase();
    let markers = [
        "tags:", "technologies:", "tech stack:", "skills:",
        "requirements:", "nice to have:", "bonus points:",
        "preferred qualifications:",
    ];

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

// ─── Tests ──────────────────────────────────────────────────────────────────

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
    fn test_tokenize_basic() {
        let tokens = tokenize("Senior Rust Engineer with Python experience");
        assert!(tokens.contains(&"rust".to_string()));
        assert!(tokens.contains(&"python".to_string()));
        assert!(tokens.contains(&"senior".to_string()));
        assert!(tokens.contains(&"engineer".to_string()));
        // "with" is a stop word, should be filtered
        assert!(!tokens.contains(&"with".to_string()));
    }

    #[test]
    fn test_tokenize_tech_terms() {
        let tokens = tokenize("C++ TypeScript Node.js Kubernetes");
        assert!(tokens.contains(&"c++".to_string()));
        assert!(tokens.contains(&"typescript".to_string()));
        assert!(tokens.contains(&"node.js".to_string()));
        assert!(tokens.contains(&"kubernetes".to_string()));
    }

    #[test]
    fn test_tokenize_filters_stop_words() {
        let tokens = tokenize("the and for but not experience");
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_partial() {
        let a = vec![1.0, 1.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.7071).abs() < 0.01);
    }

    #[test]
    fn test_tfidf_vectorizer() {
        let docs = vec![
            tokenize("rust python kubernetes"),
            tokenize("java spring kubernetes"),
            tokenize("rust wasm browser"),
        ];

        let vectorizer = TfIdfVectorizer::fit(&docs);

        // "kubernetes" appears in 2 of 3 docs, so IDF should be lower
        // than "rust" (2 of 3) or "wasm" (1 of 3)
        assert!(vectorizer.vocabulary.contains(&"kubernetes".to_string()));
        assert!(vectorizer.vocabulary.contains(&"rust".to_string()));
        assert!(vectorizer.vocabulary.contains(&"wasm".to_string()));

        // Transform a document
        let tokens = tokenize("rust python");
        let vec = vectorizer.transform(&tokens);
        assert_eq!(vec.len(), vectorizer.vocabulary.len());
    }

    #[test]
    fn test_ml_scoring_rust_job() {
        let jobs = vec![make_job("Rust Engineer", "We need a Rust engineer with Python experience")];
        let resume = make_resume(&["rust", "python"], &["engineer"]);
        let scores = score_batch(&jobs, &resume);
        assert!(scores[0] > 0.3);
    }

    #[test]
    fn test_ml_scoring_unrelated_job() {
        let jobs = vec![make_job("Barista", "Making coffee and serving customers")];
        let resume = make_resume(&["rust", "python", "kubernetes"], &["software engineer"]);
        let scores = score_batch(&jobs, &resume);
        // Should be very low — no overlap in vocabulary
        assert!(scores[0] < 0.2);
    }

    #[test]
    fn test_ml_scoring_ranks_correctly() {
        let jobs = vec![
            make_job("Senior Rust Engineer", "Looking for a Rust developer with Python skills and distributed systems experience"),
            make_job("Fullstack Developer", "Python Go React TypeScript full stack development"),
            make_job("Barista", "Coffee shop experience required making and serving coffee"),
        ];
        let resume = make_resume(&["rust", "python", "go", "kubernetes"], &["senior software engineer"]);

        let scores = score_batch(&jobs, &resume);

        // Rust job should score highest (matches rust + python)
        assert!(scores[0] > scores[1]);
        assert!(scores[0] > scores[2]);

        // Fullstack should beat barista (python + go match)
        assert!(scores[1] > scores[2]);
    }

    #[test]
    fn test_empty_corpus() {
        let scores = score_batch(&[], &make_resume(&["rust"], &["engineer"]));
        assert!(scores.is_empty());
    }

    #[test]
    fn test_build_resume_text_includes_fields() {
        let resume = make_resume(&["rust", "python"], &["senior engineer"]);
        let text = build_resume_text(&resume);
        assert!(text.contains("rust"));
        assert!(text.contains("python"));
        assert!(text.contains("senior engineer"));
    }
}
