use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;

use crate::commands::alert::AlertCommand;
use crate::commands::market::MarketCommand;
use crate::commands::report::ReportCommand;
use crate::commands::trade::TradeCommand;
use crate::commands::volume::VolumeCommand;
use crate::commands::watchlist::WatchlistCommand;

/// CLI tool for querying VolumeLeaders institutional trade data.
#[derive(Debug, Parser)]
#[command(
    name = "volumeleaders-agent",
    version,
    about = "CLI tool for querying VolumeLeaders institutional trade data",
    long_about = "volumeleaders-agent queries institutional trade data from VolumeLeaders.\n\n\
        Use it for trades, volume leaderboards, market data, alerts, and watchlists.\n\n\
        Auth: reads browser cookies automatically. If auth fails with exit code 2,\n\
        log in at https://www.volumeleaders.com in your browser, then retry.\n\n\
        Output: tab-separated values (TSV) to stdout by default. Use --json for compact JSON\n\
        or --pretty for indented JSON.\n\
        Errors and logs go to stderr.",
    arg_required_else_help = true,
    propagate_version = true
)]
pub struct Cli {
    /// Use compact JSON output instead of TSV.
    #[arg(short, long, global = true)]
    pub json: bool,

    /// Pretty-print JSON output with indentation.
    #[arg(short, long, global = true, conflicts_with = "json")]
    pub pretty: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level command groups.
#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// Preset report commands for common trade scans.
    Report(ReportArgs),
    /// Individual trade lookup, dashboards, and analysis.
    Trade(TradeArgs),
    /// Volume leaderboard commands.
    Volume(VolumeArgs),
    /// Market data commands (earnings, exhaustion).
    Market(MarketArgs),
    /// Alert configuration management.
    Alert(AlertArgs),
    /// Watchlist management and inspection.
    Watchlist(WatchlistArgs),
    /// Generate shell completions.
    Completions(CompletionsArgs),
}

/// Arguments for the report command group.
#[derive(Debug, Args)]
pub struct ReportArgs {
    #[command(subcommand)]
    pub command: ReportCommand,
}

/// Arguments for trade commands.
#[derive(Debug, Args)]
pub struct TradeArgs {
    #[command(subcommand)]
    pub command: TradeCommand,
}

/// Arguments for volume commands.
#[derive(Debug, Args)]
pub struct VolumeArgs {
    #[command(subcommand)]
    pub command: VolumeCommand,
}

/// Arguments for market commands.
#[derive(Debug, Args)]
pub struct MarketArgs {
    #[command(subcommand)]
    pub command: MarketCommand,
}

/// Arguments for alert commands.
#[derive(Debug, Args)]
pub struct AlertArgs {
    #[command(subcommand)]
    pub command: AlertCommand,
}

/// Arguments for watchlist commands.
#[derive(Debug, Args)]
pub struct WatchlistArgs {
    #[command(subcommand)]
    pub command: WatchlistCommand,
}

/// Arguments for shell completion generation (populated in a later wave).
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    pub shell: Shell,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    #[test]
    fn command_tree_is_valid() {
        Cli::command().debug_assert();
    }
}
