//! Command-line interface for VolumeLeaders data.

/// Clap argument definitions and top-level command structs.
pub mod args;
/// Human-readable command discovery output.
pub mod command_list;
/// Command handlers for each CLI subcommand group.
pub mod commands;
/// Shared CLI utilities: auth, dates, formatting, tickers, types.
pub mod common;
/// Local environment and auth readiness diagnostics.
pub mod doctor;
/// Structured runtime error rendering and semantic exit-code mapping.
pub mod error;
/// JSON output formatting and field selection.
pub mod output;
/// Machine-readable CLI schema generation.
pub mod schema;

use clap::Parser;

pub use args::{
    AlertArgs, Cli, Commands, CommandsArgs, CompletionsArgs, MarketArgs, ReportArgs, TradeArgs,
    VolumeArgs, WatchlistArgs,
};

/// Parses CLI arguments, routes to the appropriate command handler, and returns
/// the process exit code.
pub async fn run() -> i32 {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Report(args) => commands::report::handle(args).await,
        Commands::Trade(args) => commands::trade::handle(args).await,
        Commands::Volume(args) => commands::volume::handle(args).await,
        Commands::Market(args) => commands::market::handle(args).await,
        Commands::Alert(args) => commands::alert::handle(args).await,
        Commands::Watchlist(args) => commands::watchlist::handle(args).await,
        Commands::Doctor => doctor::handle(),
        Commands::Commands(args) => command_list::handle(args),
        Commands::Schema => schema::handle(),
        Commands::Completions(args) => {
            commands::completions::handle(args);
            0
        }
    }
}
