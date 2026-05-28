use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::cli::commands::alert::AlertCommand;
use crate::cli::commands::market::MarketCommand;
use crate::cli::commands::report::ReportCommand;
use crate::cli::commands::trade::TradeCommand;
use crate::cli::commands::volume::VolumeCommand;
use crate::cli::commands::watchlist::WatchlistCommand;

/// CLI tool for querying VolumeLeaders institutional trade data.
#[derive(Debug, Parser)]
#[command(
    name = "volumeleaders-agent",
    version,
    about = "CLI tool for querying VolumeLeaders institutional trade data",
    long_about = "volumeleaders-agent queries institutional trade data from VolumeLeaders.\n\n\
        Use it for trades, volume leaderboards, market data, alerts, and watchlists.\n\n\
        Auth: reads browser cookies automatically. If auth fails with exit code 3,\n\
        log in at https://www.volumeleaders.com in your browser, then retry.\n\n\
        Output: compact JSON to stdout. Pipe through jq for pretty-printing.\n\
        Runtime errors use one structured JSON line on stderr.",
    disable_help_subcommand = true,
    arg_required_else_help = true,
    propagate_version = true
)]
pub struct Cli {
    /// Accepted for backward compatibility (output is always JSON now).
    #[arg(short, long, global = true, hide = true)]
    pub json: bool,

    /// Subcommand to run.
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
    /// Check local auth and environment readiness as JSON.
    Doctor,
    /// List available leaf command paths.
    Commands(CommandsArgs),
    /// Show built-in operational help topics.
    Help(HelpArgs),
    /// Emit machine-readable command metadata as JSON.
    Schema,
    /// Generate shell completions.
    Completions(CompletionsArgs),
}

/// Arguments for the report command group.
#[derive(Debug, Args)]
pub struct ReportArgs {
    /// Report subcommand to run.
    #[command(subcommand)]
    pub command: ReportCommand,
}

/// Arguments for trade commands.
#[derive(Debug, Args)]
pub struct TradeArgs {
    /// Trade subcommand to run.
    #[command(subcommand)]
    pub command: TradeCommand,
}

/// Arguments for volume commands.
#[derive(Debug, Args)]
pub struct VolumeArgs {
    /// Volume subcommand to run.
    #[command(subcommand)]
    pub command: VolumeCommand,
}

/// Arguments for market commands.
#[derive(Debug, Args)]
pub struct MarketArgs {
    /// Market subcommand to run.
    #[command(subcommand)]
    pub command: MarketCommand,
}

/// Arguments for alert commands.
#[derive(Debug, Args)]
pub struct AlertArgs {
    /// Alert subcommand to run.
    #[command(subcommand)]
    pub command: AlertCommand,
}

/// Arguments for watchlist commands.
#[derive(Debug, Args)]
pub struct WatchlistArgs {
    /// Watchlist subcommand to run.
    #[command(subcommand)]
    pub command: WatchlistCommand,
}

/// Arguments for command discovery output.
#[derive(Debug, Args)]
pub struct CommandsArgs {
    /// Group commands by their top-level command with short descriptions.
    #[arg(long)]
    pub grouped: bool,
}

/// Arguments for built-in operational help topics.
#[derive(Debug, Args)]
pub struct HelpArgs {
    /// Help topic to show.
    pub topic: HelpTopic,
}

/// Built-in operational help topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HelpTopic {
    /// Browser-cookie authentication and local diagnostics.
    Auth,
    /// Local environment expectations.
    Environment,
    /// Semantic exit codes and recovery guidance.
    ExitCodes,
    /// CLI discovery through schema and commands output.
    Schema,
    /// Copy-paste command examples.
    Examples,
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
