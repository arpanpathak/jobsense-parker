//! # Transformer-Powered Scoring Engine
//!
//! Uses a **sentence transformer** model (`all-MiniLM-L6-v2`) to encode
//! resume and job texts into dense vector embeddings, then computes
//! cosine similarity between them.
//!
//! This captures deep semantic similarity that bag-of-words approaches
//! (TF-IDF, keyword counting) cannot:
//!
//! | Term pair | TF-IDF | Transformer |
//! |-----------|--------|-------------|
//! | `"kubernetes"` vs `"k8s"` | 0% | ~85% |
//! | `"React"` vs `"ReactJS"` | 0% | ~92% |
//! | `"software engineer"` vs `"software developer"` | ~30% | ~95% |
//! | `"experienced"` vs `"5 years"` | 0% | ~60% |
//!
//! ## Model
//!
//! `sentence-transformers/all-MiniLM-L6-v2` (~80MB safetensors).
//! Downloaded automatically on first run via HuggingFace Hub.
//! Cached at `~/.cache/huggingface/hub/`.
//!
//! ## Architecture
//!
//! The model is loaded **once** (lazy static) and reused across all
//! scoring calls within a session. Encoding is batched for efficiency.

use std::sync::OnceLock;

use anyhow::{Context, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::api::sync::Api;
use tokenizers::Tokenizer;

use crate::models::{JobPost, Resume};

// ─── Model Constants ────────────────────────────────────────────────────────

/// HuggingFace model ID for the sentence transformer.
const MODEL_ID: &str = "sentence-transformers/all-MiniLM-L6-v2";

/// Maximum sequence length for tokenization.
const MAX_LEN: usize = 256;

// ─── Lazy Model Loading ─────────────────────────────────────────────────────

/// Global singleton for the transformer model. Loaded once on first use.
static TRANSFORMER: OnceLock<TransformerScorer> = OnceLock::new();

/// Get or initialize the transformer scorer.
fn get_transformer() -> Result<&'static TransformerScorer> {
    if let Some(t) = TRANSFORMER.get() {
        return Ok(t);
    }
    // First call: initialize the model.
    // Race-safe: if two threads race, only one set() succeeds.
    eprintln!("  ⏳ Loading NLP model (first run downloads ~80MB)...");
    match TransformerScorer::new() {
        Ok(scorer) => {
            eprintln!("  ✓ NLP model loaded.");
            let _ = TRANSFORMER.set(scorer);
            Ok(TRANSFORMER.get().unwrap())
        }
        Err(e) => {
            eprintln!("  ✗ Failed to load NLP model: {e}");
            Err(e)
        }
    }
}

// ─── Transformer Scorer ─────────────────────────────────────────────────────

/// Wraps a sentence transformer model for computing text embeddings.
struct TransformerScorer {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl TransformerScorer {
    /// Load the model from HuggingFace Hub.
    ///
    /// Downloads `all-MiniLM-L6-v2` on first run. Subsequent runs use
    /// the cached copy at `~/.cache/huggingface/hub/`.
    fn new() -> Result<Self> {
        let device = Device::Cpu;

        let api = Api::new()?;
        let repo = api.model(MODEL_ID.to_string());

        let tokenizer_path = repo
            .get("tokenizer.json")
            .context("Failed to download tokenizer.json")?;
        let config_path = repo
            .get("config.json")
            .context("Failed to download config.json")?;
        let weights_path = repo
            .get("model.safetensors")
            .context("Failed to download model.safetensors")?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;

        let config: Config = serde_json::from_str(
            &std::fs::read_to_string(&config_path)
                .context("Failed to read config.json")?,
        )
        .context("Failed to parse config.json")?;

        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device)?
        };
        let model = BertModel::load(vb, &config)?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    /// Encode a batch of texts into normalized embedding vectors.
    ///
    /// Returns a `Vec` of embeddings, each a `Vec<f64>` of length 384
    /// (the embedding dimension for MiniLM-L6-v2).
    fn encode(&self, texts: &[String]) -> Result<Vec<Vec<f64>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // ── Tokenize ──────────────────────────────────────────────────
        let encodings = self
            .tokenizer
            .encode_batch(texts.to_vec(), false)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;

        let max_len = encodings
            .iter()
            .map(|e| e.get_ids().len())
            .max()
            .unwrap_or(0)
            .min(MAX_LEN);

        let batch_size = texts.len();
        let mut input_ids_arr = Vec::with_capacity(batch_size * max_len);
        let mut attention_mask_arr = Vec::with_capacity(batch_size * max_len);

        for encoding in &encodings {
            let ids = encoding.get_ids();
            let len = ids.len().min(MAX_LEN);

            for i in 0..max_len {
                if i < len {
                    input_ids_arr.push(ids[i] as u32);
                    attention_mask_arr.push(1u32);
                } else {
                    input_ids_arr.push(0u32);     // padding token
                    attention_mask_arr.push(0u32); // mask out
                }
            }
        }

        let input_ids = Tensor::from_slice(
            &input_ids_arr,
            (batch_size, max_len),
            &self.device,
        )?;
        let attention_mask = Tensor::from_slice(
            &attention_mask_arr,
            (batch_size, max_len),
            &self.device,
        )?;
        let token_type_ids = Tensor::zeros((batch_size, max_len), candle_core::DType::U32, &self.device)?;

        // ── Forward pass through BERT ─────────────────────────────────
        // Output shape: [batch, seq_len, hidden_size]
        let hidden = self
            .model
            .forward(&input_ids, &token_type_ids, Some(&attention_mask))?;

        // ── Mean pooling ───────────────────────────────────────────────
        // Average the non-padding token embeddings for each sequence.
        let mask_f32 = attention_mask
            .to_dtype(candle_core::DType::F32)?
            .unsqueeze(2)?; // [batch, seq, 1]

        // Use broadcast_mul to avoid shape mismatch: [b, s, h] * [b, s, 1]
        let sum_embeddings = hidden.broadcast_mul(&mask_f32)?.sum(1)?; // [batch, hidden]
        let mask_sum = mask_f32.sum(1)?;                               // [batch, hidden]
        let mean_embeddings = sum_embeddings.broadcast_div(&mask_sum)?; // [batch, hidden]

        // ── L2 normalize ──────────────────────────────────────────────
        let normalized = mean_embeddings.broadcast_div(
            &mean_embeddings.sqr()?.sum(1)?.sqrt()?.unsqueeze(1)?,
        )?;

        // ── Convert to Vec<Vec<f64>> ──────────────────────────────────
        let dim = normalized.dim(1)?;
        let flat: Vec<f32> = normalized.to_vec2::<f32>()?.into_iter().flatten().collect();

        let embeddings: Vec<Vec<f64>> = flat
            .chunks(dim)
            .map(|chunk| chunk.iter().map(|&v| v as f64).collect())
            .collect();

        Ok(embeddings)
    }
}

// ─── Cosine Similarity ──────────────────────────────────────────────────────

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    dot.clamp(0.0, 1.0) // vectors are already L2-normalized, dot = cos
}

// ─── Text Builders ──────────────────────────────────────────────────────────

/// Concatenate relevant fields of a job post into a single string for embedding.
///
/// Combines `title`, `description`, `company`, `location`, `salary`,
/// and `job_type` into one space-separated string.
///
/// Tags are excluded (job boards dump platform-level tag clouds).
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

/// Build resume text for embedding — combines all relevant fields.
pub fn build_resume_text(resume: &Resume) -> String {
    let mut parts: Vec<String> = Vec::new();

    for role in &resume.role_titles {
        parts.push(role.clone());
    }
    for area in &resume.focus_areas {
        parts.push(area.clone());
    }
    for skill in &resume.skills {
        parts.push(skill.clone());
    }
    for kw in &resume.keywords {
        parts.push(kw.clone());
    }
    if let Some(loc) = &resume.preferred_location {
        parts.push(format!("location: {}", loc));
    }
    if let Some(jt) = &resume.preferred_job_type {
        parts.push(format!("type: {}", jt));
    }
    if let Some(yrs) = resume.experience_years {
        parts.push(format!("{} years experience", yrs as u32));
    }

    parts.join(" ").trim().to_string()
}

// ─── Core Scorer ────────────────────────────────────────────────────────────

/// Score a batch of jobs against a resume using transformer embeddings.
///
/// ## ML Pipeline
///
/// 1. Encode resume text into a 384-dim embedding vector
/// 2. Encode all job texts into 384-dim embedding vectors (batched)
/// 3. Cosine similarity between resume vector and each job vector
/// 4. Small bonus modifiers for location/job-type preferences
///
/// Returns scores in the same order as the input `jobs` slice.
pub fn score_batch(jobs: &[JobPost], resume: &Resume) -> Vec<f64> {
    if jobs.is_empty() {
        return vec![];
    }

    // Get or initialize the transformer model
    let transformer = match get_transformer() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("  Warning: NLP model unavailable ({e}), scoring disabled.");
            return vec![0.0; jobs.len()];
        }
    };

    // Build texts
    let resume_text = build_resume_text(resume);
    let job_texts: Vec<String> = jobs.iter().map(|j| build_job_text(j)).collect();

    // Encode
    let resume_embedding = match transformer.encode(&[resume_text]) {
        Ok(mut v) => v.pop().unwrap_or_default(),
        Err(e) => {
            eprintln!("  Warning: failed to encode resume: {e}");
            return vec![0.0; jobs.len()];
        }
    };

    let job_embeddings = match transformer.encode(&job_texts) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("  Warning: failed to encode jobs: {e}");
            return vec![0.0; jobs.len()];
        }
    };

    // Compute cosine similarity for each job + bonus modifiers
    jobs.iter()
        .zip(job_embeddings.iter())
        .map(|(job, job_emb)| {
            let mut score = cosine_similarity(&resume_embedding, job_emb);

            // Small bonuses for explicit preference matches
            let title_lower = job.title.to_lowercase();
            let desc_lower = job.description.to_lowercase();

            // Role-title bonus
            let has_role_match = resume.role_titles.iter().any(|r| {
                let rl = r.to_lowercase();
                title_lower.contains(&rl) || desc_lower.contains(&rl)
            });
            if has_role_match {
                score += 0.05;
            }

            // Location bonus
            if let (Some(pref_loc), Some(job_loc)) = (&resume.preferred_location, &job.location) {
                let pl = pref_loc.to_lowercase();
                let jl = job_loc.to_lowercase();
                if pl.contains(&jl) || jl.contains(&pl) {
                    score += 0.03;
                }
            }

            // Job-type bonus
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

// ─── Tag Cloud Stripper ─────────────────────────────────────────────────────

/// Strip tag-cloud sections from job descriptions.
///
/// Many job boards append a comma-separated tag cloud of every keyword.
/// These inflate similarity scores. We detect common section markers
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
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_build_resume_text_includes_fields() {
        let resume = make_resume(&["rust", "python"], &["senior engineer"]);
        let text = build_resume_text(&resume);
        assert!(text.contains("rust"));
        assert!(text.contains("python"));
        assert!(text.contains("senior engineer"));
    }

    #[test]
    fn test_strip_tag_cloud() {
        let text = "We need a Rust engineer.\nTags: rust, python, go";
        let stripped = strip_tag_cloud(text);
        assert!(!stripped.contains("Tags:"));
        assert!(stripped.contains("Rust engineer"));
    }
}
