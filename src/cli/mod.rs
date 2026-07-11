//! CLI application logic — interactive menu and non-interactive command dispatch.

mod views;

use colored::Colorize;
use dialoguer::{FuzzySelect, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use uuid::Uuid;

use crate::crawler::company::CompanyCrawler;
use crate::crawler::CrawlerCoordinator;
use crate::matcher::Matcher;
use crate::models::{Command, CompanyDatabase, JobPost, MatchResult, Resume, ScanRecord, SearchConfig};
use crate::storage;

pub use views::{banner, print_help, show_scan_history};

/// Main application struct that ties together the matcher, crawlers, and storage.
pub struct App {
    matcher: Matcher,
    coordinator: CrawlerCoordinator,
    results: Vec<MatchResult>,
    config: SearchConfig,
    scan_history: Vec<ScanRecord>,
    /// Local cache of known companies with careers-page URLs.
    company_db: CompanyDatabase,
}

impl App {
    /// Create a new app instance, loading persisted state (resume, results, history).
    pub fn new() -> Self {
        let prefs = storage::load_preferences().unwrap_or_default();
        let resume = storage::load_resume().unwrap_or(None);
        let last_results = storage::load_last_results().unwrap_or_default();
        let scan_history = storage::load_scan_history().unwrap_or_default();
        let company_db = storage::load_company_database().unwrap_or_default();

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
            company_db,
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
                Command::ListCompanies => self.cmd_list_companies(),
                Command::AddCompany(name, url) => self.cmd_add_company(&name, &url),
                Command::RemoveCompany(name) => self.cmd_remove_company(&name),
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

    /// Print all cached companies (used by --companies flag).
    pub fn show_companies(&self) {
        views::show_companies_list(&self.company_db);
    }

    /// Add a company from CLI args (used by --add-company flag).
    pub fn add_company_cli(&mut self, name: &str, url: &str) {
        self.cmd_add_company(name, url);
    }

    /// Remove a company from CLI args (used by --remove-company flag).
    pub fn remove_company_cli(&mut self, name: &str) {
        self.cmd_remove_company(name);
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
            println!("      {}", views::clickable(&r.job.url, &r.job.url).dimmed());
        }
        if self.results.len() > 10 {
            println!("  ... and {} more", self.results.len() - 10);
        }
        println!("  Use 'View results' for full paginated browser with j/k navigation.");
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

        let company_count = self.company_db.companies.len();
        let company_status = format!("{} companies cached", company_count).cyan().to_string();

        let items = vec![
            format!("Scan jobs (all sources + career sites)"),
            format!("Search with custom query"),
            format!("View results ({result_count})"),
            format!("Company career sites ({company_status})"),
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
            .unwrap_or(Some(8))
            .unwrap_or(8);

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
                // Company management sub-menu
                self.cmd_list_companies();
                println!("  Add a company? Enter name and careers URL, or just press Enter to skip.");
                let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Company name (or blank to skip)")
                    .allow_empty(true)
                    .interact_text()
                    .unwrap_or_default();
                if !name.trim().is_empty() {
                    let url: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("Careers URL")
                        .interact_text()
                        .unwrap_or_default();
                    if !url.trim().is_empty() {
                        Command::AddCompany(name.trim().to_string(), url.trim().to_string())
                    } else {
                        println!("  No URL given. Skipping.");
                        Command::ShowResume // no-op
                    }
                } else {
                    Command::ShowResume // no-op
                }
            }
            4 => {
                match pick_resume_file() {
                    Some(p) => Command::LoadResume(p),
                    None => {
                        println!("  Cancelled.");
                        Command::ShowResume // no-op
                    }
                }
            }
            5 => Command::ShowResume,
            6 => Command::FilterResults,
            7 => {
                show_scan_history(&self.scan_history);
                Command::ShowResume // no-op
            }
            8 | _ => Command::Quit,
        }
    }

    // ─── Command: Load Resume ─────────────────────────────────────────

    /// Handle the "Load Resume" command.
    fn cmd_load_resume(&mut self, input: &str) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("  No text provided.");
            return;
        }

        let path = std::path::Path::new(trimmed);
        let path_exists = path.exists();

        let resume = if path_exists && trimmed.ends_with(".pdf") {
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
            serde_json::from_str::<Resume>(trimmed)
                .or_else(|_| serde_yaml::from_str::<Resume>(trimmed))
                .unwrap_or_else(|_| Resume::from_text(trimmed))
        };

        self.matcher.load_resume(resume.clone());
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

    fn cmd_show_resume(&self) {
        match self.matcher.resume() {
            None => println!("  No resume loaded."),
            Some(r) => views::show_resume(r),
        }
    }

    // ─── Command: Scan ────────────────────────────────────────────────

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

        let kw = self.config.keywords.clone();
        self.run_with_spinner("Scanning", &kw, false).await;
    }

    // ─── Command: Search ──────────────────────────────────────────────

    /// Execute a search with a user-supplied query string.
    async fn cmd_search(&mut self, query: &str) {
        if query.trim().is_empty() {
            println!("  Empty query, cancelling.");
            return;
        }

        self.config.keywords = query.split_whitespace().map(|s| s.to_string()).collect();
        let kw = self.config.keywords.clone();
        self.run_with_spinner("Searching", &kw, true).await;
    }

    // ─── Shared crawl + spinner logic ─────────────────────────────────

    /// Run a crawl with a progress spinner showing status in real-time.
    /// Also crawls company career sites and auto-discovers new companies.
    async fn run_with_spinner(&mut self, action: &str, keywords: &[String], save_query: bool) {
        let kw_display = keywords
            .iter()
            .map(|k| k.green().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        // ── Phase 1: Job board crawl ─────────────────────────────────
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(format!(
            "{} jobs for: {} (Remote OK, Reddit, Hacker News)...",
            action, kw_display
        ));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let mut jobs = self.coordinator.crawl_all(&self.config).await;
        pb.finish_and_clear();

        // ── Auto-discover companies from job posts ───────────────────
        let discovered = self.auto_discover_companies(&jobs);
        if discovered > 0 {
            eprintln!(
                "  {} Auto-discovered {} new {}",
                "🔍".to_string(),
                discovered,
                if discovered == 1 { "company" } else { "companies" }
            );
        }

        // ── Phase 2: Company career site crawl ───────────────────────
        if !self.company_db.companies.is_empty() {
            let pb2 = ProgressBar::new_spinner();
            pb2.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            pb2.set_message(format!(
                "Crawling {} company career sites...",
                self.company_db.companies.len()
            ));
            pb2.enable_steady_tick(std::time::Duration::from_millis(100));

            let company_jobs = CompanyCrawler::crawl_all(&mut self.company_db, &self.config).await;
            pb2.finish_and_clear();

            if !company_jobs.is_empty() {
                eprintln!(
                    "  {} {} jobs from company career sites",
                    "+".green(),
                    company_jobs.len()
                );
                jobs.extend(company_jobs);
            }

            // Persist updated company DB (with crawl timestamps)
            let _ = storage::save_company_database(&self.company_db);
        }

        // ── Process results ──────────────────────────────────────────
        let raw_count = jobs.len();

        if jobs.is_empty() {
            println!("\n  No jobs found. Try different keywords or sources.\n");
            return;
        }

        println!(
            "  {} {} raw job posts. Matching against resume...",
            "⚡".bright_green(),
            raw_count
        );

        if self.matcher.has_resume() {
            self.results = self.matcher.score_all(&jobs);
        } else {
            // Score by keyword relevance even without a resume.
            // This gives meaningful ranking: jobs mentioning more keywords
            // in their title get a higher score.
            let kw_lower: Vec<String> = self.config.keywords.iter().map(|k| k.to_lowercase()).collect();
            let query_phrase = kw_lower.join(" ");

            self.results = jobs
                .into_iter()
                .map(|j| {
                    let title_lower = j.title.to_lowercase();
                    let desc_lower = j.description.to_lowercase();

                    // Count keyword matches in title (weighted 3x)
                    let title_matches: usize = kw_lower.iter().filter(|kw| title_lower.contains(kw.as_str())).count();
                    // Count keyword matches in description
                    let desc_matches: usize = kw_lower.iter().filter(|kw| desc_lower.contains(kw.as_str())).count();
                    // Exact phrase match in title (big bonus)
                    let phrase_bonus = if title_lower.contains(&query_phrase) { 2.0 } else { 0.0 };

                    let score = if kw_lower.is_empty() {
                        0.5
                    } else {
                        let max_kw = kw_lower.len() as f64;
                        let raw = (title_matches as f64 * 3.0 + desc_matches as f64 * 1.0) / (max_kw * 3.0 + max_kw) + phrase_bonus * 0.1;
                        raw.clamp(0.05, 0.99)
                    };

                    MatchResult {
                        score,
                        matched_skills: vec![],
                        matched_keywords: kw_lower.iter().filter(|kw| title_lower.contains(kw.as_str()) || desc_lower.contains(kw.as_str())).cloned().collect(),
                        missing_skills: vec![],
                        job: j,
                    }
                })
                .collect();
        }

        // Save query to history
        if save_query {
            let _ = storage::push_query(&keywords.join(" "));
        }
        // Persist results
        let _ = storage::save_last_results(&self.results);

        // Record scan in history
        let top_score = self.results.iter().map(|r| r.score).fold(0.0, f64::max);
        let record = ScanRecord {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            query: keywords.join(" "),
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

        // Show top results immediately
        if !self.results.is_empty() {
            println!(
                "  {} {} matched results (top score: {:.0}%)\n",
                "✓".bright_green(),
                self.results.len(),
                top_score * 100.0
            );
            self.show_results();
        } else {
            println!("  No matches above threshold.\n");
        }
    }

    // ─── Company Management ─────────────────────────────────────────────

    /// Extract company names from job posts and add them to the local cache.
    /// Returns the number of newly discovered companies.
    fn auto_discover_companies(&mut self, jobs: &[JobPost]) -> usize {
        let mut count = 0usize;

        // If there are already 100+ companies, skip auto-discovery to avoid bloat.
        if self.company_db.companies.len() >= 100 {
            return 0;
        }

        for job in jobs {
            if let Some(ref name) = job.company {
                // Skip very short or generic names
                let trimmed = name.trim();
                if trimmed.len() < 2 {
                    continue;
                }
                // Skip generic company-like words that aren't actual companies
                let generic = [
                    "remote", "inc", "llc", "corp", "ltd", "gmbh", "co", "company",
                    "startup", "client", "company name", "confidential", "private",
                ];
                if generic.iter().any(|g| trimmed.eq_ignore_ascii_case(g)) {
                    continue;
                }
                // Skip if already in DB
                if self.company_db.companies.iter().any(|c| c.name.eq_ignore_ascii_case(trimmed)) {
                    continue;
                }

                // Guess the careers URL from the company name
                let url = storage::guess_careers_url(trimmed);
                if url.is_empty() {
                    continue;
                }

                if self.company_db.add(trimmed, &url) {
                    count += 1;
                }
            }
        }

        if count > 0 {
            let _ = storage::save_company_database(&self.company_db);
        }

        count
    }

    /// Show all cached companies in a paginated list.
    fn cmd_list_companies(&self) {
        if self.company_db.companies.is_empty() {
            println!("  No companies cached yet. They are auto-discovered from job posts.");
            return;
        }

        let failed = &self.company_db.failed;
        println!();
        println!(
            "  {} companies in cache ({} failed last crawl)",
            self.company_db.companies.len(),
            failed.len()
        );
        println!("  {}", "─".repeat(60).dimmed());

        for (i, company) in self.company_db.companies.iter().enumerate() {
            let status = match company.last_crawled {
                Some(_) => "✓".green().to_string(),
                None => "—".dimmed().to_string(),
            };
            let fail_note = if failed.contains_key(&company.name) {
                format!(" {}", "⚠ failed".red())
            } else {
                String::new()
            };
            println!(
                "  {:>3}. {} {} {}{}",
                i + 1,
                status,
                company.name.bright_white(),
                company.careers_url.dimmed(),
                fail_note,
            );
        }
        println!();
        println!("  Use menu option 'Company career sites' to add more.");
        println!();
    }

    /// Add a company to the cache.
    fn cmd_add_company(&mut self, name: &str, url: &str) {
        if name.trim().is_empty() || url.trim().is_empty() {
            println!("  Both name and URL are required.");
            return;
        }
        if self.company_db.add(name.trim(), url.trim()) {
            let _ = storage::save_company_database(&self.company_db);
            println!("  Added: {} → {}", name.trim().green(), url.trim().dimmed());
        } else {
            println!("  '{}' is already in the cache.", name);
        }
    }

    /// Remove a company from the cache.
    fn cmd_remove_company(&mut self, name: &str) {
        if self.company_db.remove(name.trim()) {
            let _ = storage::save_company_database(&self.company_db);
            println!("  Removed: {}", name.trim().green());
        } else {
            println!("  '{}' not found in cache.", name);
        }
    }

    // ─── Command: View Results ────────────────────────────────────────

    /// Open the vim-style paginated results browser.
    fn cmd_view_results(&self) {
        if self.results.is_empty() {
            println!("  No results yet. Run a scan or search first.");
            return;
        }

        // Enter raw mode via console Term for the vim-style viewer.
        // The viewer handles its own screen rendering and key reading.
        if let Err(e) = views::run_results_viewer(&self.results) {
            eprintln!("  Viewer error: {e}");
        }
    }

    // ─── Command: Filter Results ──────────────────────────────────────

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
