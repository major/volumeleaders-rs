//! Report commands: preset trade scans and listing.

use std::{collections::HashMap, io};

use clap::{Args, Subcommand};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use tracing::instrument;

use crate::cli::ReportArgs;
use crate::cli::common::DATE_FMT;
use crate::cli::common::auth::make_client;
use crate::cli::common::dates::resolve_date_range;
use crate::cli::common::tickers::parse_tickers;
use crate::cli::common::types::SummaryGroup;
use crate::cli::error::{CliExit, usage_error};
use crate::cli::field_metadata;
use crate::cli::field_metadata::TRADE_HEADERS;
use crate::cli::output::{
    finish_output, print_json, print_records_with_allowed_fields, selected_fields,
};

/// Default trade limit when none is specified on the command line.
const DEFAULT_LIMIT: usize = 500;

/// A preset report definition with its API filter parameters.
#[derive(Debug)]
pub struct ReportPreset {
    /// Kebab-case name used as the CLI subcommand.
    pub use_name: &'static str,
    /// Human-readable display name.
    pub display_name: &'static str,
    /// Short description shown in help text.
    pub short: &'static str,
    /// API filter key-value pairs that replace or extend the shared defaults.
    pub overrides: &'static [(&'static str, &'static str)],
    /// Shared default filter keys omitted by this preset.
    pub omitted_filters: &'static [&'static str],
}

impl ReportPreset {
    /// Return the complete API filter set for this preset.
    #[must_use]
    pub fn filters(&self) -> Vec<(&'static str, &'static str)> {
        let mut filters = BASE_REPORT_FILTERS
            .iter()
            .copied()
            .filter(|(key, _)| !self.omitted_filters.contains(key))
            .collect::<Vec<_>>();

        for &(key, value) in self.overrides {
            if let Some((_, existing)) = filters.iter_mut().find(|(existing, _)| existing == &key) {
                *existing = value;
            } else {
                filters.push((key, value));
            }
        }

        filters
    }
}

/// Shared report filter defaults. Presets list only their differences below.
const BASE_REPORT_FILTERS: &[(&str, &str)] = &[
    ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
    ("DarkPools", "-1"),
    ("EvenShared", "-1"),
    ("IncludeAH", "1"),
    ("IncludeClosing", "1"),
    ("IncludeOffsetting", "-1"),
    ("IncludeOpening", "1"),
    ("IncludePhantom", "-1"),
    ("IncludePremarket", "1"),
    ("IncludeRTH", "1"),
    ("LatePrints", "-1"),
    ("MarketCap", "0"),
    ("MaxDollars", "100000000000"),
    ("MaxPrice", "100000"),
    ("MaxVolume", "2000000000"),
    ("MinDollars", "500000"),
    ("MinPrice", "0"),
    ("MinVolume", "10000"),
    ("RelativeSize", "0"),
    ("SecurityTypeKey", "-1"),
    ("SignaturePrints", "-1"),
    ("Sweeps", "-1"),
    ("TradeCount", "3"),
    ("TradeRank", "100"),
    ("TradeRankSnapshot", "-1"),
    ("VCD", "0"),
];

/// All available report presets, ported from the Go source.
pub static REPORT_PRESETS: &[ReportPreset] = &[
    ReportPreset {
        use_name: "top-100-rank",
        display_name: "Top 100 Rank",
        short: "Top 100 ranked institutional trades",
        overrides: &[],
        omitted_filters: &[],
    },
    ReportPreset {
        use_name: "top-10-rank",
        display_name: "Top 10 Rank",
        short: "Top 10 ranked institutional trades",
        overrides: &[("TradeRank", "10")],
        omitted_filters: &[],
    },
    ReportPreset {
        use_name: "dark-pool-sweeps",
        display_name: "Dark Pool Sweeps",
        short: "Dark pool sweep trades",
        overrides: &[
            ("DarkPools", "1"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "0"),
            ("SignaturePrints", "0"),
            ("Sweeps", "1"),
        ],
        omitted_filters: &[],
    },
    ReportPreset {
        use_name: "disproportionately-large",
        display_name: "Disproportionately Large",
        short: "Disproportionately large trades relative to average",
        overrides: &[
            ("Conditions", "-1"),
            ("IncludeOffsetting", "1"),
            ("IncludePhantom", "1"),
            ("MaxDollars", "30000000000"),
            ("MinVolume", "0"),
            ("RelativeSize", "5"),
            ("TradeRank", "-1"),
        ],
        omitted_filters: &["TradeCount"],
    },
    ReportPreset {
        use_name: "leveraged-etfs",
        display_name: "Leveraged ETFs",
        short: "Institutional trades in leveraged ETFs",
        overrides: &[("MaxDollars", "1000000000000"), ("SectorIndustry", "X B")],
        omitted_filters: &[],
    },
    ReportPreset {
        use_name: "rsi-overbought",
        display_name: "RSI Overbought",
        short: "Trades with overbought RSI conditions",
        overrides: &[
            ("Conditions", "OBD,OBH"),
            ("MaxDollars", "10000000000"),
            ("RelativeSize", "5"),
            ("SignaturePrints", "0"),
        ],
        omitted_filters: &[],
    },
    ReportPreset {
        use_name: "rsi-oversold",
        display_name: "RSI Oversold",
        short: "Trades with oversold RSI conditions",
        overrides: &[
            ("Conditions", "OSD,OSH"),
            ("MaxDollars", "10000000000"),
            ("RelativeSize", "5"),
            ("SignaturePrints", "0"),
        ],
        omitted_filters: &[],
    },
    ReportPreset {
        use_name: "dark-pool-20x",
        display_name: "Dark Pool 20x",
        short: "Dark pool trades at 20x relative size",
        overrides: &[
            ("DarkPools", "1"),
            ("MaxDollars", "10000000000"),
            ("RelativeSize", "20"),
            ("SignaturePrints", "0"),
        ],
        omitted_filters: &[],
    },
    ReportPreset {
        use_name: "top-30-rank-10x-99th",
        display_name: "Top 30 Rank 10x 99th Percentile",
        short: "Top 30 ranked trades at 10x size in the 99th percentile",
        overrides: &[
            ("MaxDollars", "10000000000"),
            ("RelativeSize", "10"),
            ("SignaturePrints", "0"),
            ("TradeRank", "30"),
            ("VCD", "99"),
        ],
        omitted_filters: &[],
    },
    ReportPreset {
        use_name: "phantom-trades",
        display_name: "Phantom Trades",
        short: "Phantom print trades (dark pool only)",
        overrides: &[
            ("DarkPools", "1"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOffsetting", "0"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "1"),
            ("IncludePremarket", "0"),
            ("IncludeRTH", "0"),
            ("MinVolume", "0"),
            ("SignaturePrints", "0"),
            ("TradeRank", "-1"),
        ],
        omitted_filters: &[],
    },
    ReportPreset {
        use_name: "offsetting-trades",
        display_name: "Offsetting Trades",
        short: "Offsetting institutional trades",
        overrides: &[
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOffsetting", "1"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "0"),
            ("IncludePremarket", "0"),
            ("IncludeRTH", "0"),
            ("MinVolume", "0"),
            ("SignaturePrints", "0"),
            ("TradeRank", "-1"),
        ],
        omitted_filters: &[],
    },
];
/// Report subcommands: list presets or run a specific preset.
#[derive(Debug, Subcommand)]
pub enum ReportCommand {
    /// List available report presets.
    #[command(
        long_about = "List available report presets and their command names.\n\nExamples:\n  volumeleaders-agent report list\n  volumeleaders-agent report list | jq '.[].command'"
    )]
    List,
    /// Top 100 ranked institutional trades.
    #[command(
        name = "top-100-rank",
        long_about = "Run the top 100 ranked institutional trades preset.\n\nExamples:\n  volumeleaders-agent report top-100-rank\n  volumeleaders-agent report top-100-rank --tickers NVDA,AAPL --days 5 --limit 50"
    )]
    Top100Rank(#[command(flatten)] ReportFlags),
    /// Top 10 ranked institutional trades.
    #[command(
        name = "top-10-rank",
        long_about = "Run the top 10 ranked institutional trades preset.\n\nExamples:\n  volumeleaders-agent report top-10-rank\n  volumeleaders-agent report top-10-rank --tickers NVDA --start-date 2026-05-01 --end-date 2026-05-27"
    )]
    Top10Rank(#[command(flatten)] ReportFlags),
    /// Dark pool sweep trades.
    #[command(
        name = "dark-pool-sweeps",
        long_about = "Run the dark pool sweep trades preset.\n\nExamples:\n  volumeleaders-agent report dark-pool-sweeps\n  volumeleaders-agent report dark-pool-sweeps --tickers SPY,QQQ --days 3 --fields FullTimeString24,Price,Dollars,DollarsMultiplier"
    )]
    DarkPoolSweeps(#[command(flatten)] ReportFlags),
    /// Disproportionately large trades relative to average.
    #[command(
        name = "disproportionately-large",
        long_about = "Run the disproportionately large trades preset.\n\nExamples:\n  volumeleaders-agent report disproportionately-large\n  volumeleaders-agent report disproportionately-large --tickers AAPL --limit 25 --summary-group ticker"
    )]
    DisproportionatelyLarge(#[command(flatten)] ReportFlags),
    /// Institutional trades in leveraged ETFs.
    #[command(
        name = "leveraged-etfs",
        long_about = "Run the leveraged ETF institutional trades preset.\n\nExamples:\n  volumeleaders-agent report leveraged-etfs\n  volumeleaders-agent report leveraged-etfs --days 10 --limit 100 --fields FullTimeString24,Price,Dollars,DollarsMultiplier"
    )]
    LeveragedEtfs(#[command(flatten)] ReportFlags),
    /// Trades with overbought RSI conditions.
    #[command(
        name = "rsi-overbought",
        long_about = "Run the overbought RSI trades preset.\n\nExamples:\n  volumeleaders-agent report rsi-overbought\n  volumeleaders-agent report rsi-overbought --tickers NVDA,MSFT --days 7 --limit 40"
    )]
    RsiOverbought(#[command(flatten)] ReportFlags),
    /// Trades with oversold RSI conditions.
    #[command(
        name = "rsi-oversold",
        long_about = "Run the oversold RSI trades preset.\n\nExamples:\n  volumeleaders-agent report rsi-oversold\n  volumeleaders-agent report rsi-oversold --tickers TSLA --start-date 2026-05-01 --end-date 2026-05-27"
    )]
    RsiOversold(#[command(flatten)] ReportFlags),
    /// Dark pool trades at 20x relative size.
    #[command(
        name = "dark-pool-20x",
        long_about = "Run the dark pool trades at 20x relative size preset.\n\nExamples:\n  volumeleaders-agent report dark-pool-20x\n  volumeleaders-agent report dark-pool-20x --tickers AAPL,NVDA --days 5 --limit 50"
    )]
    DarkPool20x(#[command(flatten)] ReportFlags),
    /// Top 30 ranked trades at 10x size in the 99th percentile.
    #[command(
        name = "top-30-rank-10x-99th",
        long_about = "Run the top 30 ranked 10x size 99th percentile preset.\n\nExamples:\n  volumeleaders-agent report top-30-rank-10x-99th\n  volumeleaders-agent report top-30-rank-10x-99th --tickers QQQ --days 2 --all-fields"
    )]
    Top30Rank10x99th(#[command(flatten)] ReportFlags),
    /// Phantom print trades (dark pool only).
    #[command(
        name = "phantom-trades",
        long_about = "Run the phantom print trades preset.\n\nExamples:\n  volumeleaders-agent report phantom-trades\n  volumeleaders-agent report phantom-trades --tickers SPY --start-date 2026-05-01 --end-date 2026-05-27"
    )]
    PhantomTrades(#[command(flatten)] ReportFlags),
    /// Offsetting institutional trades.
    #[command(
        name = "offsetting-trades",
        long_about = "Run the offsetting institutional trades preset.\n\nExamples:\n  volumeleaders-agent report offsetting-trades\n  volumeleaders-agent report offsetting-trades --tickers NVDA --days 5 --summary-group ticker"
    )]
    OffsettingTrades(#[command(flatten)] ReportFlags),
}

/// Shared flags for all preset report commands.
#[derive(Clone, Debug, Args)]
pub struct ReportFlags {
    /// Comma-separated ticker symbols to filter by.
    #[arg(short, long)]
    pub tickers: Option<String>,

    /// Start date (YYYY-MM-DD). Defaults to 5 days ago.
    #[arg(short, long)]
    pub start_date: Option<String>,

    /// End date (YYYY-MM-DD). Defaults to today.
    #[arg(short, long)]
    pub end_date: Option<String>,

    /// Number of days to look back (overrides start/end dates).
    #[arg(short, long)]
    pub days: Option<u32>,

    /// Maximum number of trades to return.
    #[arg(short, long)]
    pub limit: Option<usize>,

    /// Group results into a summary by ticker, day, or both.
    #[arg(long, value_enum)]
    pub summary_group: Option<SummaryGroup>,
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields report top-100-rank`, `fields report dark-pool-sweeps`, or the matching report path.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every raw API field.
    #[arg(long)]
    pub all_fields: bool,
}

impl ReportCommand {
    /// Returns the preset use_name for preset commands, or None for List.
    fn preset_name(&self) -> Option<&'static str> {
        match self {
            Self::List => None,
            Self::Top100Rank(_) => Some("top-100-rank"),
            Self::Top10Rank(_) => Some("top-10-rank"),
            Self::DarkPoolSweeps(_) => Some("dark-pool-sweeps"),
            Self::DisproportionatelyLarge(_) => Some("disproportionately-large"),
            Self::LeveragedEtfs(_) => Some("leveraged-etfs"),
            Self::RsiOverbought(_) => Some("rsi-overbought"),
            Self::RsiOversold(_) => Some("rsi-oversold"),
            Self::DarkPool20x(_) => Some("dark-pool-20x"),
            Self::Top30Rank10x99th(_) => Some("top-30-rank-10x-99th"),
            Self::PhantomTrades(_) => Some("phantom-trades"),
            Self::OffsettingTrades(_) => Some("offsetting-trades"),
        }
    }

    /// Returns the ReportFlags for preset commands, or None for List.
    fn flags(&self) -> Option<&ReportFlags> {
        match self {
            Self::List => None,
            Self::Top100Rank(f)
            | Self::Top10Rank(f)
            | Self::DarkPoolSweeps(f)
            | Self::DisproportionatelyLarge(f)
            | Self::LeveragedEtfs(f)
            | Self::RsiOverbought(f)
            | Self::RsiOversold(f)
            | Self::DarkPool20x(f)
            | Self::Top30Rank10x99th(f)
            | Self::PhantomTrades(f)
            | Self::OffsettingTrades(f) => Some(f),
        }
    }
}

/// Handles the report command group.
#[instrument(skip_all)]
pub async fn handle(args: &ReportArgs) -> Result<(), CliExit> {
    match &args.command {
        ReportCommand::List => execute_list(),
        _ => execute_preset(args).await,
    }
}

/// Lists all available report presets.
#[instrument(skip_all)]
fn execute_list() -> Result<(), CliExit> {
    let entries: Vec<PresetListEntry> = REPORT_PRESETS
        .iter()
        .map(|p| PresetListEntry {
            name: p.display_name,
            command: format!("report {}", p.use_name),
            description: p.short,
        })
        .collect();

    finish_output(print_json(&entries))
}

/// Runs a preset report: builds request from preset filters + CLI overrides,
/// fetches trades, and outputs results.
#[instrument(skip_all)]
async fn execute_preset(args: &ReportArgs) -> Result<(), CliExit> {
    let preset_name = match args.command.preset_name() {
        Some(name) => name,
        None => return Err(usage_error("unexpected command state")),
    };

    let flags = match args.command.flags() {
        Some(f) => f,
        None => return Err(usage_error("unexpected command state")),
    };

    if flags.summary_group.is_some() && (flags.fields.is_some() || flags.all_fields) {
        return Err(usage_error(
            "--fields and --all-fields cannot be used with summary output",
        ));
    }

    if let Err(err) = validate_report_fields(preset_name, flags.fields.as_deref()) {
        return Err(err.into());
    }

    let preset = match REPORT_PRESETS.iter().find(|p| p.use_name == preset_name) {
        Some(p) => p,
        None => return Err(usage_error(format!("unknown preset: {preset_name}"))),
    };

    // Build the trade filters first, then hand them to the client request builder.
    let mut filters = preset
        .filters()
        .iter()
        .map(|&(key, value)| (key.to_string(), value.to_string()))
        .collect::<Vec<_>>();

    // Resolve date range and add to request.
    let (start, end) = resolve_date_range(
        flags.start_date.as_deref(),
        flags.end_date.as_deref(),
        flags.days,
    );
    filters.push(("StartDate".to_string(), start.clone()));
    filters.push(("EndDate".to_string(), end.clone()));

    // Add ticker filter if specified.
    if let Some(ref tickers_str) = flags.tickers {
        let tickers = parse_tickers(tickers_str);
        for ticker in &tickers {
            filters.push(("Tickers".to_string(), ticker.clone()));
        }
    }

    let limit = flags.limit.unwrap_or(DEFAULT_LIMIT);
    let request = build_report_request(filters, limit);

    // Authenticate and create client.
    let client = make_client().await?;

    // Fetch trades.
    let mut trades = client.get_trades(&request).await?.data;
    trades.truncate(limit);

    // Output results.
    let result = if let Some(group) = flags.summary_group {
        let summary = build_summary(&trades, group, &start, &end);
        print_json(&summary)
    } else {
        let allowed_fields = field_metadata::field_names(&report_command_path(preset_name));
        print_records_with_allowed_fields(
            &trades,
            TRADE_HEADERS,
            flags.fields.as_deref(),
            flags.all_fields,
            allowed_fields.as_deref(),
        )
    };

    finish_output(result)
}

fn build_report_request(filters: Vec<(String, String)>, limit: usize) -> crate::TradesRequest {
    let length = i32::try_from(limit).unwrap_or(i32::MAX);
    crate::TradesRequest::new()
        .with_length(length)
        .with_search("", false)
        .with_order(1, "DESC", "FullTimeString24")
        .with_trade_filters(filters)
}

fn report_command_path(preset_name: &str) -> String {
    format!("report {preset_name}")
}

fn validate_report_fields(preset_name: &str, fields: Option<&str>) -> io::Result<()> {
    let Some(fields) = selected_fields(fields) else {
        return Ok(());
    };
    let Some(available) = field_metadata::field_names(&report_command_path(preset_name)) else {
        return Ok(());
    };

    let missing = fields
        .iter()
        .filter(|field| !available.iter().any(|available| available == *field))
        .map(String::as_str)
        .collect::<Vec<_>>();

    if missing.is_empty() {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "unknown output field(s): {}. Available fields: {}",
            missing.join(", "),
            available.join(", ")
        ),
    ))
}

/// Entry for the preset list output.
#[derive(Debug, Serialize)]
struct PresetListEntry {
    name: &'static str,
    command: String,
    description: &'static str,
}

/// Summary output structure.
#[derive(Debug, Serialize)]
struct ReportSummary {
    date_range: DateRange,
    total_trades: usize,
    total_dollars: f64,
    groups: HashMap<String, GroupStats>,
}

/// Date range metadata in summary output.
#[derive(Debug, Serialize)]
struct DateRange {
    start: String,
    end: String,
}

/// Aggregated statistics for a single group.
#[derive(Debug, Serialize)]
struct GroupStats {
    trades: usize,
    dollars: f64,
    avg_dollars_multiplier: f64,
    pct_dark_pool: f64,
    pct_sweep: f64,
    avg_cumulative_distribution: f64,
}

/// Builds a summary of trades grouped by the specified dimension.
fn build_summary(
    trades: &[crate::Trade],
    group: SummaryGroup,
    start: &str,
    end: &str,
) -> ReportSummary {
    let mut groups: HashMap<String, Vec<&crate::Trade>> = HashMap::new();

    for trade in trades {
        let key = match group {
            SummaryGroup::Ticker => trade.ticker.as_deref().unwrap_or("unknown").to_string(),
            SummaryGroup::Day => trade
                .date
                .as_ref()
                .and_then(|d| d.0.map(|dt| dt.format(DATE_FMT).to_string()))
                .unwrap_or_else(|| "unknown".to_string()),
            SummaryGroup::TickerDay => {
                let ticker = trade.ticker.as_deref().unwrap_or("unknown");
                let day = trade
                    .date
                    .as_ref()
                    .and_then(|d| d.0.map(|dt| dt.format(DATE_FMT).to_string()))
                    .unwrap_or_else(|| "unknown".to_string());
                format!("{ticker}|{day}")
            }
        };
        groups.entry(key).or_default().push(trade);
    }

    let total_dollars: f64 = trades
        .iter()
        .filter_map(|t| t.dollars.and_then(|d| d.to_f64()))
        .sum();

    let group_stats: HashMap<String, GroupStats> = groups
        .into_iter()
        .map(|(key, group_trades)| {
            let count = group_trades.len();
            let dollars: f64 = group_trades
                .iter()
                .filter_map(|t| t.dollars.and_then(|d| d.to_f64()))
                .sum();

            let multipliers: Vec<f64> = group_trades
                .iter()
                .filter_map(|t| t.dollars_multiplier)
                .collect();
            let avg_multiplier = if multipliers.is_empty() {
                0.0
            } else {
                multipliers.iter().sum::<f64>() / multipliers.len() as f64
            };

            let dark_pool_count = group_trades
                .iter()
                .filter(|t| t.dark_pool.as_ref().is_some_and(|dp| dp.0 == Some(true)))
                .count();
            let pct_dark_pool = if count == 0 {
                0.0
            } else {
                (dark_pool_count as f64 / count as f64) * 100.0
            };

            let sweep_count = group_trades
                .iter()
                .filter(|t| t.sweep.as_ref().is_some_and(|s| s.0 == Some(true)))
                .count();
            let pct_sweep = if count == 0 {
                0.0
            } else {
                (sweep_count as f64 / count as f64) * 100.0
            };

            let cds: Vec<f64> = group_trades
                .iter()
                .filter_map(|t| t.cumulative_distribution)
                .collect();
            let avg_cd = if cds.is_empty() {
                0.0
            } else {
                cds.iter().sum::<f64>() / cds.len() as f64
            };

            (
                key,
                GroupStats {
                    trades: count,
                    dollars,
                    avg_dollars_multiplier: avg_multiplier,
                    pct_dark_pool,
                    pct_sweep,
                    avg_cumulative_distribution: avg_cd,
                },
            )
        })
        .collect();

    ReportSummary {
        date_range: DateRange {
            start: start.to_string(),
            end: end.to_string(),
        },
        total_trades: trades.len(),
        total_dollars,
        groups: group_stats,
    }
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
