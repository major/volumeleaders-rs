use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
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

    /// Exit with code 7 and a structured error when record-array output is empty.
    #[arg(long, global = true)]
    pub strict_empty: bool,

    /// Increase diagnostic logging on stderr (-v info, -vv debug, -vvv trace).
    #[arg(short = 'v', long = "verbose", global = true, action = ArgAction::Count)]
    pub verbose: u8,

    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Commands,
}

/// Rewrites supported top-level aliases into their canonical command paths.
pub fn normalize_alias_args(args: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut args = args.into_iter().collect::<Vec<_>>();
    let Some(index) = args
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, arg)| (!arg.starts_with('-')).then_some(index))
    else {
        return args;
    };

    let replacement = match args[index].as_str() {
        "trades" => Some(["trade", "list"]),
        "dashboard" => Some(["trade", "dashboard"]),
        "levels" => Some(["trade", "levels"]),
        _ => None,
    };

    if let Some(replacement) = replacement {
        args.splice(index..=index, replacement.map(str::to_string));
    }

    args
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
    #[command(
        long_about = "Check local auth and environment readiness as compact JSON without using the network.\n\nExamples:\n  volumeleaders-agent doctor\n  volumeleaders-agent doctor | jq '.auth.status'"
    )]
    Doctor,
    /// List available leaf command paths.
    #[command(
        long_about = "List available leaf command paths from the live clap command tree.\n\nExamples:\n  volumeleaders-agent commands\n  volumeleaders-agent commands --grouped"
    )]
    Commands(CommandsArgs),
    /// Show built-in operational help topics.
    #[command(
        long_about = "Show built-in operational help topics when README access is unavailable.\n\nExamples:\n  volumeleaders-agent help auth\n  volumeleaders-agent help examples"
    )]
    Help(HelpArgs),
    /// Emit machine-readable command metadata as JSON.
    #[command(
        long_about = "Emit machine-readable command metadata generated from the live clap command tree.\n\nExamples:\n  volumeleaders-agent schema\n  volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == \"trade list\")'"
    )]
    Schema,
    /// Generate shell completions.
    #[command(
        long_about = "Generate shell completion scripts for supported shells.\n\nExamples:\n  volumeleaders-agent completions bash\n  volumeleaders-agent completions zsh > _volumeleaders-agent"
    )]
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
    /// Automation guidance for non-interactive agents.
    Agent,
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
    use clap::{Command, CommandFactory, Parser};

    use super::{Cli, normalize_alias_args};

    #[test]
    fn command_tree_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn strict_empty_is_global_flag() {
        let before_command = Cli::try_parse_from([
            "volumeleaders-agent",
            "--strict-empty",
            "trade",
            "list",
            "NVDA",
        ])
        .unwrap();
        let after_command = Cli::try_parse_from([
            "volumeleaders-agent",
            "trade",
            "list",
            "NVDA",
            "--strict-empty",
        ])
        .unwrap();

        assert!(before_command.strict_empty);
        assert!(after_command.strict_empty);
    }

    #[test]
    fn verbose_is_global_count_flag() {
        let before_command = Cli::try_parse_from(["volumeleaders-agent", "-vv", "doctor"]).unwrap();
        let after_command = Cli::try_parse_from(["volumeleaders-agent", "doctor", "-vvv"]).unwrap();

        assert_eq!(before_command.verbose, 2);
        assert_eq!(after_command.verbose, 3);
    }

    #[test]
    fn top_level_trade_aliases_normalize_to_canonical_paths() {
        assert_eq!(
            normalize_alias_args(["volumeleaders-agent", "trades", "NVDA"].map(str::to_string)),
            ["volumeleaders-agent", "trade", "list", "NVDA"]
        );
        assert_eq!(
            normalize_alias_args(["volumeleaders-agent", "dashboard", "NVDA"].map(str::to_string)),
            ["volumeleaders-agent", "trade", "dashboard", "NVDA"]
        );
        assert_eq!(
            normalize_alias_args(["volumeleaders-agent", "levels", "NVDA"].map(str::to_string)),
            ["volumeleaders-agent", "trade", "levels", "NVDA"]
        );
    }

    #[test]
    fn top_level_trade_aliases_normalize_after_global_flags() {
        assert_eq!(
            normalize_alias_args(
                [
                    "volumeleaders-agent",
                    "--strict-empty",
                    "-vv",
                    "trades",
                    "NVDA"
                ]
                .map(str::to_string)
            ),
            [
                "volumeleaders-agent",
                "--strict-empty",
                "-vv",
                "trade",
                "list",
                "NVDA"
            ]
        );

        assert_eq!(
            normalize_alias_args(
                ["volumeleaders-agent", "trade", "list", "NVDA"].map(str::to_string)
            ),
            ["volumeleaders-agent", "trade", "list", "NVDA"]
        );
    }

    #[test]
    fn top_level_trade_aliases_leave_flag_only_invocations_unchanged() {
        assert_eq!(
            normalize_alias_args(["volumeleaders-agent", "--version"].map(str::to_string)),
            ["volumeleaders-agent", "--version"]
        );
    }

    #[test]
    fn every_leaf_command_has_long_about_examples() {
        let command = Cli::command();
        let mut missing = Vec::new();

        collect_leaf_commands(&command, &mut Vec::new(), &mut |path, command| {
            missing.extend(missing_long_about_examples(path, command));
        });

        assert!(
            missing.is_empty(),
            "leaf commands missing concise about and two long_about examples: {missing:?}"
        );
    }

    #[test]
    fn long_about_example_check_reports_missing_metadata() {
        let command = Command::new("volumeleaders-agent").subcommand(Command::new("broken"));
        let mut missing = Vec::new();

        collect_leaf_commands(&command, &mut Vec::new(), &mut |path, command| {
            missing.extend(missing_long_about_examples(path, command));
        });

        assert_eq!(missing, vec!["broken"]);
    }

    #[test]
    fn long_about_example_check_accepts_complete_metadata() {
        let command = Command::new("good").about("Good command").long_about(
            "Good command.\n\nExamples:\n  volumeleaders-agent good\n  volumeleaders-agent good --flag",
        );
        let path = ["good".to_string()];

        assert_eq!(missing_long_about_examples(&path, &command), None);
    }

    fn collect_leaf_commands(
        command: &Command,
        path: &mut Vec<String>,
        visit: &mut impl FnMut(&[String], &Command),
    ) {
        for subcommand in command
            .get_subcommands()
            .filter(|command| !command.is_hide_set())
        {
            path.push(subcommand.get_name().to_string());
            if subcommand.has_subcommands() {
                collect_leaf_commands(subcommand, path, visit);
            } else {
                visit(path, subcommand);
            }
            path.pop();
        }
    }

    fn example_line_count(long_about: &str) -> usize {
        long_about
            .lines()
            .filter(|line| line.trim_start().starts_with("volumeleaders-agent "))
            .count()
    }

    fn missing_long_about_examples(path: &[String], command: &Command) -> Option<String> {
        let about = command.get_about().map(ToString::to_string);
        let long_about = command.get_long_about().map(ToString::to_string);
        let example_count = long_about
            .as_deref()
            .map(example_line_count)
            .unwrap_or_default();

        (about.as_deref().is_none_or(str::is_empty)
            || long_about
                .as_deref()
                .is_none_or(|text| !text.contains("Examples:"))
            || example_count < 2)
            .then(|| path.join(" "))
    }
}
