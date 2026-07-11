//! CLI entry point for **jobsense-parker** — a terminal-based job search tool
//! that hunts job boards and company career sites for positions matching your
//! resume or keywords.
//!
//! # Modes
//!
//! The tool has two modes of operation:
//!
//! ## 1. Interactive Mode (no arguments)
//!
//! ```text
//! $ jobsense-parker
//! ```
//!
//! Opens a colourful terminal menu where you can:
//! - **Scan** all sources + company career sites with your loaded resume
//! - **Search** with a custom query
//! - **View results** in a vim-style paginated browser (`j`/`k` to scroll,
//!   `Enter` to open URLs, `?` for help)
//! - **Manage companies** — view cached companies, add new ones
//! - **Load a resume** from PDF, JSON, YAML, or plain text
//!
//! ## 2. Non-Interactive Mode (flags)
//!
//! ```text
//! $ jobsense-parker --search "rust engineer"       # search + show results
//! $ jobsense-parker --scan                          # scan with loaded resume
//! $ jobsense-parker --companies                     # list cached companies
//! $ jobsense-parker --add-company "Tesla" "https://www.tesla.com/careers"
//! $ jobsense-parker --results                       # view last cached results
//! $ jobsense-parker --history                       # show scan history
//! ```
//!
//! # Data Storage
//!
//! All data is persisted under `~/.jobsense-parker/`:
//!
//! | File | Purpose |
//! |------|---------|
//! | `resume.json` | Your parsed resume (skills, roles, keywords) |
//! | `preferences.json` | User preferences (sources, max results) |
//! | `companies.json` | Discovered companies with careers-page URLs |
//! | `queries.json` | Recent search queries |
//! | `scan_history.json` | Scan records with timestamps, scores, counts |
//! | `last_results.json` | Most recent match results |
//!
//! # Architecture
//!
//! ```text
//! main.rs                         ← CLI entry point
//!   └─ cli/mod.rs                 ← Interactive menu + command dispatch
//!        └─ cli/views.rs          ← Terminal rendering (banner, vim viewer, help)
//!   lib.rs                        ← Library root
//!     ├─ crawler/                 ← Job-source crawlers
//!     │   ├─ mod.rs               ← CrawlerCoordinator (runs all sources)
//!     │   ├─ fetcher.rs           ← HTTP client (concurrent-safe)
//!     │   ├─ remoteok.rs          ← Remote OK job board
//!     │   ├─ reddit.rs            ← Reddit (r/forhire, r/jobbit, etc.)
//!     │   ├─ hackernews.rs        ← Hacker News "Who is Hiring?" thread
//!     │   └─ company.rs           ← Company career-site heuristics
//!     ├─ matcher/
//!     │   ├─ mod.rs               ← Resume-to-job matching engine
//!     │   └─ scoring.rs           ← Scoring algorithm (fuzzy matching)
//!     ├─ models/mod.rs            ← Data types (JobPost, Resume, etc.)
//!     └─ storage/mod.rs           ← JSON persistence layer
//! ```

use std::env;

use jobsense_parker::cli::{self, App};
use jobsense_parker::storage;

/// Entry point. Parses CLI arguments and either starts the interactive menu
/// or runs a single command.
///
/// # Examples
///
/// ```text
/// # Interactive session
/// jobsense-parker
///
/// # Quick search (results printed to stdout immediately)
/// jobsense-parker --search "python backend"
///
/// # Full scan with resume + company career sites
/// jobsense-parker --resume ~/resume.pdf --scan
/// ```
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Show help if requested
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        cli::print_help();
        return;
    }

    // Non-interactive mode: process CLI arguments one by one.
    // Flags are order-independent: `--search "rust" --resume r.pdf` works.
    if args.len() > 1 {
        let mut app = App::new();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--resume" | "-r" => {
                    i += 1;
                    if i < args.len() {
                        app.load_resume_file(&args[i]);
                    } else {
                        eprintln!("  --resume requires a file path argument");
                        return;
                    }
                }
                "--scan" | "-s" => {
                    app.run_scan().await;
                }
                "--search" => {
                    i += 1;
                    if i < args.len() {
                        app.run_search(&args[i]).await;
                    } else {
                        eprintln!("  --search requires a query string argument");
                        return;
                    }
                }
                "--results" => {
                    app.show_results();
                }
                "--history" => {
                    match storage::load_scan_history() {
                        Ok(records) => cli::show_scan_history(&records),
                        Err(e) => eprintln!("  Failed to load history: {e}"),
                    }
                }
                "--companies" | "--list-companies" => {
                    app.show_companies();
                }
                "--add-company" => {
                    i += 1;
                    if i + 1 < args.len() {
                        let name = &args[i];
                        i += 1;
                        let url = &args[i];
                        app.add_company_cli(name, url);
                    } else {
                        eprintln!("  --add-company requires NAME and URL arguments");
                        eprintln!("  Example: --add-company \"My Corp\" \"https://mycorp.com/careers\"");
                        return;
                    }
                }
                "--remove-company" => {
                    i += 1;
                    if i < args.len() {
                        app.remove_company_cli(&args[i]);
                    } else {
                        eprintln!("  --remove-company requires a company name argument");
                        return;
                    }
                }
                other => {
                    eprintln!("  Unknown argument: {other}");
                    cli::print_help();
                    return;
                }
            }
            i += 1;
        }
        return;
    }

    // Interactive mode: start the menu loop
    let mut app = App::new();
    app.run().await;
}
