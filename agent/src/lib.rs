//! VolumeLeaders CLI agent.
//!
//! This project is unofficial and is not affiliated with, endorsed by, or
//! sponsored by [volumeleaders.com](https://www.volumeleaders.com).

#![deny(missing_docs)]

/// Clap command tree and top-level argument structs.
pub mod cli;
/// Command handlers for each CLI subcommand group.
pub mod commands;
/// Shared utilities: auth, dates, formatting, tickers, types.
pub mod common;
/// JSON output formatting and field selection.
pub mod output;

use clap::Parser;

use crate::cli::{Cli, Commands};

/// Parses CLI arguments, routes to the appropriate command handler, and returns
/// the process exit code.
pub async fn run() -> i32 {
    let cli = Cli::parse();

    let json_table = cli.json_table;

    match &cli.command {
        Commands::Report(args) => commands::report::handle(args, json_table).await,
        Commands::Trade(args) => commands::trade::handle(args, json_table).await,
        Commands::Volume(args) => commands::volume::handle(args, json_table).await,
        Commands::Market(args) => commands::market::handle(args, json_table).await,
        Commands::Alert(args) => commands::alert::handle(args, json_table).await,
        Commands::Watchlist(args) => commands::watchlist::handle(args, json_table).await,
        Commands::Completions(args) => {
            commands::completions::handle(args);
            0
        }
    }
}
