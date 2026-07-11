//! Terminal rendering helpers — banner, resume display, results pagination,
//! scan history, and CLI help text.

use colored::Colorize;

use crate::models::{MatchResult, Resume, ScanRecord};

/// Render the startup banner.
pub fn banner() {
    println!();
    println!(
        "  {}",
        "╔══════════════════════════════════════════════════╗"
            .bright_blue()
    );
    println!(
        "  {}  JobSense-Parker  v0.2{}",
        "║".bright_blue(),
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

/// Render a single page of match results with navigation.
pub fn show_results_page(results: &[MatchResult], page: usize, total_pages: usize) {
    let page_size = 10;
    let start = page * page_size;
    let end = usize::min(start + page_size, results.len());
    let page_results = &results[start..end];

    println!();
    println!(
        "  Results (page {}/{} -- {} total)",
        page + 1,
        total_pages,
        results.len()
    );
    println!("  {}", "─".repeat(60).dimmed());
    println!();

    for (i, result) in page_results.iter().enumerate() {
        let idx = start + i + 1;
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
                "     + {}",
                result.matched_skills.join(", ")
            );
        }
        if !result.missing_skills.is_empty() {
            println!(
                "     - {}",
                result.missing_skills.join(", ")
            );
        }

        println!("     {}", result.job.url.dimmed());
        println!();
    }
}

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

/// Print the CLI usage help text.
pub fn print_help() {
    println!();
    println!("  Usage: jobsense-parker [COMMAND]");
    println!();
    println!("  Commands:");
    println!("    (no args)     Start interactive menu");
    println!("    --help        Show this help");
    println!("    --scan        Scan all sources with loaded resume");
    println!("    --search <q>  Search with a custom query");
    println!("    --resume <p>  Set resume file path (PDF, JSON, YAML, TXT)");
    println!("    --results     View last cached results");
    println!("    --history     Show scan history");
    println!();
}
