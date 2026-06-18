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
/// Dry-run planning helpers for mutating commands.
pub mod dry_run;
/// Structured runtime error rendering and semantic exit-code mapping.
pub mod error;
/// Static output field metadata for commands that support field projection.
pub mod field_metadata;
/// Built-in operational help topics.
pub mod help;
/// Stderr-only tracing initialization for CLI diagnostics.
pub mod logging;
/// JSON output formatting and field selection.
pub mod output;
/// Machine-readable CLI schema generation.
pub mod schema;

use clap::Parser;

use crate::cli::error::CliExit;

pub use args::{
    AlertArgs, Cli, Commands, CommandsArgs, CompletionsArgs, DoctorArgs, FieldsArgs, HelpArgs,
    HelpTopic, MarketArgs, ReportArgs, TradeArgs, VolumeArgs, WatchlistArgs,
};

/// Parses CLI arguments, routes to the appropriate command handler, and returns
/// the process exit code.
pub async fn run() -> i32 {
    let cli = Cli::parse_from(args::normalize_alias_args(std::env::args()));
    logging::init(cli.verbose);
    output::configure_strict_empty(
        cli.strict_empty,
        output::strict_empty_command_from_args(std::env::args().skip(1)),
    );

    let result: Result<(), CliExit> = match &cli.command {
        Commands::Report(args) => commands::report::handle(args).await,
        Commands::Trade(args) => commands::trade::handle(args).await,
        Commands::Volume(args) => commands::volume::handle(args).await,
        Commands::Market(args) => commands::market::handle(args).await,
        Commands::Alert(args) => commands::alert::handle(args).await,
        Commands::Watchlist(args) => commands::watchlist::handle(args).await,
        Commands::Doctor(args) => doctor::handle(args).await,
        Commands::Commands(args) => command_list::handle(args),
        Commands::Fields(args) => commands::fields::handle(args),
        Commands::Help(args) => help::handle(args),
        Commands::Schema => schema::handle(),
        Commands::Completions(args) => {
            commands::completions::handle(args);
            Ok(())
        }
    };

    match result {
        Ok(()) => 0,
        Err(exit) => exit.code(),
    }
}
