//! # JobSense-Parker
//!
//! A CLI job crawler that hunts the internet for hiring posts matching your
//! resume — because LinkedIn sucks.
//!
//! ## Architecture
//!
//! The app is organised into four modules:
//!
//! - **`models`** — Core data types: [`Resume`](crate::models::Resume),
//!   [`JobPost`](crate::models::JobPost), [`JobSource`](crate::models::JobSource),
//!   [`MatchResult`](crate::models::MatchResult),
//!   [`SearchConfig`](crate::models::SearchConfig), and the
//!   [`Command`](crate::models::Command) enum.
//! - **`crawler`** — Web scrapers for Indeed, Google Jobs, Reddit, and Hacker
//!   News, plus an HTTP fetcher with polite delays and user-agent rotation.
//! - **`matcher`** — Scores job posts against a loaded resume using keyword
//!   matching, fuzzy string comparison, and configurable bonuses.
//! - **`cli`** — Interactive terminal UI built on [`dialoguer`] with a menu,
//!   paginated results, and a live directory file-picker.
//!
//! ## Usage
//!
//! ```bash
//! cargo run
//! ```
//!
//! Then follow the on-screen menu to load a resume (PDF, JSON, YAML, or plain
//! text), scan job sources, and view matched results.

mod cli;
mod crawler;
mod matcher;
mod models;

use cli::App;

/// Entry point: creates the application and runs the interactive CLI loop.
///
/// Initialises the [`App`] state (matcher, crawler coordinator, results cache,
/// search config) and enters the main menu loop. The loop exits when the user
/// selects the Quit command.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::new();
    app.run().await;
    Ok(())
}
