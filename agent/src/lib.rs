//! VolumeLeaders CLI agent.
//!
//! This project is unofficial and is not affiliated with, endorsed by, or
//! sponsored by [volumeleaders.com](https://www.volumeleaders.com).

pub mod cli;
pub mod commands;
pub mod common;
pub mod output;

use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::output::OutputFormat;

/// Parses CLI arguments, routes to the appropriate command handler, and returns
/// the process exit code.
pub async fn run() -> i32 {
    let cli = Cli::parse();
    let format = if cli.pretty {
        OutputFormat::JsonPretty
    } else if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Tsv
    };

    match &cli.command {
        Commands::Report(args) => commands::report::handle(args, &format).await,
        Commands::Trade(args) => commands::trade::handle(args, &format).await,
        Commands::Volume(args) => commands::volume::handle(args, &format).await,
        Commands::Market(args) => commands::market::handle(args, &format).await,
        Commands::Alert(args) => commands::alert::handle(args, &format).await,
        Commands::Watchlist(args) => commands::watchlist::handle(args, &format).await,
        Commands::Completions(args) => {
            commands::completions::handle(args);
            0
        }
    }
}
