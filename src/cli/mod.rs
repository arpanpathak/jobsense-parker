//! # CLI
//!
//! Interactive terminal interface for JobSense-Parker.
//!
//! Built on [`dialoguer`] with a colourful main menu, paginated result
//! viewer, and a live directory browser for picking resume files. Every
//! user-facing operation is exposed as a method on the [`App`] struct and
//! dispatched from the main event loop.
//!
//! ## Menu commands
//!
//! | # | Action                    | Description                              |
//! |---|---------------------------|------------------------------------------|
//! | 0 | Scan jobs                 | Crawl all sources using resume keywords  |
//! | 1 | Search                    | Crawl with a custom query                |
//! | 2 | View results              | Paginated browsing of match results      |
//! | 3 | Load resume               | File-picker for PDF / JSON / YAML / text |
//! | 4 | Show resume               | Display loaded resume summary            |
//! | 5 | Filter / sort results     | Reorder or filter the result set         |
//! | 6 | Quit                      | Exit the application                     |

use colored::Colorize;
use dialoguer::{FuzzySelect, Input, Select};

use crate::crawler::CrawlerCoordinator;
use crate::matcher::Matcher;
use crate::models::{Command, MatchResult, Resume, SearchConfig};

// ─── App State ────────────────────────────────────────────────────────────────

/// Top-level application state and event-loop orchestrator.
///
/// Owns the [`Matcher`], [`CrawlerCoordinator`], result cache, and active
/// [`SearchConfig`]. Call [`App::run`] to enter the interactive menu loop.
pub struct App {
    /// Resume-job matching engine.
    matcher: Matcher,
    /// Multi-source crawl coordinator.
    coordinator: CrawlerCoordinator,
    /// Cached results from the most recent scan or search.
    results: Vec<MatchResult>,
    /// Current search configuration (keywords, sources, filters).
    config: SearchConfig,
}

impl App {
    /// Construct a new [`App`] with default values and no loaded resume.
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(),
            coordinator: CrawlerCoordinator::new(),
            results: Vec::new(),
            config: SearchConfig::default(),
        }
    }

    /// Enter the main interactive loop.
    ///
    /// Prints the banner, then repeatedly prompts the user for a command and
    /// dispatches it. The loop exits when the user selects Quit.
    pub async fn run(&mut self) {
        banner();

        loop {
            let cmd = self.prompt_command();
            match cmd {
                Command::Quit => {
                    println!("\n  {}\n", "👋 Later, hunter. Good luck out there.".bright_green());
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

    // ─── Command Prompt ───────────────────────────────────────────────────

    /// Render the main menu and return the selected [`Command`].
    ///
    /// Shows the current resume load status and result count in the menu
    /// labels. Uses [`dialoguer::Select`] for the menu and
    /// [`dialoguer::Input`] for the search query prompt.
    fn prompt_command(&self) -> Command {
        let resume_status = if self.matcher.has_resume() {
            "✓ loaded".green().to_string()
        } else {
            "not loaded".yellow().to_string()
        };

        let result_count = if self.results.is_empty() {
            "no results".dimmed().to_string()
        } else {
            format!("{} results", self.results.len()).cyan().to_string()
        };

        let items = vec![
            format!("{}  Scan jobs (all sources)", "🔍".bright_cyan()),
            format!("{}  Search with custom query", "🎯".bright_blue()),
            format!("{}  View results ({})", "📋", result_count),
            format!("{}  Load resume ({resume_status})", "📄".bright_yellow()),
            format!("{}  Show current resume", "👤".bright_magenta()),
            format!("{}  Filter / sort results", "🔧".bright_white()),
            format!("{}  Quit", "🚪".bright_red()),
        ];

        let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("jobense-parker")
            .items(&items)
            .default(0)
            .interact_opt()
            .unwrap_or(Some(6))
            .unwrap_or(6);

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
                let path = match pick_resume_file() {
                    Some(p) => p,
                    None => {
                        println!("  {} Cancelled.", "ℹ".yellow());
                        String::new()
                    }
                };
                Command::LoadResume(path)
            }
            4 => Command::ShowResume,
            5 => Command::FilterResults,
            6 | _ => Command::Quit,
        }
    }

    // ─── Commands ─────────────────────────────────────────────────────────

    /// Load a resume from user-provided input.
    ///
    /// The input is interpreted in the following priority order:
    ///
    /// 1. **PDF file path** (`.pdf`) — text is extracted via `pdf-extract`
    ///    and parsed with [`Resume::from_text`].
    /// 2. **File path** containing `/`, `\`, or ending in `.json` / `.yaml` /
    ///    `.yml` — the file is read and parsed as JSON, then YAML, then
    ///    plain text as a fallback.
    /// 3. **Raw text** — tried as JSON, then YAML, then
    ///    [`Resume::from_text`].
    ///
    /// On success the resume is passed to the [`Matcher`] and a summary is
    /// printed.
    fn cmd_load_resume(&mut self, input: &str) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("  {} No text provided.", "✗".red());
            return;
        }

        // Determine if the input looks like a file path
        let is_pdf = trimmed.ends_with(".pdf") || trimmed.ends_with(".PDF");
        let is_json = trimmed.ends_with(".json");
        let is_yaml = trimmed.ends_with(".yaml") || trimmed.ends_with(".yml");
        let looks_like_path = is_pdf || is_json || is_yaml || trimmed.contains('/') || trimmed.contains('\\');

        let resume = if is_pdf {
            // PDF file — extract text directly
            match pdf_extract::extract_text(trimmed) {
                Ok(pdf_text) => {
                    println!("  {} Extracted {} chars from PDF.", "✓".green(), pdf_text.len());
                    Resume::from_text(&pdf_text)
                }
                Err(e) => {
                    println!("  {} Failed to read PDF: {e}", "✗".red());
                    // Fallback: treat as raw text
                    Resume::from_text(trimmed)
                }
            }
        } else if looks_like_path {
            // Try reading the file and parsing it
            match std::fs::read_to_string(trimmed) {
                Ok(content) => {
                    // Attempt JSON/YAML structured resume, fallback to text extraction
                    serde_json::from_str::<Resume>(&content)
                        .or_else(|_| serde_yaml::from_str::<Resume>(&content))
                        .unwrap_or_else(|_| {
                            println!("  {} Read file as plain text ({} chars).", "ℹ".yellow(), content.len());
                            Resume::from_text(&content)
                        })
                }
                Err(e) => {
                    println!("  {} Could not read file '{}': {e}", "✗".red(), trimmed);
                    // Final fallback: treat the path itself as raw text
                    Resume::from_text(trimmed)
                }
            }
        } else {
            // Raw text input — try JSON/YAML first, fallback to text extraction
            serde_json::from_str::<Resume>(trimmed)
                .or_else(|_| serde_yaml::from_str::<Resume>(trimmed))
                .unwrap_or_else(|_| Resume::from_text(trimmed))
        };

        self.matcher.load_resume(resume);
        println!(
            "  {} Resume loaded! ({} skills, {} roles)",
            "✓".green(),
            self.matcher.resume().map_or(0, |r| r.skills.len()),
            self.matcher.resume().map_or(0, |r| r.role_titles.len()),
        );
    }

    /// Display the currently loaded resume (skills, roles, experience,
    /// location, job type).
    fn cmd_show_resume(&self) {
        match self.matcher.resume() {
            None => println!("  {} No resume loaded.", "ℹ".yellow()),
            Some(r) => {
                println!();
                println!("  {} Current Resume", "📄".bright_yellow());
                println!("  {}", "─".repeat(48).dimmed());
                if !r.skills.is_empty() {
                    println!(
                        "  Skills:  {}",
                        r.skills
                            .iter()
                            .map(|s| s.green().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                if !r.role_titles.is_empty() {
                    println!(
                        "  Roles:   {}",
                        r.role_titles
                            .iter()
                            .map(|s| s.cyan().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                if let Some(y) = r.experience_years {
                    println!("  Exp:     {} years", y);
                }
                if let Some(l) = &r.preferred_location {
                    println!("  Loc:     {l}");
                }
                if let Some(t) = &r.preferred_job_type {
                    println!("  Type:    {t}");
                }
                println!();
            }
        }
    }

    /// Run a full scan across all configured job sources.
    ///
    /// Keywords are derived from the loaded resume (skills + role titles).
    /// If no resume is loaded, defaults to `"software engineer"` and
    /// `"developer"` and results are unscored.
    async fn cmd_scan(&mut self) {
        if !self.matcher.has_resume() {
            println!("  {} No resume loaded. Scanning without matching — results won't be scored.", "⚠".yellow());
            println!("     Use 'Load resume' first, or continue with raw results anyway.");
        }

        self.config.keywords = if let Some(r) = self.matcher.resume() {
            let mut kws = r.skills.clone();
            kws.extend(r.role_titles.clone());
            if kws.is_empty() {
                vec!["software engineer".to_string(), "developer".to_string()]
            } else {
                kws
            }
        } else {
            vec!["software engineer".to_string(), "developer".to_string()]
        };

        println!(
            "\n  {} Scanning with keywords: {}\n",
            "🌐".bright_blue(),
            self.config.keywords.iter().map(|k| k.green().to_string()).collect::<Vec<_>>().join(", ")
        );

        let jobs = self.coordinator.crawl_all(&self.config).await;

        if jobs.is_empty() {
            println!("\n  {} No jobs found. Try different keywords or sources.", "⚠".yellow());
            return;
        }

        println!(
            "\n  {} Found {} raw job posts. Matching against resume...",
            "📊".bright_cyan(),
            jobs.len()
        );

        if self.matcher.has_resume() {
            self.results = self.matcher.score_all(&jobs);
            println!(
                "  {} {} matched results (above threshold)\n",
                "✓".green(),
                self.results.len()
            );
        } else {
            // Without resume, create low-score entries for everything
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
            println!(
                "  {} {} raw results (no resume to match)\n",
                "ℹ".yellow(),
                self.results.len()
            );
        }
    }

    /// Search with a custom query string, then score results against the
    /// loaded resume (if any).
    async fn cmd_search(&mut self, query: &str) {
        if query.trim().is_empty() {
            println!("  {} Empty query, cancelling.", "✗".red());
            return;
        }

        self.config.keywords = query.split_whitespace().map(|s| s.to_string()).collect();

        println!(
            "\n  {} Searching for: {}\n",
            "🎯".bright_blue(),
            query.bright_white()
        );

        let jobs = self.coordinator.crawl_all(&self.config).await;

        if jobs.is_empty() {
            println!("\n  {} No jobs found.", "⚠".yellow());
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

        println!(
            "\n  {} Found {} results\n",
            "✓".green(),
            self.results.len()
        );
    }

    /// Display the cached results with pagination.
    ///
    /// Shows 10 results per page. Each entry includes the score, source,
    /// company, location, matched skills, and missing skills. The user can
    /// navigate pages, jump to a specific page, or return to the main menu.
    fn cmd_view_results(&self) {
        if self.results.is_empty() {
            println!("  {} No results yet. Run a scan or search first.", "ℹ".yellow());
            return;
        }

        let page_size = 10;
        let total_pages = (self.results.len() + page_size - 1) / page_size;
        let mut current_page = 0;

        loop {
            let start = current_page * page_size;
            let end = usize::min(start + page_size, self.results.len());
            let page_results = &self.results[start..end];

            println!();
            println!("  {} Results (page {}/{} — {} total)", "📋".bright_cyan(), current_page + 1, total_pages, self.results.len());
            println!("  {}", "─".repeat(60).dimmed());
            println!();

            for (i, result) in page_results.iter().enumerate() {
                let idx = start + i + 1;
                let _score_bar = score_bar(result.score);
                let score_pct = format!("{:.0}%", result.score * 100.0);

                let score_color = if result.score >= 0.7 {
                    score_pct.green()
                } else if result.score >= 0.4 {
                    score_pct.yellow()
                } else {
                    score_pct.dimmed()
                };

                println!(
                    "  {}. {} {} [{}]",
                    format!("{:>2}", idx).bright_black(),
                    result.job.title.bright_white(),
                    score_color,
                    result.job.source,
                );

                if let Some(c) = &result.job.company {
                    println!("     Company: {}", c.cyan());
                }
                if let Some(l) = &result.job.location {
                    println!("     Location: {}", l);
                }

                if !result.matched_skills.is_empty() {
                    println!(
                        "     {} {}",
                        "✓".green(),
                        result.matched_skills.join(", ")
                    );
                }
                if !result.missing_skills.is_empty() {
                    println!(
                        "     {} {}",
                        "✗".red(),
                        result.missing_skills.join(", ")
                    );
                }

                println!("     {}", result.job.url.dimmed());
                println!();
            }

            // Navigation
            let nav_items = if total_pages <= 1 {
                vec!["Back to main menu".to_string()]
            } else if current_page == 0 {
                vec![
                    "Next page →".to_string(),
                    "Back to main menu".to_string(),
                    format!("Jump to page (1-{total_pages})"),
                ]
            } else if current_page == total_pages - 1 {
                vec![
                    "← Previous page".to_string(),
                    "Back to main menu".to_string(),
                ]
            } else {
                vec![
                    "← Previous page".to_string(),
                    "Next page →".to_string(),
                    "Back to main menu".to_string(),
                    format!("Jump to page (1-{total_pages})"),
                ]
            };

            let selection = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                .with_prompt("Navigate")
                .items(&nav_items)
                .default(0)
                .interact_opt()
                .unwrap_or(Some(1))
                .unwrap_or(1);

            match nav_items[selection].as_str() {
                "← Previous page" => {
                    if current_page > 0 {
                        current_page -= 1
                    }
                }
                "Next page →" => {
                    if current_page < total_pages - 1 {
                        current_page += 1
                    }
                }
                "Back to main menu" => break,
                _ => {
                    if selection > 0 {
                        let page: String = Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
                            .with_prompt("Page number")
                            .interact_text()
                            .unwrap_or_default();
                        if let Ok(p) = page.parse::<usize>() {
                            if p > 0 && p <= total_pages {
                                current_page = p - 1;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Filter or sort the current result set in-place.
    ///
    /// Options: sort ascending / descending by score, show only high /
    /// medium / low matches, or reset (user is told to re-scan).
    fn cmd_filter_results(&mut self) {
        if self.results.is_empty() {
            println!("  {} No results to filter.", "ℹ".yellow());
            return;
        }

        let items = vec![
            "Sort by score (high → low)".to_string(),
            "Sort by score (low → high)".to_string(),
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
                println!("  {} Sorted by score (descending).", "✓".green());
            }
            1 => {
                self.results
                    .sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
                println!("  {} Sorted by score (ascending).", "✓".green());
            }
            2 => {
                self.results.retain(|r| r.score >= 0.7);
                println!("  {} Filtered to {} high-match results.", "✓".green(), self.results.len());
            }
            3 => {
                self.results.retain(|r| r.score >= 0.4 && r.score < 0.7);
                println!("  {} Filtered to {} medium-match results.", "✓".green(), self.results.len());
            }
            4 => {
                self.results.retain(|r| r.score < 0.4);
                println!("  {} Filtered to {} low-match results.", "✓".green(), self.results.len());
            }
            5 => {
                // Can't really "reset" without re-scanning, so we'll tell user
                println!("  {} Re-run a scan to get fresh results.", "ℹ".yellow());
            }
            _ => {}
        }
    }
}

// ─── File Picker ───────────────────────────────────────────────────────────────

/// Interactive directory walker to pick a resume file.
///
/// Starts in the current working directory and lets the user navigate the
/// file system using arrow keys and fuzzy search. Only shows directories
/// and files with supported extensions (`.pdf`, `.json`, `.yaml`, `.yml`).
///
/// - **Enter** on a directory → drill into it
/// - **Enter** on a file → return its absolute path
/// - **`..`** entry → go up one level
/// - **Escape** → cancel and return `None`
///
/// Uses [`dialoguer::FuzzySelect`] so the user can type to filter the
/// current directory's contents.
fn pick_resume_file() -> Option<String> {
    let mut current_dir = std::env::current_dir().ok()?;

    loop {
        // ── Read directory ─────────────────────────────────────────────────
        let mut entries: Vec<(String, bool, std::path::PathBuf)> = Vec::new();

        // Parent directory (unless we're at root)
        if let Some(parent) = current_dir.parent() {
            entries.push(("..".to_string(), true, parent.to_path_buf()));
        }

        let dir_iter = match std::fs::read_dir(&current_dir) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("  {} Cannot read directory.", "✗".red());
                return None;
            }
        };

        for entry in dir_iter.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue; // skip hidden files/dirs
            }

            if path.is_dir() {
                entries.push((name, true, path));
            } else {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase())
                    .unwrap_or_default();
                if matches!(ext.as_str(), "pdf" | "json" | "yaml" | "yml") {
                    entries.push((name, false, path));
                }
            }
        }

        if entries.is_empty() {
            eprintln!("  {} No compatible files in this directory.", "ℹ".yellow());
            return None;
        }

        // Sort: directories first, then files, both alphabetically
        entries.sort_by(|a, b| {
            b.1
                .cmp(&a.1)
                .then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase()))
        });

        // Build display list with icons
        let display_items: Vec<String> = entries
            .iter()
            .map(|(name, is_dir, _)| {
                if *is_dir {
                    format!("  {} {}/", "📁", name)
                } else {
                    format!("  {} {}", "📄", name)
                }
            })
            .collect();

        // ── Prompt user ────────────────────────────────────────────────────
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
            None => return None, // Esc → cancel
            Some(idx) => {
                let (_, is_dir, path) = &entries[idx];
                if *is_dir {
                    current_dir = path.clone();
                } else {
                    let path_str = path.to_string_lossy().to_string();
                    println!(
                        "  {} Selected: {}",
                        "📄".bright_yellow(),
                        path_str.dimmed()
                    );
                    return Some(path_str);
                }
            }
        }
    }
}

// ─── Banner ───────────────────────────────────────────────────────────────────

/// Print the application "splash" banner when the app starts.
fn banner() {
    println!();
    println!(
        "  {}",
        "╔══════════════════════════════════════════════════╗"
            .bright_blue()
    );
    println!(
        "  {}  {}  v0.1{}",
        "║".bright_blue(),
        "🔍 JobSense-Parker".bright_white().bold(),
        "║".bright_blue(),
    );
    println!(
        "  {}  Hunt the internet for your next gig.   {}",
        "║".bright_blue(),
        "║".bright_blue()
    );
    println!(
        "  {}  (LinkedIn-free zone)                   {}",
        "║".bright_blue(),
        "║".bright_blue()
    );
    println!(
        "  {}",
        "╚══════════════════════════════════════════════════╝"
            .bright_blue()
    );
    println!();
}

/// Render a visual score bar of 20 blocks.
///
/// Green for scores ≥ 70 %, yellow for 40–69 %, dimmed below 40 %.
fn score_bar(score: f64) -> String {
    let filled = (score * 20.0).round() as usize;
    let empty = 20 - filled;
    let bar: String = std::iter::repeat("█")
        .take(filled)
        .chain(std::iter::repeat("░").take(empty))
        .collect();
    if score >= 0.7 {
        bar.green().to_string()
    } else if score >= 0.4 {
        bar.yellow().to_string()
    } else {
        bar.dimmed().to_string()
    }
}
