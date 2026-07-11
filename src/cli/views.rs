//! Terminal rendering helpers — banner, resume display, vim-style paginated
//! results viewer, scan history, and CLI help text.
//!
//! All URLs are rendered with OSC 8 terminal hyperlinks so you can
//! Cmd+click (macOS) or Ctrl+click (Linux/Windows) to open them.

use anyhow::Result;
use colored::Colorize;
use console::Term;

use crate::models::{CompanyDatabase, MatchResult, Resume, ScanRecord};

// ─── OSC 8 Hyperlink ───────────────────────────────────────────────────────

/// Wrap `text` in an [OSC 8 terminal hyperlink](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda).
///
/// Most modern terminals (iTerm2, Terminal.app, kitty, alacritty, Windows
/// Terminal) support Cmd+click / Ctrl+click on these.
pub fn clickable(url: &str, text: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
}

// ─── Banner ────────────────────────────────────────────────────────────────

/// Render the startup banner.
pub fn banner() {
    println!();
    println!(
        "  {}",
        "╔══════════════════════════════════════════════════════╗"
            .bright_blue()
    );
    println!(
        "  {}  JobSense-Parker  v0.3                          {}",
        "║".bright_blue(),
        "║".bright_blue(),
    );
    println!(
        "  {}  Hunt the internet for your next gig.           {}",
        "║".bright_blue(),
        "║".bright_blue()
    );
    println!(
        "  {}  Type '?' at any results view for keybindings   {}",
        "║".bright_blue(),
        "║".bright_blue()
    );
    println!(
        "  {}  (LinkedIn-free zone)                           {}",
        "║".bright_blue(),
        "║".bright_blue()
    );
    println!(
        "  {}",
        "╚══════════════════════════════════════════════════════╝"
            .bright_blue()
    );
    println!();
}

// ─── Resume ────────────────────────────────────────────────────────────────

/// Display the parsed contents of a resume.
pub fn show_resume(r: &Resume) {
    println!();
    println!("  Current Resume");
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

// ─── Vim-Style Paginated Results Viewer ────────────────────────────────────

const PAGE_SIZE: usize = 10;

/// Keybinding reference overlay.
const KEYBINDINGS: &str = r"
  ╔═══════════════════════════════════════╗
  ║          Results Viewer Keys          ║
  ╠═══════════════════════════════════════╣
  ║  j / ↓        Move selection down     ║
  ║  k / ↑        Move selection up       ║
  ║  n / →        Next page               ║
  ║  p / ←        Previous page           ║
  ║  g            First page              ║
  ║  G            Last page               ║
  ║  Enter / o    Open job URL in browser ║
  ║  q / Esc      Back to menu            ║
  ║  ?            Toggle this help        ║
  ╚═══════════════════════════════════════╝
";

/// Run the vim-style paginated results viewer.
///
/// ## Keybindings
///
/// | Key | Action |
/// |-----|--------|
/// | `j` / `↓` | Move selection down |
/// | `k` / `↑` | Move selection up |
/// | `n` / `→` | Next page |
/// | `p` / `←` | Previous page |
/// | `g` | First page |
/// | `G` | Last page |
/// | `Enter` / `o` | Open selected job URL in browser |
/// | `q` / `Esc` | Back to menu |
/// | `?` | Toggle keybinding help |
pub fn run_results_viewer(results: &[MatchResult]) -> Result<()> {
    if results.is_empty() {
        println!("  No results to display.");
        return Ok(());
    }

    let total_pages = (results.len() + PAGE_SIZE - 1) / PAGE_SIZE;
    let mut page = 0usize;
    let mut selected = 0usize; // index within current page
    let mut show_help = false;
    let term = Term::stdout();

    loop {
        // ── Render page ──────────────────────────────────────────────
        let start = page * PAGE_SIZE;
        let end = usize::min(start + PAGE_SIZE, results.len());
        let page_results = &results[start..end];

        // Clear screen (scroll up so history is still accessible)
        print!("\x1b[2J\x1b[H");

        println!();
        println!(
            "  {} {}results (page {}/{} · {} total){}",
            "▸".bright_blue(),
            "".bright_blue(),
            page + 1,
            total_pages,
            results.len(),
            "".bright_blue(),
        );
        println!("  {}", "─".repeat(60).dimmed());
        println!();

        for (i, result) in page_results.iter().enumerate() {
            let is_selected = i == selected;
            let prefix = if is_selected { "▸".yellow() } else { " ".into() };

            let idx = start + i + 1;
            let score_pct = format!("{:.0}%", result.score * 100.0);
            let score_color = if result.score >= 0.7 {
                score_pct.green()
            } else if result.score >= 0.4 {
                score_pct.yellow()
            } else {
                score_pct.dimmed()
            };

            let company = result
                .job
                .company
                .as_deref()
                .map(|c| format!(" @ {}", c.cyan()))
                .unwrap_or_default();

            // Highlight selected row
            let line = if is_selected {
                format!(
                    " {}{:>2}. {} {} [{}]{}",
                    prefix,
                    idx,
                    result.job.title.bright_white(),
                    score_color,
                    result.job.source,
                    company,
                ).on_blue().black().to_string()
            } else {
                format!(
                    " {}{:>2}. {} {} [{}]{}",
                    prefix,
                    idx,
                    result.job.title.bright_white(),
                    score_color,
                    result.job.source,
                    company,
                )
            };
            println!("{line}");

            // URL as clickable link
            let url_display = if is_selected {
                clickable(&result.job.url, &result.job.url).dimmed().to_string()
            } else {
                format!("     {}", clickable(&result.job.url, &result.job.url).dimmed())
            };
            println!("{url_display}");

            // Matched skills on selected item
            if is_selected && !result.matched_skills.is_empty() {
                println!(
                    "     {} {}",
                    "+".green(),
                    result.matched_skills.join(", ")
                );
            }
            if is_selected && !result.missing_skills.is_empty() {
                println!(
                    "     {} {}",
                    "-".red(),
                    result.missing_skills.join(", ")
                );
            }

            println!();
        }

        // ── Footer ───────────────────────────────────────────────────
        let footer = format!(
            "  [j↓ k↑  n→ p←  g/G  Enter:open  ?:help  q:quit]  ▸ {}",
            results[start + selected].job.title
        );
        println!("  {}", footer.dimmed());
        println!();

        // ── Help overlay ─────────────────────────────────────────────
        if show_help {
            for line in KEYBINDINGS.lines() {
                println!("{}", line.bright_yellow());
            }
            println!();
        }

        // ── Read key ─────────────────────────────────────────────────
        let key = term.read_key()?;

        match key {
            console::Key::Char('q') | console::Key::Escape => break,
            console::Key::Char('j') | console::Key::ArrowDown => {
                if selected + 1 < page_results.len() {
                    selected += 1;
                } else if page + 1 < total_pages {
                    page += 1;
                    selected = 0;
                }
            }
            console::Key::Char('k') | console::Key::ArrowUp => {
                if selected > 0 {
                    selected -= 1;
                } else if page > 0 {
                    page -= 1;
                    selected = PAGE_SIZE - 1;
                    // Clamp in case last page has fewer items
                    let prev_start = page * PAGE_SIZE;
                    let prev_end = usize::min(prev_start + PAGE_SIZE, results.len());
                    selected = usize::min(selected, prev_end - prev_start - 1);
                }
            }
            console::Key::Char('n') | console::Key::ArrowRight => {
                if page + 1 < total_pages {
                    page += 1;
                    selected = 0;
                }
            }
            console::Key::Char('p') | console::Key::ArrowLeft => {
                if page > 0 {
                    page -= 1;
                    selected = 0;
                }
            }
            console::Key::Char('g') => {
                page = 0;
                selected = 0;
            }
            console::Key::Char('G') => {
                page = total_pages - 1;
                selected = 0;
            }
            console::Key::Char('o') | console::Key::Enter => {
                let job = &results[start + selected].job;
                if let Err(e) = open_url(&job.url) {
                    eprintln!("  Failed to open URL: {e}");
                }
            }
            console::Key::Char('?') => {
                show_help = !show_help;
            }
            _ => {}
        }
    }

    print!("\x1b[2J\x1b[H");
    Ok(())
}

/// Open a URL in the system browser.
fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()?;
    }
    Ok(())
}

// ─── Scan History ──────────────────────────────────────────────────────────

/// Render the scan history (last 10 records).
pub fn show_scan_history(records: &[ScanRecord]) {
    if records.is_empty() {
        println!("  No scan history yet.");
        return;
    }
    println!();
    println!("  Scan History (last {} scans)", records.len());
    println!("  {}", "─".repeat(60).dimmed());
    for rec in records.iter().take(10) {
        println!(
            "  {} | query: '{}' | {} sources | {} results | top score: {:.0}%",
            rec.timestamp.format("%Y-%m-%d %H:%M"),
            rec.query,
            rec.source_count,
            rec.result_count,
            rec.top_score * 100.0,
        );
    }
    println!();
}

// ─── CLI Help ──────────────────────────────────────────────────────────────

/// Display the list of cached companies in a formatted table.
pub fn show_companies_list(db: &CompanyDatabase) {
    if db.companies.is_empty() {
        println!("  No companies cached yet. They are auto-discovered from job posts.");
        println!("  Use --add-company or the interactive menu to add manually.");
        return;
    }

    let failed = &db.failed;
    println!();
    println!(
        "  {} companies in cache ({} failed last crawl)",
        db.companies.len(),
        failed.len()
    );
    println!("  {}", "─".repeat(60).dimmed());

    for (i, company) in db.companies.iter().enumerate() {
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
            clickable(&company.careers_url, &company.careers_url).dimmed(),
            fail_note,
        );
    }
    println!();
}

/// Print the CLI usage help text with all commands and keybindings documented.
pub fn print_help() {
    println!();
    println!("  Usage: jobsense-parker [COMMAND]");
    println!();
    println!("  Commands:");
    println!("    (no args)               Start interactive menu");
    println!("    --help, -h              Show this help");
    println!("    --scan, -s              Scan all sources + company career sites");
    println!("    --search <query>        Search with a custom query");
    println!("    --resume, -r <file>     Set resume file path (PDF, JSON, YAML, TXT)");
    println!("    --results               View last cached results");
    println!("    --history               Show scan history");
    println!("    --companies             List all cached companies & career sites");
    println!("    --add-company <name> <url>  Add a company career site");
    println!("    --remove-company <name> Remove a company from the cache");
    println!();
    println!("  Examples:");
    println!("    jobsense-parker --search \"rust engineer\"");
    println!("    jobsense-parker --add-company \"Tesla\" \"https://www.tesla.com/careers\"");
    println!("    jobsense-parker --companies");
    println!();
    println!("  Interactive Menu Keybindings:");
    println!("    ↑/↓           Navigate menu items");
    println!("    Enter         Select item");
    println!("    Esc / q       Quit");
    println!();
    println!("  Results Viewer Keybindings:");
    println!("    j / ↓         Move selection down");
    println!("    k / ↑         Move selection up");
    println!("    n / →         Next page");
    println!("    p / ←         Previous page");
    println!("    g             First page");
    println!("    G             Last page");
    println!("    Enter / o     Open job URL in browser");
    println!("    q / Esc       Back to menu");
    println!("    ?             Toggle keybinding help overlay");
    println!();
    println!("  Company Career Sites:");
    println!("    On first run, 80+ major tech companies are pre-seeded.");
    println!("    New companies are auto-discovered from job posts during scans.");
    println!("    Company career pages are crawled alongside job boards during scans.");
    println!("    Career-site job listings appear in results just like board posts.");
    println!();
    println!("  URLs are clickable (Cmd+click on macOS, Ctrl+click elsewhere).");
    println!();
}
