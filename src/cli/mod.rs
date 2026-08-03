//! CLI application logic — interactive menu and non-interactive command dispatch.

mod views;

use colored::Colorize;
use dialoguer::{FuzzySelect, Input, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use uuid::Uuid;

use crate::crawler::company::CompanyCrawler;
use crate::crawler::CrawlerCoordinator;
use crate::matcher::Matcher;
use crate::models::{
    Command, CompanyDatabase, JobPost, JobSource, MatchResult, Resume, ScanRecord, SearchConfig,
};
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
    #[allow(clippy::new_without_default)]
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
                Command::Scan => {
                    self.cmd_scan().await;
                    self.cmd_view_results();
                }
                Command::Search(query) => {
                    self.cmd_search(&query).await;
                    self.cmd_view_results();
                }
                Command::ViewResults => self.cmd_view_results(),
                Command::FilterResults => self.cmd_filter_results(),
                Command::ListCompanies => self.cmd_list_companies(),
                Command::ShowScanHistory => {
                    show_scan_history(&self.scan_history);
                }
                Command::SetProfile => {
                    Self::cmd_set_profile();
                }
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
    ///
    /// NOTE: This method ONLY shows the menu and returns a command.
    /// NO side effects — every selection is dispatched by `run()`.
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
        let company_status = format!("{} companies cached", company_count)
            .cyan()
            .to_string();

        let items = vec![
            format!("Scan jobs (all sources + career sites)"),
            format!("Search with custom query"),
            format!("View results ({result_count})"),
            format!("Company career sites ({company_status})"),
            format!("Load resume ({resume_status})"),
            format!("Show current resume"),
            format!("Filter / sort results"),
            format!("Scan history"),
            format!("Set profile (for auto-fill)"),
            format!("Quit"),
        ];

        let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("jobense-parker")
            .items(&items)
            .default(0)
            .interact_opt()
            .unwrap_or(Some(9))
            .unwrap_or(9);

        match selection {
            0 => Command::Scan,
            1 => {
                let query: String =
                    Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                        .with_prompt("Search query")
                        .interact_text()
                        .unwrap_or_default();
                Command::Search(query)
            }
            2 => Command::ViewResults,
            3 => Command::ListCompanies,
            4 => match pick_resume_file() {
                Some(p) => Command::LoadResume(p),
                None => {
                    println!("  Cancelled.");
                    Command::Scan // harmless no-op, goes back to menu
                }
            },
            5 => Command::ShowResume,
            6 => Command::FilterResults,
            7 => Command::ShowScanHistory,
            8 => Command::SetProfile,
            _ => Command::Quit,
        }
    }

    // ─── Command: Load Resume ─────────────────────────────────────────

    /// Parse resume text from a PDF, file (JSON/YAML/text), or raw string.
    fn parse_resume_from_input(input: &str) -> Resume {
        let trimmed = input.trim();
        let path = std::path::Path::new(trimmed);

        // PDF file
        if path.exists() && trimmed.ends_with(".pdf") {
            match pdf_extract::extract_text(trimmed) {
                Ok(pdf_text) => {
                    println!("  Extracted {} chars from PDF.", pdf_text.len());
                    return Resume::from_text(&pdf_text);
                }
                Err(e) => {
                    println!("  Failed to read PDF: {e}");
                    return Resume::from_text(trimmed);
                }
            }
        }

        // Existing file: try JSON → YAML → plain text
        if path.exists() {
            match std::fs::read_to_string(trimmed) {
                Ok(content) => {
                    if let Ok(r) = serde_json::from_str::<Resume>(&content) {
                        return r;
                    }
                    if let Ok(r) = serde_yaml::from_str::<Resume>(&content) {
                        return r;
                    }
                    println!("  Read file as plain text ({} chars).", content.len());
                    return Resume::from_text(&content);
                }
                Err(e) => {
                    println!("  Could not read file '{trimmed}': {e}");
                    return Resume::from_text(trimmed);
                }
            }
        }

        // Not a file: try JSON → YAML → plain text
        serde_json::from_str::<Resume>(trimmed)
            .or_else(|_| serde_yaml::from_str::<Resume>(trimmed))
            .unwrap_or_else(|_| Resume::from_text(trimmed))
    }

    /// Handle the "Load Resume" command.
    fn cmd_load_resume(&mut self, input: &str) {
        if input.trim().is_empty() {
            println!("  No text provided.");
            return;
        }

        let resume = Self::parse_resume_from_input(input);

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

    /// Score jobs by keyword relevance when no resume is loaded.
    /// Keywords in the title are weighted 3x vs description, with an exact-phrase bonus.
    fn score_jobs_by_keywords(&self, jobs: Vec<JobPost>) -> Vec<MatchResult> {
        if self.config.keywords.is_empty() {
            return jobs
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
        let kw_lower: Vec<String> = self.config.keywords.iter().map(|k| k.to_lowercase()).collect();
        let query_phrase = kw_lower.join(" ");
        let max_kw = kw_lower.len() as f64;
        let max_score = max_kw * 3.0 + max_kw;

        jobs.into_iter()
            .map(|j| {
                let title_lower = j.title.to_lowercase();
                let desc_lower = j.description.to_lowercase();

                let title_matches = kw_lower
                    .iter()
                    .filter(|kw| title_lower.contains(kw.as_str()))
                    .count() as f64;
                let desc_matches = kw_lower
                    .iter()
                    .filter(|kw| desc_lower.contains(kw.as_str()))
                    .count() as f64;
                let phrase_bonus = if title_lower.contains(&query_phrase) {
                    2.0
                } else {
                    0.0
                };

                let raw = (title_matches * 3.0 + desc_matches) / max_score + phrase_bonus * 0.1;
                let score = raw.clamp(0.05, 0.99);

                let matched_keywords: Vec<String> = kw_lower
                    .iter()
                    .filter(|kw| title_lower.contains(kw.as_str()) || desc_lower.contains(kw.as_str()))
                    .cloned()
                    .collect();

                MatchResult {
                    score,
                    matched_skills: vec![],
                    matched_keywords,
                    missing_skills: vec![],
                    job: j,
                }
            })
            .collect()
    }

    /// Run a crawl with a progress spinner showing status in real-time.
    /// Also crawls company career sites and auto-discovers new companies.
    async fn run_with_spinner(&mut self, action: &str, keywords: &[String], save_query: bool) {
        let kw_display = keywords
            .iter()
            .map(|k| k.green().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        // ── Run board crawl + company crawl IN PARALLEL ──────────────
        let coordinator = &self.coordinator;
        let config = &self.config;
        let company_db = &mut self.company_db;

        let spinner_msg = format!(
            "{} jobs for: {} (boards + {} company sites)...",
            action,
            kw_display,
            company_db.companies.len()
        );

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(spinner_msg);
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let (board_result, company_result) = tokio::join!(
            tokio::time::timeout(
                std::time::Duration::from_secs(45),
                coordinator.crawl_all(config),
            ),
            tokio::time::timeout(
                std::time::Duration::from_secs(60),
                CompanyCrawler::crawl_all(company_db, config),
            ),
        );

        pb.finish_and_clear();

        // ── Unpack and merge results ────────────────────────────────
        let mut jobs: Vec<JobPost> = match board_result {
            Ok(j) => j,
            Err(_) => {
                eprintln!("  {} Board crawl timed out (45s).", "!".yellow());
                vec![]
            }
        };
        let company_jobs: Vec<JobPost> = match company_result {
            Ok(j) => j,
            Err(_) => {
                eprintln!("  {} Company crawl timed out (60s).", "!".yellow());
                vec![]
            }
        };
        if !company_jobs.is_empty() {
            jobs.extend(company_jobs);
        }
        let _ = storage::save_company_database(&self.company_db);

        if jobs.is_empty() {
            println!("\n  No jobs found. Try different keywords or sources.\n");
            self.results.clear();
            return;
        }

        // ── Auto-discover companies from job posts ──────────────────
        let discovered = self.auto_discover_companies(&jobs);
        if discovered > 0 {
            eprintln!(
                "  {} Auto-discovered {} new {}",
                "+".green(),
                discovered,
                if discovered == 1 { "company" } else { "companies" }
            );
        }

        // ── Score all jobs ──────────────────────────────────────────
        let raw_count = jobs.len();

        // Re-activate spinner so user sees progress during matching
        pb.reset();
        pb.set_message(format!("Scoring {} jobs against resume...", raw_count));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        self.results = if self.matcher.has_resume() {
            self.matcher.score_all(&jobs)
        } else {
            self.score_jobs_by_keywords(jobs)
        };

        pb.finish_and_clear();

        // ── Sort by date DESC (newest first), then by score DESC ────
        sort_by_date_newest(&mut self.results);

        // ── Persist history and results ─────────────────────────────
        if save_query {
            let _ = storage::push_query(&keywords.join(" "));
        }
        let _ = storage::save_last_results(&self.results);
        self.record_scan(keywords, raw_count);

        // ── Show summary ────────────────────────────────────────────
        let top_score = self.results.iter().map(|r| r.score).fold(0.0, f64::max);
        if !self.results.is_empty() {
            println!(
                "  {} {} matched results (top score: {:.0}%) — opening viewer...\n",
                "\u{2713}".bright_green(),
                self.results.len(),
                top_score * 100.0
            );
        } else {
            println!("  No matches above threshold.\n");
        }
    }

    /// Create a ScanRecord for this run, prepend it to in-memory history, and persist.
    fn record_scan(&mut self, keywords: &[String], raw_count: usize) {
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
        self.scan_history.truncate(100);
        let _ = storage::push_scan_record(&record);
    }

    // ─── Company Management ─────────────────────────────────────────────

    /// Extract company names from job posts and add them to the local cache.
    /// Returns the number of newly discovered companies.
    fn auto_discover_companies(&mut self, jobs: &[JobPost]) -> usize {
        if self.company_db.companies.len() >= 100 {
            return 0; // cap auto-discovery to avoid bloat
        }
        let generic: &[&str] = &[
            "remote", "inc", "llc", "corp", "ltd", "gmbh", "co", "company", "startup", "client",
            "company name", "confidential", "private",
        ];

        let mut count = 0usize;
        for job in jobs {
            let Some(ref name) = job.company else {
                continue;
            };
            let trimmed = name.trim();
            if trimmed.len() < 2 {
                continue;
            }
            if generic.iter().any(|g| trimmed.eq_ignore_ascii_case(g)) {
                continue;
            }
            if self
                .company_db
                .companies
                .iter()
                .any(|c| c.name.eq_ignore_ascii_case(trimmed))
            {
                continue;
            }
            let url = storage::guess_careers_url(trimmed);
            if url.is_empty() {
                continue;
            }
            if self.company_db.add(trimmed, &url) {
                count += 1;
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
        println!("  {}", "\u{2500}".repeat(60).dimmed());

        for (i, company) in self.company_db.companies.iter().enumerate() {
            let status = match company.last_crawled {
                Some(_) => "\u{2713}".green().to_string(),
                None => "\u{2014}".dimmed().to_string(),
            };
            let fail_note = if failed.contains_key(&company.name) {
                format!(" {}", "\u{26A0} failed".red())
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
            println!("  Added: {} \u{2192} {}", name.trim().green(), url.trim().dimmed());
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

    // ─── Command: Set Profile ─────────────────────────────────────────

    /// Set or update your personal profile for auto-filling job applications.
    fn cmd_set_profile() {
        let mut prefs = storage::load_preferences().unwrap_or_default();

        println!();
        println!("  {}", "╔══════════════════════════════════════════════════╗".bright_blue());
        println!("  {}  Profile Setup (for auto-fill)                {}", "║".bright_blue(), "║".bright_blue());
        println!("  {}", "╚══════════════════════════════════════════════════╝".bright_blue());
        println!();

        let name: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Full name")
            .default(prefs.full_name.unwrap_or_default())
            .interact_text()
            .unwrap_or_default();
        prefs.full_name = if name.is_empty() { None } else { Some(name) };

        let email: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Email")
            .default(prefs.email.unwrap_or_default())
            .interact_text()
            .unwrap_or_default();
        prefs.email = if email.is_empty() { None } else { Some(email) };

        let phone: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Phone (optional)")
            .default(prefs.phone.unwrap_or_default())
            .interact_text()
            .unwrap_or_default();
        prefs.phone = if phone.is_empty() { None } else { Some(phone) };

        let location: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Location (optional)")
            .default(prefs.preferred_location.unwrap_or_default())
            .interact_text()
            .unwrap_or_default();
        prefs.preferred_location = if location.is_empty() { None } else { Some(location) };

        let linkedin: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("LinkedIn URL (optional)")
            .default(prefs.linkedin_url.unwrap_or_default())
            .interact_text()
            .unwrap_or_default();
        prefs.linkedin_url = if linkedin.is_empty() { None } else { Some(linkedin) };

        let github: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("GitHub/Portfolio URL (optional)")
            .default(prefs.github_url.unwrap_or_default())
            .interact_text()
            .unwrap_or_default();
        prefs.github_url = if github.is_empty() { None } else { Some(github) };

        if let Err(e) = storage::save_preferences(&prefs) {
            eprintln!("  {} Failed to save profile: {}", "!".red(), e);
        } else {
            println!();
            println!("  {} Profile saved to ~/.jobsense-parker/preferences.json", "✓".green());
            println!("  {} Press 'a' on any job to auto-fill the application form.", "→".cyan());
            println!();
        }
    }

    // ─── Command: View Results ────────────────────────────────────────

    /// Open the vim-style paginated results browser.
    fn cmd_view_results(&self) {
        if self.results.is_empty() {
            println!("  No results yet. Run a scan or search first.");
            return;
        }
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
            "Sort by score (high → low)".to_string(),
            "Sort by score (low → high)".to_string(),
            "Sort by date (newest first)".to_string(),
            "Sort by date (oldest first)".to_string(),
            "Filter: only Remote OK".to_string(),
            "Filter: only Reddit".to_string(),
            "Filter: only Hacker News".to_string(),
            "Filter: only Company career sites".to_string(),
            "Score: only high (>70%)".to_string(),
            "Score: only medium (40-70%)".to_string(),
            "Score: only low (<40%)".to_string(),
            "Filter by country".to_string(),
            "Reset all filters".to_string(),
            "Back".to_string(),
        ];

        let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Filter / sort results")
            .items(&items)
            .default(0)
            .interact_opt()
            .unwrap_or(Some(items.len() - 1))
            .unwrap_or(items.len() - 1);

        match selection {
            0 => {
                sort_by_score_desc(&mut self.results);
                println!("  ✓ Sorted by score (high → low).");
            }
            1 => {
                self.results
                    .sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
                println!("  ✓ Sorted by score (low → high).");
            }
            2 => {
                sort_by_date_newest(&mut self.results);
                println!("  ✓ Sorted by date (newest first).");
            }
            3 => {
                sort_by_date_oldest(&mut self.results);
                println!("  ✓ Sorted by date (oldest first).");
            }
            4 => {
                let before = self.results.len();
                self.results.retain(|r| matches!(r.job.source, JobSource::RemoteOk));
                println!("  ✓ Filtered to {} Remote OK results (was {}).", self.results.len(), before);
            }
            5 => {
                let before = self.results.len();
                self.results.retain(|r| matches!(r.job.source, JobSource::Reddit));
                println!("  ✓ Filtered to {} Reddit results (was {}).", self.results.len(), before);
            }
            6 => {
                let before = self.results.len();
                self.results.retain(|r| matches!(r.job.source, JobSource::HackerNews));
                println!("  ✓ Filtered to {} HN results (was {}).", self.results.len(), before);
            }
            7 => {
                let before = self.results.len();
                self.results.retain(|r| matches!(r.job.source, JobSource::Custom(_)));
                println!(
                    "  ✓ Filtered to {} company career-site results (was {}).",
                    self.results.len(),
                    before
                );
            }
            8 => {
                self.results.retain(|r| r.score >= 0.7);
                println!("  ✓ Filtered to {} high-match results (>70%).", self.results.len());
            }
            9 => {
                self.results.retain(|r| r.score >= 0.4 && r.score < 0.7);
                println!("  ✓ Filtered to {} medium-match results (40-70%).", self.results.len());
            }
            10 => {
                self.results.retain(|r| r.score < 0.4);
                println!("  ✓ Filtered to {} low-match results (<40%).", self.results.len());
            }
            11 => {
                let countries = collect_countries(&self.results);
                if countries.is_empty() {
                    println!("  No location data available to filter by country.");
                    return;
                }
                // All countries pre-selected by default
                let defaults: Vec<bool> = std::iter::repeat(true).take(countries.len()).collect();
                let selections = MultiSelect::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Filter by country (Space to toggle, Enter to confirm)")
                    .items(&countries)
                    .defaults(&defaults)
                    .interact_opt()
                    .unwrap_or(None);

                match selections {
                    Some(selected) => {
                        let before = self.results.len();
                        let keep: std::collections::HashSet<&str> = selected
                            .iter()
                            .map(|&i| countries[i].as_str())
                            .collect();
                        self.results.retain(|r| {
                            let c = infer_country(r.job.location.as_deref());
                            keep.contains(c.as_str())
                        });
                        println!(
                            "  ✓ Filtered by country ({} countries selected, results: {} → {}).",
                            selected.len(),
                            before,
                            self.results.len()
                        );
                    }
                    None => {
                        println!("  Cancelled.");
                    }
                }
            }
            12 => match storage::load_last_results() {
                Ok(saved) => {
                    let count_before = self.results.len();
                    self.results = saved;
                    println!("  ✓ Reset filters. Back to {} results (was {}).", self.results.len(), count_before);
                }
                Err(_) => {
                    println!("  No cached results to restore. Re-run a scan.");
                }
            },
            _ => {}
        }
    }
}

// ─── Free-standing sort helpers ────────────────────────────────────────

/// Sort `MatchResult`s by score descending, then by posted_at descending as tiebreaker.
fn sort_by_score_desc(results: &mut [MatchResult]) {
    results.sort_by(|a, b| {
        let score_cmp = b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal);
        if score_cmp != std::cmp::Ordering::Equal {
            return score_cmp;
        }
        match (b.job.posted_at, a.job.posted_at) {
            (Some(b_date), Some(a_date)) => b_date.cmp(&a_date),
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => b.job.crawled_at.cmp(&a.job.crawled_at),
        }
    });
}

/// Sort by posted_at descending (newest first), falling back to crawled_at.
fn sort_by_date_newest(results: &mut [MatchResult]) {
    results.sort_by(|a, b| {
        match (b.job.posted_at, a.job.posted_at) {
            (Some(bd), Some(ad)) => bd.cmp(&ad),
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => b.job.crawled_at.cmp(&a.job.crawled_at),
        }
    });
}

/// Sort by posted_at ascending (oldest first), falling back to crawled_at.
fn sort_by_date_oldest(results: &mut [MatchResult]) {
    results.sort_by(|a, b| {
        match (a.job.posted_at, b.job.posted_at) {
            (Some(ad), Some(bd)) => ad.cmp(&bd),
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => a.job.crawled_at.cmp(&b.job.crawled_at),
        }
    });
}

// ─── Location / Country helpers ────────────────────────────────────────

/// Infer a country name from a job's location field.
/// Falls back to "Unknown" when nothing can be determined.
fn infer_country(location: Option<&str>) -> String {
    let loc = match location {
        Some(l) => l.trim(),
        None => return "Unknown".into(),
    };
    if loc.is_empty() || loc.eq_ignore_ascii_case("remote") || loc.eq_ignore_ascii_case("anywhere")
    {
        return "Remote".into();
    }

    // Check for explicit country names in location text
    let country_names: &[(&str, &[&str])] = &[
        ("United States", &["united states", "usa", "u.s.a.", "america"][..]),
        ("United Kingdom", &["united kingdom", "uk", "u.k.", "england", "scotland", "wales", "britain"][..]),
        ("Canada", &["canada"][..]),
        ("Australia", &["australia"][..]),
        ("Germany", &["germany", "deutschland"][..]),
        ("France", &["france"][..]),
        ("India", &["india"][..]),
        ("Japan", &["japan"][..]),
        ("Singapore", &["singapore"][..]),
        ("China", &["china"][..]),
        ("South Korea", &["south korea", "korea"][..]),
        ("Netherlands", &["netherlands", "holland"][..]),
        ("Switzerland", &["switzerland"][..]),
        ("Sweden", &["sweden"][..]),
        ("Denmark", &["denmark"][..]),
        ("Norway", &["norway"][..]),
        ("Finland", &["finland"][..]),
        ("Spain", &["spain"][..]),
        ("Italy", &["italy"][..]),
        ("Brazil", &["brazil"][..]),
        ("Ireland", &["ireland"][..]),
        ("New Zealand", &["new zealand"][..]),
        ("Israel", &["israel"][..]),
        ("Poland", &["poland"][..]),
        ("Russia", &["russia"][..]),
        ("Mexico", &["mexico"][..]),
        ("Argentina", &["argentina"][..]),
    ];
    let loc_lower = loc.to_lowercase();
    for (country, aliases) in country_names {
        if aliases.iter().any(|a| loc_lower.contains(a)) {
            return country.to_string();
        }
    }

    // Check for US state abbreviations (2-letter codes)
    let us_states = [
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA",
        "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
        "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
        "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
        "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY",
        "DC", "PR",
    ];
    // Check for ", XX" pattern at end of location (e.g. "San Francisco, CA")
    for word in loc.split(&[',', ' '][..]).filter(|w| !w.is_empty()) {
        let word_upper = word.to_uppercase();
        if us_states.contains(&word_upper.as_str()) {
            return "United States".into();
        }
    }

    // Check for Canadian province abbreviations
    let ca_provinces = [
        "AB", "BC", "MB", "NB", "NL", "NS", "NT", "NU", "ON", "PE", "QC", "SK", "YT",
    ];
    for word in loc.split(&[',', ' '][..]).filter(|w| !w.is_empty()) {
        let word_upper = word.to_uppercase();
        if ca_provinces.contains(&word_upper.as_str()) {
            return "Canada".into();
        }
    }

    // Fallback
    "Unknown".into()
}

/// Collect unique countries from a slice of match results.
fn collect_countries(results: &[MatchResult]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut countries: Vec<String> = Vec::new();
    for r in results {
        let c = infer_country(r.job.location.as_deref());
        if seen.insert(c.clone()) {
            countries.push(c);
        }
    }
    countries.sort();
    countries
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MatchResult;

    fn make_result(score: f64, days_ago: i64) -> MatchResult {
        use chrono::TimeDelta;
        let posted = chrono::Utc::now() - TimeDelta::try_days(days_ago).unwrap();
        MatchResult {
            score,
            matched_skills: vec![],
            matched_keywords: vec![],
            missing_skills: vec![],
            job: crate::models::JobPost {
                id: String::new(),
                title: String::new(),
                company: None,
                location: None,
                description: String::new(),
                url: String::new(),
                source: crate::models::JobSource::RemoteOk,
                posted_at: Some(posted),
                crawled_at: chrono::Utc::now(),
                salary: None,
                job_type: None,
                tags: vec![],
            },
        }
    }

    #[test]
    fn test_sort_by_score_desc() {
        let mut results = vec![make_result(0.5, 5), make_result(0.9, 1), make_result(0.5, 2)];
        sort_by_score_desc(&mut results);
        assert_eq!(results[0].score, 0.9);
        // scores equal → newer first
        assert_eq!(results[1].score, 0.5);
        assert!(results[1].job.posted_at.unwrap() > results[2].job.posted_at.unwrap());
    }

    #[test]
    fn test_sort_by_date_newest() {
        let mut results = vec![make_result(0.5, 10), make_result(0.5, 1)];
        sort_by_date_newest(&mut results);
        assert!(results[0].job.posted_at.unwrap() > results[1].job.posted_at.unwrap());
    }

    #[test]
    fn test_sort_by_date_oldest() {
        let mut results = vec![make_result(0.5, 1), make_result(0.5, 10)];
        sort_by_date_oldest(&mut results);
        assert!(results[0].job.posted_at.unwrap() < results[1].job.posted_at.unwrap());
    }
}
