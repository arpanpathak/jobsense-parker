//! CLI application logic — interactive menu and non-interactive command dispatch.

mod views;

use colored::Colorize;
use dialoguer::{FuzzySelect, Input, Select};
use uuid::Uuid;

use crate::crawler::CrawlerCoordinator;
use crate::matcher::Matcher;
use crate::models::{Command, MatchResult, Resume, ScanRecord, SearchConfig};
use crate::storage;

pub use views::{banner, print_help, show_scan_history};

/// Main application struct that ties together the matcher, crawlers, and storage.
pub struct App {
    matcher: Matcher,
    coordinator: CrawlerCoordinator,
    results: Vec<MatchResult>,
    config: SearchConfig,
    scan_history: Vec<ScanRecord>,
}

impl App {
    /// Create a new app instance, loading persisted state (resume, results, history).
    pub fn new() -> Self {
        let prefs = storage::load_preferences().unwrap_or_default();
        let resume = storage::load_resume().unwrap_or(None);
        let last_results = storage::load_last_results().unwrap_or_default();
        let scan_history = storage::load_scan_history().unwrap_or_default();

        let mut matcher = Matcher::new();
        if let Some(r) = &resume {
            matcher.load_resume(r.clone());
            eprintln!("  + Auto-loaded resume from storage.");
        }

        let config = SearchConfig {
            keywords: vec![],
            sources: prefs.active_sources.clone(),
            max_results: prefs.max_results,
            location: prefs.preferred_location.clone(),
            ..Default::default()
        };

        Self {
            matcher,
            coordinator: CrawlerCoordinator::new(),
            results: last_results,
            config,
            scan_history,
        }
    }

    // ─── Main Loop ────────────────────────────────────────────────────

    /// Start the interactive menu loop.
    pub async fn run(&mut self) {
        banner();

        loop {
            let cmd = self.prompt_command();
            match cmd {
                Command::Quit => {
                    println!("\n  {}\n", "Later, hunter. Good luck out there.".bright_green());
                    break;
                }
                Command::LoadResume(path) => self.cmd_load_resume(&path),
                Command::ShowResume => self.cmd_show_resume(),
                Command::Scan => self.cmd_scan().await,
                Command::Search(query) => self.cmd_search(&query).await,
                Command::ViewResults => self.cmd_view_results(),
                Command::FilterResults => self.cmd_filter_results(),
            }
        }
    }

    // ─── Non-interactive commands ─────────────────────────────────────

    /// Run a scan using keywords derived from the loaded resume.
    pub async fn run_scan(&mut self) {
        self.prepare_keywords();
        self.cmd_scan().await;
        if !self.results.is_empty() {
            self.show_results();
        }
    }

    /// Search with a custom query string (space-separated keywords).
    pub async fn run_search(&mut self, query: &str) {
        self.config.keywords = query.split_whitespace().map(|s| s.to_string()).collect();
        self.cmd_search(query).await;
        if !self.results.is_empty() {
            self.show_results();
        }
    }

    /// Load a resume from a file path.
    pub fn load_resume_file(&mut self, path: &str) {
        self.cmd_load_resume(path);
    }

    /// Print a summary of cached results to stdout.
    pub fn show_results(&self) {
        if self.results.is_empty() {
            println!("  No results found.");
            return;
        }
        println!("  {} results\n", self.results.len());
        for (i, r) in self.results.iter().enumerate().take(10) {
            let score = format!("{:.0}%", r.score * 100.0);
            let score_colored = if r.score >= 0.7 {
                score.green()
            } else if r.score >= 0.4 {
                score.yellow()
            } else {
                score.dimmed()
            };
            let company = r
                .job
                .company
                .as_deref()
                .map(|c| format!(" @ {}", c.cyan()))
                .unwrap_or_default();
            println!(
                "  {:>2}. {} {} [{}]{}",
                i + 1,
                r.job.title.bright_white(),
                score_colored,
                r.job.source,
                company,
            );
            println!("      {}", r.job.url.dimmed());
        }
        if self.results.len() > 10 {
            println!("  ... and {} more", self.results.len() - 10);
        }
        println!();
    }

    // ─── Menu ─────────────────────────────────────────────────────────

    /// Show the main menu and return the user's chosen command.
    fn prompt_command(&self) -> Command {
        let resume_status = if self.matcher.has_resume() {
            "loaded".green().to_string()
        } else {
            "not loaded".yellow().to_string()
        };

        let result_count = if self.results.is_empty() {
            "no results".dimmed().to_string()
        } else {
            format!("{} results", self.results.len()).cyan().to_string()
        };

        let items = vec![
            format!("Scan jobs (all sources)"),
            format!("Search with custom query"),
            format!("View results ({result_count})"),
            format!("Load resume ({resume_status})"),
            format!("Show current resume"),
            format!("Filter / sort results"),
            format!("Scan history"),
            format!("Quit"),
        ];

        let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("jobense-parker")
            .items(&items)
            .default(0)
            .interact_opt()
            .unwrap_or(Some(7))
            .unwrap_or(7);

        match selection {
            0 => Command::Scan,
            1 => {
                let query: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Search query")
                    .interact_text()
                    .unwrap_or_default();
                Command::Search(query)
            }
            2 => Command::ViewResults,
            3 => {
                match pick_resume_file() {
                    Some(p) => Command::LoadResume(p),
                    None => {
                        println!("  Cancelled.");
                        Command::ShowResume // no-op
                    }
                }
            }
            4 => Command::ShowResume,
            5 => Command::FilterResults,
            6 => {
                show_scan_history(&self.scan_history);
                Command::ShowResume // no-op
            }
            7 | _ => Command::Quit,
        }
    }

    // ─── Command: Load Resume ─────────────────────────────────────────

    /// Handle the "Load Resume" command.
    ///
    /// Determines whether `input` is a file or raw text by checking if
    /// the path exists on disk. PDF files are extracted with `pdf_extract`;
    /// JSON/YAML files are deserialised; all others fall back to
    /// [`Resume::from_text`].
    fn cmd_load_resume(&mut self, input: &str) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("  No text provided.");
            return;
        }

        let path = std::path::Path::new(trimmed);
        let path_exists = path.exists();

        let resume = if path_exists && trimmed.ends_with(".pdf") {
            // PDF file — extract text from it
            match pdf_extract::extract_text(trimmed) {
                Ok(pdf_text) => {
                    println!("  Extracted {} chars from PDF.", pdf_text.len());
                    Resume::from_text(&pdf_text)
                }
                Err(e) => {
                    println!("  Failed to read PDF: {e}");
                    Resume::from_text(trimmed)
                }
            }
        } else if path_exists {
            // Existing file — read and try to parse as JSON/YAML, fall back to plain text
            match std::fs::read_to_string(trimmed) {
                Ok(content) => {
                    serde_json::from_str::<Resume>(&content)
                        .or_else(|_| serde_yaml::from_str::<Resume>(&content))
                        .unwrap_or_else(|_| {
                            println!("  Read file as plain text ({} chars).", content.len());
                            Resume::from_text(&content)
                        })
                }
                Err(e) => {
                    println!("  Could not read file '{trimmed}': {e}");
                    Resume::from_text(trimmed)
                }
            }
        } else {
            // Not a file — treat as raw text (JSON, YAML, or plain text)
            serde_json::from_str::<Resume>(trimmed)
                .or_else(|_| serde_yaml::from_str::<Resume>(trimmed))
                .unwrap_or_else(|_| Resume::from_text(trimmed))
        };

        self.matcher.load_resume(resume.clone());
        // Persist resume immediately
        if let Err(e) = storage::save_resume(&resume) {
            eprintln!("  Warning: failed to persist resume: {e}");
        }

        println!(
            "  Resume loaded! ({} skills, {} roles)",
            self.matcher.resume().map_or(0, |r| r.skills.len()),
            self.matcher.resume().map_or(0, |r| r.role_titles.len()),
        );
    }

    // ─── Command: Show Resume ─────────────────────────────────────────

    /// Display the currently loaded resume to the user.
    fn cmd_show_resume(&self) {
        match self.matcher.resume() {
            None => println!("  No resume loaded."),
            Some(r) => views::show_resume(r),
        }
    }

    // ─── Command: Scan ────────────────────────────────────────────────

    /// Prepare search keywords from the loaded resume (skills + roles).
    /// If no resume is loaded and keywords are empty, this is a no-op
    /// (the scan will return zero results unless the user provides keywords).
    fn prepare_keywords(&mut self) {
        if !self.matcher.has_resume() {
            println!("  No resume loaded. Search keywords must be provided manually.");
            return;
        }

        if let Some(r) = self.matcher.resume() {
            let mut kws = r.skills.clone();
            kws.extend(r.role_titles.clone());
            if !kws.is_empty() {
                self.config.keywords = kws;
            }
        }
    }

    /// Execute a scan against all sources with the current config.
    async fn cmd_scan(&mut self) {
        self.prepare_keywords();

        if self.config.keywords.is_empty() {
            println!("\n  No keywords available. Load a resume or use --search \"your keywords\".\n");
            return;
        }

        println!(
            "\n  Scanning with keywords: {}\n",
            self.config.keywords.iter().map(|k| k.green().to_string()).collect::<Vec<_>>().join(", ")
        );

        let jobs = self.coordinator.crawl_all(&self.config).await;
        let raw_count = jobs.len();

        if jobs.is_empty() {
            println!("\n  No jobs found. Try different keywords or sources.\n");
            return;
        }

        println!(
            "\n  Found {} raw job posts. Matching against resume...",
            raw_count
        );

        if self.matcher.has_resume() {
            self.results = self.matcher.score_all(&jobs);
            println!("  {} matched results (above threshold)\n", self.results.len());
        } else {
            self.results = jobs
                .into_iter()
                .map(|j| MatchResult {
                    score: 0.5,
                    matched_skills: vec![],
                    matched_keywords: vec![],
                    missing_skills: vec![],
                    job: j,
                })
                .collect();
            println!("  {} raw results (no resume to match)\n", self.results.len());
        }

        // Persist results
        let _ = storage::save_last_results(&self.results);

        // Record scan in history (impact profile)
        let top_score = self.results.iter().map(|r| r.score).fold(0.0, f64::max);
        let record = ScanRecord {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            query: self.config.keywords.join(" "),
            source_count: self.config.sources.len(),
            total_jobs_found: raw_count,
            top_score,
            result_count: self.results.len(),
        };
        self.scan_history.insert(0, record.clone());
        if self.scan_history.len() > 100 {
            self.scan_history.truncate(100);
        }
        let _ = storage::push_scan_record(&record);

        self.show_results();
    }

    // ─── Command: Search ──────────────────────────────────────────────

    /// Execute a search with a user-supplied query string.
    async fn cmd_search(&mut self, query: &str) {
        if query.trim().is_empty() {
            println!("  Empty query, cancelling.");
            return;
        }

        self.config.keywords = query.split_whitespace().map(|s| s.to_string()).collect();

        println!("\n  Searching for: {}\n", query.bright_white());

        let jobs = self.coordinator.crawl_all(&self.config).await;

        if jobs.is_empty() {
            println!("\n  No jobs found.\n");
            return;
        }

        if self.matcher.has_resume() {
            self.results = self.matcher.score_all(&jobs);
        } else {
            self.results = jobs
                .into_iter()
                .map(|j| MatchResult {
                    score: 0.5,
                    matched_skills: vec![],
                    matched_keywords: vec![],
                    missing_skills: vec![],
                    job: j,
                })
                .collect();
        }

        // Save query to history
        let _ = storage::push_query(query);
        // Persist results
        let _ = storage::save_last_results(&self.results);

        // Record scan
        let top_score = self.results.iter().map(|r| r.score).fold(0.0, f64::max);
        let record = ScanRecord {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            query: query.to_string(),
            source_count: self.config.sources.len(),
            total_jobs_found: self.results.len(),
            top_score,
            result_count: self.results.len(),
        };
        self.scan_history.insert(0, record.clone());
        if self.scan_history.len() > 100 {
            self.scan_history.truncate(100);
        }
        let _ = storage::push_scan_record(&record);

        println!("\n  Found {} results\n", self.results.len());

        self.show_results();
    }

    // ─── Command: View Results ────────────────────────────────────────

    /// Paginated viewer for the current match results.
    fn cmd_view_results(&self) {
        if self.results.is_empty() {
            println!("  No results yet. Run a scan or search first.");
            return;
        }

        let page_size = 10;
        let total_pages = (self.results.len() + page_size - 1) / page_size;
        let mut page = 0usize;

        loop {
            views::show_results_page(&self.results, page, total_pages);

            let has_prev = page > 0;
            let has_next = page < total_pages - 1;
            let show_jump = total_pages > 3;

            let mut nav = Vec::new();
            if has_prev { nav.push("<- Previous page"); }
            if has_next { nav.push("Next page ->"); }
            let jump_label = if show_jump { Some(format!("Jump to page (1-{total_pages})")) } else { None };
            if let Some(ref jl) = jump_label { nav.push(jl); }
            nav.push("Back to main menu");

            // Map selection index to action
            let sel = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Navigate")
                .items(&nav)
                .default(0)
                .interact_opt()
                .unwrap_or(Some(nav.len() - 1))
                .unwrap_or(nav.len() - 1);

            let n_prev = has_prev as usize;
            let n_next = has_next as usize;
            let n_jump = show_jump as usize;

            // Actions are in this order: [prev?] [next?] [jump?] [back]
            match sel {
                i if n_prev > 0 && i == 0 => page -= 1,
                i if n_next > 0 && i == n_prev => page += 1,
                i if n_jump > 0 && i == n_prev + n_next => { page = Self::jump_to_page(total_pages); }
                _ => break,
            }
        }
    }

    /// Prompt user for a page number and update `page` in `cmd_view_results`.
    fn jump_to_page(total_pages: usize) -> usize {
        let input: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Page number")
            .interact_text()
            .unwrap_or_default();
        input.parse::<usize>().ok()
            .filter(|p| *p > 0 && *p <= total_pages)
            .map(|p| p - 1)
            .unwrap_or(0)
    }

    // ─── Command: Filter Results ──────────────────────────────────────

    /// Interactive filter/sort menu for the current results.
    fn cmd_filter_results(&mut self) {
        if self.results.is_empty() {
            println!("  No results to filter.");
            return;
        }

        let items = vec![
            "Sort by score (high -> low)".to_string(),
            "Sort by score (low -> high)".to_string(),
            "Show only high matches (>70%)".to_string(),
            "Show only medium matches (40-70%)".to_string(),
            "Show only low matches (<40%)".to_string(),
            "Reset filters".to_string(),
            "Back".to_string(),
        ];

        let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Filter results")
            .items(&items)
            .default(0)
            .interact_opt()
            .unwrap_or(Some(6))
            .unwrap_or(6);

        match selection {
            0 => {
                self.results
                    .sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
                println!("  Sorted by score (descending).");
            }
            1 => {
                self.results
                    .sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
                println!("  Sorted by score (ascending).");
            }
            2 => {
                self.results.retain(|r| r.score >= 0.7);
                println!("  Filtered to {} high-match results.", self.results.len());
            }
            3 => {
                self.results.retain(|r| r.score >= 0.4 && r.score < 0.7);
                println!("  Filtered to {} medium-match results.", self.results.len());
            }
            4 => {
                self.results.retain(|r| r.score < 0.4);
                println!("  Filtered to {} low-match results.", self.results.len());
            }
            5 => {
                println!("  Re-run a scan to get fresh results.");
            }
            _ => {}
        }
    }
}

// ─── File Picker ─────────────────────────────────────────────────────────

/// Interactive file-picker dialog for selecting a resume file.
///
/// Navigate directories with arrow keys and fuzzy-find. Supports
/// PDF, JSON, YAML, and TXT extensions.
fn pick_resume_file() -> Option<String> {
    let mut current_dir = std::env::current_dir().ok()?;

    loop {
        let mut entries: Vec<(String, bool, std::path::PathBuf)> = Vec::new();

        if let Some(parent) = current_dir.parent() {
            entries.push(("..".to_string(), true, parent.to_path_buf()));
        }

        let dir_iter = match std::fs::read_dir(&current_dir) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("  Cannot read directory.");
                return None;
            }
        };

        for entry in dir_iter.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                entries.push((name, true, path));
            } else {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase())
                    .unwrap_or_default();
                if matches!(ext.as_str(), "pdf" | "json" | "yaml" | "yml" | "txt") {
                    entries.push((name, false, path));
                }
            }
        }

        if entries.is_empty() {
            eprintln!("  No compatible files in this directory.");
            return None;
        }

        entries.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase()))
        });

        let display_items: Vec<String> = entries
            .iter()
            .map(|(name, is_dir, _)| {
                if *is_dir {
                    format!("  [DIR] {name}/")
                } else {
                    format!("  [FILE] {name}")
                }
            })
            .collect();

        let prompt = format!(
            "Select resume (in {})",
            current_dir.display().to_string().dimmed()
        );

        let selection = FuzzySelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(&prompt)
            .items(&display_items)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            None => return None,
            Some(idx) => {
                let (_, is_dir, path) = &entries[idx];
                if *is_dir {
                    current_dir = path.clone();
                } else {
                    let path_str = path.to_string_lossy().to_string();
                    println!("  Selected: {}", path_str.dimmed());
                    return Some(path_str);
                }
            }
        }
    }
}
