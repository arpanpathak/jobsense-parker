//! CLI entry point for jobsense-parker.
//!
//! Supports two modes:
//! * **Non-interactive** — pass flags like `--resume`, `--scan`, `--search`, `--results`, `--history`, `--companies`
//! * **Interactive** — run without arguments to open the terminal menu

use std::env;

use jobsense_parker::cli::{self, App};
use jobsense_parker::storage;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Show help if requested
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        cli::print_help();
        return;
    }

    // Non-interactive mode: process CLI arguments
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
