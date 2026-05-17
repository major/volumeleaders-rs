pub mod cli;
pub mod commands;
pub mod common;
pub mod output;

use clap::Parser;

use crate::cli::{Cli, Commands};

/// Parses CLI arguments, routes to the appropriate command handler, and returns
/// the process exit code.
pub async fn run() -> i32 {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Report(args) => commands::report::handle(args, cli.pretty).await,
        Commands::Trade(args) => commands::trade::handle(args, cli.pretty).await,
        Commands::Volume(args) => commands::volume::handle(args, cli.pretty).await,
        Commands::Market(args) => commands::market::handle(args, cli.pretty).await,
        Commands::Alert(args) => commands::alert::handle(args, cli.pretty).await,
        Commands::Watchlist(args) => commands::watchlist::handle(args, cli.pretty).await,
        Commands::Completions(args) => {
            commands::completions::handle(args);
            0
        }
    }
}
