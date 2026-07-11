//! Aggregates job listings from multiple sources and scores them against your resume.
//!
//! This is the library root. Re-exported modules:
//!
//! * `cli` — interactive and non-interactive command handling
//! * `crawler` — job board scrapers (Remote OK, Reddit, Hacker News)
//! * `matcher` — resume-to-job scoring engine
//! * `models` — shared data types (JobPost, Resume, MatchResult, etc.)
//! * `storage` — JSON persistence for resumes, preferences, history, results

pub mod cli;
pub mod crawler;
pub mod matcher;
pub mod models;
pub mod storage;
