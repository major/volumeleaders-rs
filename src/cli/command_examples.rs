//! Structured command examples for machine-readable CLI schema output.

use serde::Serialize;

/// Copy-pasteable command example metadata for schema consumers.
#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct CommandExample {
    /// Short human-readable purpose for the example.
    pub(crate) description: &'static str,
    /// Complete command invocation.
    pub(crate) command: &'static str,
    /// Optional caveat or usage note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) notes: Option<&'static str>,
}

/// Return structured examples for a canonical command path.
pub(crate) fn examples_for_path(path: &[String]) -> &'static [CommandExample] {
    match path {
        [command] if command == "doctor" => DOCTOR_EXAMPLES,
        [command] if command == "commands" => COMMANDS_EXAMPLES,
        [command] if command == "schema" => SCHEMA_EXAMPLES,
        [group, command] if group == "trade" && command == "list" => TRADE_LIST_EXAMPLES,
        [group, command] if group == "trade" && command == "dashboard" => TRADE_DASHBOARD_EXAMPLES,
        [group, command] if group == "trade" && command == "levels" => TRADE_LEVELS_EXAMPLES,
        [group, command] if group == "report" && command == "list" => REPORT_LIST_EXAMPLES,
        [group, command] if group == "report" && command == "dark-pool-sweeps" => {
            REPORT_DARK_POOL_SWEEPS_EXAMPLES
        }
        [group, command] if group == "volume" && command == "institutional" => {
            VOLUME_INSTITUTIONAL_EXAMPLES
        }
        [group, command] if group == "market" && command == "earnings" => MARKET_EARNINGS_EXAMPLES,
        [group, command] if group == "watchlist" && command == "tickers" => {
            WATCHLIST_TICKERS_EXAMPLES
        }
        _ => &[],
    }
}

macro_rules! example {
    ($description:expr, $command:expr) => {
        CommandExample {
            description: $description,
            command: $command,
            notes: None,
        }
    };
}

const DOCTOR_EXAMPLES: &[CommandExample] = &[
    example!(
        "Check local auth readiness and recovery actions",
        "volumeleaders-agent doctor"
    ),
    example!(
        "Run a live authenticated connectivity check",
        "volumeleaders-agent doctor --live"
    ),
];

const COMMANDS_EXAMPLES: &[CommandExample] = &[
    example!("List every leaf command", "volumeleaders-agent commands"),
    example!(
        "List commands grouped by top-level area",
        "volumeleaders-agent commands --grouped"
    ),
];

const SCHEMA_EXAMPLES: &[CommandExample] = &[
    example!(
        "Emit the full machine-readable CLI schema",
        "volumeleaders-agent schema"
    ),
    example!(
        "Inspect metadata for the trade list command",
        "volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == \"trade list\")'"
    ),
];

const TRADE_LIST_EXAMPLES: &[CommandExample] = &[
    example!(
        "List recent trades for a ticker",
        "volumeleaders-agent trade list NVDA"
    ),
    example!(
        "List selected trade fields over a date range",
        "volumeleaders-agent trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields Ticker,DateTime,Price,Dollars"
    ),
];

const TRADE_DASHBOARD_EXAMPLES: &[CommandExample] = &[
    example!(
        "Show the trade dashboard for a ticker",
        "volumeleaders-agent trade dashboard NVDA"
    ),
    example!(
        "Show dashboard data for a date range",
        "volumeleaders-agent trade dashboard NVDA --start-date 2026-05-01 --end-date 2026-05-27"
    ),
];

const TRADE_LEVELS_EXAMPLES: &[CommandExample] = &[
    example!(
        "Show trade levels for a ticker",
        "volumeleaders-agent trade levels NVDA"
    ),
    example!(
        "Show selected level fields",
        "volumeleaders-agent trade levels NVDA --fields Ticker,Price,TradeLevelRank"
    ),
];

const REPORT_LIST_EXAMPLES: &[CommandExample] = &[
    example!(
        "List available report presets",
        "volumeleaders-agent report list"
    ),
    example!(
        "List report preset names with jq",
        "volumeleaders-agent report list | jq '.[].name'"
    ),
];

const REPORT_DARK_POOL_SWEEPS_EXAMPLES: &[CommandExample] = &[
    example!(
        "Run the dark-pool sweeps report preset",
        "volumeleaders-agent report dark-pool-sweeps"
    ),
    example!(
        "Run the preset with selected fields",
        "volumeleaders-agent report dark-pool-sweeps --fields Ticker,DateTime,Price,Dollars"
    ),
];

const VOLUME_INSTITUTIONAL_EXAMPLES: &[CommandExample] = &[
    example!(
        "Show institutional volume for one ticker",
        "volumeleaders-agent volume institutional --date 2026-05-27 --tickers AAPL"
    ),
    example!(
        "Limit institutional volume rows across tickers",
        "volumeleaders-agent volume institutional --date 2026-05-27 --tickers AAPL,NVDA --limit 50"
    ),
];

const MARKET_EARNINGS_EXAMPLES: &[CommandExample] = &[
    example!(
        "Show upcoming earnings",
        "volumeleaders-agent market earnings"
    ),
    example!(
        "Show earnings over a date range",
        "volumeleaders-agent market earnings --start-date 2026-05-01 --end-date 2026-05-27"
    ),
];

const WATCHLIST_TICKERS_EXAMPLES: &[CommandExample] = &[
    example!(
        "List tickers for a watchlist",
        "volumeleaders-agent watchlist tickers --watchlist-key 123"
    ),
    example!(
        "List selected watchlist ticker fields",
        "volumeleaders-agent watchlist tickers --watchlist-key 123 --fields Ticker,AddedDate"
    ),
];
