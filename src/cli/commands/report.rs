//! Report commands: preset trade scans and listing.

use std::{collections::HashMap, io};

use clap::{Args, Subcommand};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use tracing::instrument;

use crate::cli::ReportArgs;
use crate::cli::common::DATE_FMT;
use crate::cli::common::TRADE_HEADERS;
use crate::cli::common::auth::{handle_api_error, make_client};
use crate::cli::common::dates::resolve_date_range;
use crate::cli::common::tickers::parse_tickers;
use crate::cli::common::trade_transforms::TradeRecordKind;
use crate::cli::common::types::SummaryGroup;
use crate::cli::error::usage_error;
use crate::cli::field_metadata;
use crate::cli::output::{
    finish_output, print_json, print_transformed_record_values, selected_fields,
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
    /// API filter key-value pairs sent as extra form values.
    pub filters: &'static [(&'static str, &'static str)],
}

/// All available report presets, ported from the Go source.
pub static REPORT_PRESETS: &[ReportPreset] = &[
    ReportPreset {
        use_name: "top-100-rank",
        display_name: "Top 100 Rank",
        short: "Top 100 ranked institutional trades",
        filters: &[
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
        ],
    },
    ReportPreset {
        use_name: "top-10-rank",
        display_name: "Top 10 Rank",
        short: "Top 10 ranked institutional trades",
        filters: &[
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
            ("TradeRank", "10"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "0"),
        ],
    },
    ReportPreset {
        use_name: "dark-pool-sweeps",
        display_name: "Dark Pool Sweeps",
        short: "Dark pool sweep trades",
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("DarkPools", "1"),
            ("EvenShared", "-1"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOffsetting", "-1"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "0"),
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
            ("SignaturePrints", "0"),
            ("Sweeps", "1"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "0"),
        ],
    },
    ReportPreset {
        use_name: "disproportionately-large",
        display_name: "Disproportionately Large",
        short: "Disproportionately large trades relative to average",
        filters: &[
            ("Conditions", "-1"),
            ("DarkPools", "-1"),
            ("EvenShared", "-1"),
            ("IncludeAH", "1"),
            ("IncludeClosing", "1"),
            ("IncludeOffsetting", "1"),
            ("IncludeOpening", "1"),
            ("IncludePhantom", "1"),
            ("IncludePremarket", "1"),
            ("IncludeRTH", "1"),
            ("LatePrints", "-1"),
            ("MarketCap", "0"),
            ("MaxDollars", "30000000000"),
            ("MaxPrice", "100000"),
            ("MaxVolume", "2000000000"),
            ("MinDollars", "500000"),
            ("MinPrice", "0"),
            ("MinVolume", "0"),
            ("RelativeSize", "5"),
            ("SecurityTypeKey", "-1"),
            ("SignaturePrints", "-1"),
            ("Sweeps", "-1"),
            ("TradeRank", "-1"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "0"),
        ],
    },
    ReportPreset {
        use_name: "leveraged-etfs",
        display_name: "Leveraged ETFs",
        short: "Institutional trades in leveraged ETFs",
        filters: &[
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
            ("MaxDollars", "1000000000000"),
            ("MaxPrice", "100000"),
            ("MaxVolume", "2000000000"),
            ("MinDollars", "500000"),
            ("MinPrice", "0"),
            ("MinVolume", "10000"),
            ("RelativeSize", "0"),
            ("SectorIndustry", "X B"),
            ("SecurityTypeKey", "-1"),
            ("SignaturePrints", "-1"),
            ("Sweeps", "-1"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "0"),
        ],
    },
    ReportPreset {
        use_name: "rsi-overbought",
        display_name: "RSI Overbought",
        short: "Trades with overbought RSI conditions",
        filters: &[
            ("Conditions", "OBD,OBH"),
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
            ("MaxDollars", "10000000000"),
            ("MaxPrice", "100000"),
            ("MaxVolume", "2000000000"),
            ("MinDollars", "500000"),
            ("MinPrice", "0"),
            ("MinVolume", "10000"),
            ("RelativeSize", "5"),
            ("SecurityTypeKey", "-1"),
            ("SignaturePrints", "0"),
            ("Sweeps", "-1"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "0"),
        ],
    },
    ReportPreset {
        use_name: "rsi-oversold",
        display_name: "RSI Oversold",
        short: "Trades with oversold RSI conditions",
        filters: &[
            ("Conditions", "OSD,OSH"),
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
            ("MaxDollars", "10000000000"),
            ("MaxPrice", "100000"),
            ("MaxVolume", "2000000000"),
            ("MinDollars", "500000"),
            ("MinPrice", "0"),
            ("MinVolume", "10000"),
            ("RelativeSize", "5"),
            ("SecurityTypeKey", "-1"),
            ("SignaturePrints", "0"),
            ("Sweeps", "-1"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "0"),
        ],
    },
    ReportPreset {
        use_name: "dark-pool-20x",
        display_name: "Dark Pool 20x",
        short: "Dark pool trades at 20x relative size",
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("DarkPools", "1"),
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
            ("MaxDollars", "10000000000"),
            ("MaxPrice", "100000"),
            ("MaxVolume", "2000000000"),
            ("MinDollars", "500000"),
            ("MinPrice", "0"),
            ("MinVolume", "10000"),
            ("RelativeSize", "20"),
            ("SecurityTypeKey", "-1"),
            ("SignaturePrints", "0"),
            ("Sweeps", "-1"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "0"),
        ],
    },
    ReportPreset {
        use_name: "top-30-rank-10x-99th",
        display_name: "Top 30 Rank 10x 99th Percentile",
        short: "Top 30 ranked trades at 10x size in the 99th percentile",
        filters: &[
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
            ("MaxDollars", "10000000000"),
            ("MaxPrice", "100000"),
            ("MaxVolume", "2000000000"),
            ("MinDollars", "500000"),
            ("MinPrice", "0"),
            ("MinVolume", "10000"),
            ("RelativeSize", "10"),
            ("SecurityTypeKey", "-1"),
            ("SignaturePrints", "0"),
            ("Sweeps", "-1"),
            ("TradeCount", "3"),
            ("TradeRank", "30"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "99"),
        ],
    },
    ReportPreset {
        use_name: "phantom-trades",
        display_name: "Phantom Trades",
        short: "Phantom print trades (dark pool only)",
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("DarkPools", "1"),
            ("EvenShared", "-1"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOffsetting", "0"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "1"),
            ("IncludePremarket", "0"),
            ("IncludeRTH", "0"),
            ("LatePrints", "-1"),
            ("MarketCap", "0"),
            ("MaxDollars", "100000000000"),
            ("MaxPrice", "100000"),
            ("MaxVolume", "2000000000"),
            ("MinDollars", "500000"),
            ("MinPrice", "0"),
            ("MinVolume", "0"),
            ("RelativeSize", "0"),
            ("SecurityTypeKey", "-1"),
            ("SignaturePrints", "0"),
            ("Sweeps", "-1"),
            ("TradeCount", "3"),
            ("TradeRank", "-1"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "0"),
        ],
    },
    ReportPreset {
        use_name: "offsetting-trades",
        display_name: "Offsetting Trades",
        short: "Offsetting institutional trades",
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("DarkPools", "-1"),
            ("EvenShared", "-1"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOffsetting", "1"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "0"),
            ("IncludePremarket", "0"),
            ("IncludeRTH", "0"),
            ("LatePrints", "-1"),
            ("MarketCap", "0"),
            ("MaxDollars", "100000000000"),
            ("MaxPrice", "100000"),
            ("MaxVolume", "2000000000"),
            ("MinDollars", "500000"),
            ("MinPrice", "0"),
            ("MinVolume", "0"),
            ("RelativeSize", "0"),
            ("SecurityTypeKey", "-1"),
            ("SignaturePrints", "0"),
            ("Sweeps", "-1"),
            ("TradeCount", "3"),
            ("TradeRank", "-1"),
            ("TradeRankSnapshot", "-1"),
            ("VCD", "0"),
        ],
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
        long_about = "Run the dark pool sweep trades preset.\n\nExamples:\n  volumeleaders-agent report dark-pool-sweeps\n  volumeleaders-agent report dark-pool-sweeps --tickers SPY,QQQ --days 3 --fields Ticker,DateTime,Price,Dollars"
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
        long_about = "Run the leveraged ETF institutional trades preset.\n\nExamples:\n  volumeleaders-agent report leveraged-etfs\n  volumeleaders-agent report leveraged-etfs --days 10 --limit 100 --fields Ticker,DateTime,Price,Dollars"
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
    /// Return every field after semantic trade transforms.
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
pub async fn handle(args: &ReportArgs) -> i32 {
    match &args.command {
        ReportCommand::List => execute_list(),
        _ => execute_preset(args).await,
    }
}

/// Lists all available report presets.
#[instrument(skip_all)]
fn execute_list() -> i32 {
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
async fn execute_preset(args: &ReportArgs) -> i32 {
    let preset_name = match args.command.preset_name() {
        Some(name) => name,
        None => return usage_error("unexpected command state"),
    };

    let flags = match args.command.flags() {
        Some(f) => f,
        None => return usage_error("unexpected command state"),
    };

    if flags.summary_group.is_some() && (flags.fields.is_some() || flags.all_fields) {
        return usage_error("--fields and --all-fields cannot be used with summary output");
    }

    if let Err(err) = validate_report_fields(preset_name, flags.fields.as_deref()) {
        return finish_output(Err(err));
    }

    let preset = match REPORT_PRESETS.iter().find(|p| p.use_name == preset_name) {
        Some(p) => p,
        None => return usage_error(format!("unknown preset: {preset_name}")),
    };

    // Build the trade filters first, then hand them to the client request builder.
    let mut filters = preset
        .filters
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
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };

    // Fetch trades.
    let mut trades = match client.get_trades(&request).await {
        Ok(response) => response.data,
        Err(e) => return handle_api_error(e),
    };
    trades.truncate(limit);

    // Output results.
    let result = if let Some(group) = flags.summary_group {
        let summary = build_summary(&trades, group, &start, &end);
        print_json(&summary)
    } else {
        print_transformed_record_values(
            &trades,
            TradeRecordKind::Trade,
            TRADE_HEADERS,
            flags.fields.as_deref(),
            flags.all_fields,
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
mod tests {
    use super::*;
    use crate::{AspNetDate, FlexBool};

    #[test]
    fn preset_count_is_eleven() {
        assert_eq!(REPORT_PRESETS.len(), 11);
    }

    #[test]
    fn preset_names_are_unique() {
        let mut names: Vec<&str> = REPORT_PRESETS.iter().map(|p| p.use_name).collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "preset use_names must be unique");
    }

    #[test]
    fn preset_filters_are_non_empty() {
        for preset in REPORT_PRESETS {
            assert!(
                !preset.filters.is_empty(),
                "preset '{}' must have filters",
                preset.use_name
            );
        }
    }

    #[test]
    fn top_100_rank_has_trade_rank_100() {
        let preset = REPORT_PRESETS
            .iter()
            .find(|p| p.use_name == "top-100-rank")
            .expect("top-100-rank preset must exist");
        let rank = preset
            .filters
            .iter()
            .find(|&&(k, _)| k == "TradeRank")
            .map(|&(_, v)| v);
        assert_eq!(rank, Some("100"));
    }

    #[test]
    fn top_10_rank_has_trade_rank_10() {
        let preset = REPORT_PRESETS
            .iter()
            .find(|p| p.use_name == "top-10-rank")
            .expect("top-10-rank preset must exist");
        let rank = preset
            .filters
            .iter()
            .find(|&&(k, _)| k == "TradeRank")
            .map(|&(_, v)| v);
        assert_eq!(rank, Some("10"));
    }

    #[test]
    fn dark_pool_sweeps_has_correct_filters() {
        let preset = REPORT_PRESETS
            .iter()
            .find(|p| p.use_name == "dark-pool-sweeps")
            .expect("dark-pool-sweeps preset must exist");
        let dark_pools = preset
            .filters
            .iter()
            .find(|&&(k, _)| k == "DarkPools")
            .map(|&(_, v)| v);
        let sweeps = preset
            .filters
            .iter()
            .find(|&&(k, _)| k == "Sweeps")
            .map(|&(_, v)| v);
        assert_eq!(dark_pools, Some("1"));
        assert_eq!(sweeps, Some("1"));
    }

    #[test]
    fn leveraged_etfs_has_sector_industry() {
        let preset = REPORT_PRESETS
            .iter()
            .find(|p| p.use_name == "leveraged-etfs")
            .expect("leveraged-etfs preset must exist");
        let si = preset
            .filters
            .iter()
            .find(|&&(k, _)| k == "SectorIndustry")
            .map(|&(_, v)| v);
        assert_eq!(si, Some("X B"));
    }

    #[test]
    fn top_30_rank_10x_99th_has_correct_vcd() {
        let preset = REPORT_PRESETS
            .iter()
            .find(|p| p.use_name == "top-30-rank-10x-99th")
            .expect("top-30-rank-10x-99th preset must exist");
        let vcd = preset
            .filters
            .iter()
            .find(|&&(k, _)| k == "VCD")
            .map(|&(_, v)| v);
        let rank = preset
            .filters
            .iter()
            .find(|&&(k, _)| k == "TradeRank")
            .map(|&(_, v)| v);
        let rs = preset
            .filters
            .iter()
            .find(|&&(k, _)| k == "RelativeSize")
            .map(|&(_, v)| v);
        assert_eq!(vcd, Some("99"));
        assert_eq!(rank, Some("30"));
        assert_eq!(rs, Some("10"));
    }

    #[test]
    fn list_output_contains_all_presets() {
        let entries: Vec<PresetListEntry> = REPORT_PRESETS
            .iter()
            .map(|p| PresetListEntry {
                name: p.display_name,
                command: format!("report {}", p.use_name),
                description: p.short,
            })
            .collect();
        assert_eq!(entries.len(), 11);
        assert_eq!(entries[0].name, "Top 100 Rank");
        assert_eq!(entries[0].command, "report top-100-rank");
    }

    #[test]
    fn report_request_uses_limit_as_datatables_length() {
        let request = build_report_request(vec![("TradeRank".to_string(), "100".to_string())], 500);

        let encoded = request.encode();

        assert!(encoded.contains("length=500"));
        assert!(encoded.contains("TradeRank=100"));
    }

    fn make_test_trade(
        ticker: &str,
        dollars: f64,
        multiplier: f64,
        dark_pool: bool,
        sweep: bool,
        cd: f64,
    ) -> crate::Trade {
        crate::Trade {
            ticker: Some(ticker.to_string()),
            date: Some(AspNetDate(Some(
                chrono::DateTime::parse_from_rfc3339("2025-06-01T12:00:00Z")
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            ))),
            dollars: rust_decimal::Decimal::try_from(dollars).ok(),
            dollars_multiplier: Some(multiplier),
            dark_pool: Some(FlexBool(Some(dark_pool))),
            sweep: Some(FlexBool(Some(sweep))),
            cumulative_distribution: Some(cd),
            start_date: None,
            end_date: None,
            td_30: None,
            td_90: None,
            td_1cy: None,
            date_key: None,
            time_key: None,
            security_key: None,
            trade_id: None,
            sequence_number: None,
            eom: None,
            eoq: None,
            eoy: None,
            opex: None,
            volex: None,
            sector: None,
            industry: None,
            name: None,
            full_date_time: None,
            full_time_string_24: None,
            price: None,
            bid: None,
            ask: None,
            average_block_size_dollars: None,
            average_block_size_shares: None,
            volume: None,
            average_daily_volume: None,
            percent_daily_volume: None,
            relative_size: None,
            last_comparible_trade_date: None,
            ipo_date: None,
            offsetting_trade_date: None,
            phantom_print_fulfillment_date: None,
            phantom_print_fulfillment_days: None,
            trade_count: None,
            trade_rank: None,
            trade_rank_snapshot: None,
            late_print: None,
            opening_trade: None,
            closing_trade: None,
            phantom_print: None,
            inside_bar: None,
            double_inside_bar: None,
            signature_print: None,
            new_position: None,
            ah_institutional_dollars: None,
            ah_institutional_dollars_rank: None,
            ah_institutional_volume: None,
            total_institutional_dollars: None,
            total_institutional_dollars_rank: None,
            total_institutional_volume: None,
            closing_trade_dollars: None,
            closing_trade_dollars_rank: None,
            closing_trade_volume: None,
            total_dollars: None,
            total_dollars_rank: None,
            total_volume: None,
            close_price: None,
            rsi_hour: None,
            rsi_day: None,
            total_rows: None,
            trade_conditions: None,
            frequency_last_30_td: None,
            frequency_last_90_td: None,
            frequency_last_1cy: None,
            cancelled: None,
            total_trades: None,
            external_feed: None,
        }
    }

    #[test]
    fn summary_by_ticker() {
        let trades = vec![
            make_test_trade("AAPL", 1_000_000.0, 2.5, true, false, 95.0),
            make_test_trade("AAPL", 2_000_000.0, 3.0, false, true, 90.0),
            make_test_trade("MSFT", 500_000.0, 1.5, true, true, 80.0),
        ];

        let summary = build_summary(&trades, SummaryGroup::Ticker, "2025-06-01", "2025-06-05");

        assert_eq!(summary.total_trades, 3);
        assert!((summary.total_dollars - 3_500_000.0).abs() < f64::EPSILON);

        let aapl = summary.groups.get("AAPL").expect("AAPL group");
        assert_eq!(aapl.trades, 2);
        assert!((aapl.dollars - 3_000_000.0).abs() < f64::EPSILON);
        assert!((aapl.avg_dollars_multiplier - 2.75).abs() < f64::EPSILON);
        assert!((aapl.pct_dark_pool - 50.0).abs() < f64::EPSILON);
        assert!((aapl.pct_sweep - 50.0).abs() < f64::EPSILON);
        assert!((aapl.avg_cumulative_distribution - 92.5).abs() < f64::EPSILON);

        let msft = summary.groups.get("MSFT").expect("MSFT group");
        assert_eq!(msft.trades, 1);
        assert!((msft.pct_dark_pool - 100.0).abs() < f64::EPSILON);
        assert!((msft.pct_sweep - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn summary_by_day() {
        let trades = vec![
            make_test_trade("AAPL", 1_000_000.0, 2.5, true, false, 95.0),
            make_test_trade("MSFT", 500_000.0, 1.5, false, true, 80.0),
        ];

        let summary = build_summary(&trades, SummaryGroup::Day, "2025-06-01", "2025-06-05");

        assert_eq!(summary.total_trades, 2);
        // Both trades have the same date, so there should be one group.
        let day_group = summary.groups.get("2025-06-01").expect("2025-06-01 group");
        assert_eq!(day_group.trades, 2);
    }

    #[test]
    fn summary_by_ticker_day() {
        let trades = vec![
            make_test_trade("AAPL", 1_000_000.0, 2.5, true, false, 95.0),
            make_test_trade("AAPL", 2_000_000.0, 3.0, false, true, 90.0),
        ];

        let summary = build_summary(&trades, SummaryGroup::TickerDay, "2025-06-01", "2025-06-05");

        let key = "AAPL|2025-06-01";
        let group = summary.groups.get(key).expect("AAPL|2025-06-01 group");
        assert_eq!(group.trades, 2);
    }

    #[test]
    fn summary_empty_trades() {
        let trades: Vec<crate::Trade> = vec![];
        let summary = build_summary(&trades, SummaryGroup::Ticker, "2025-06-01", "2025-06-05");

        assert_eq!(summary.total_trades, 0);
        assert!((summary.total_dollars - 0.0).abs() < f64::EPSILON);
        assert!(summary.groups.is_empty());
    }

    #[test]
    fn command_preset_name_returns_correct_names() {
        let flags = ReportFlags {
            tickers: None,
            start_date: None,
            end_date: None,
            days: None,
            limit: None,
            summary_group: None,
            fields: None,
            all_fields: false,
        };

        assert_eq!(
            ReportCommand::Top100Rank(flags.clone()).preset_name(),
            Some("top-100-rank")
        );

        // List has no preset name.
        assert_eq!(ReportCommand::List.preset_name(), None);
    }

    #[test]
    fn validate_report_fields_uses_preset_metadata() {
        validate_report_fields("top-100-rank", Some("Ticker,Dollars")).unwrap();
        validate_report_fields("top-10-rank", Some("RelativeSize")).unwrap();

        let multiplier_err =
            validate_report_fields("top-10-rank", Some("DollarsMultiplier")).unwrap_err();
        assert!(multiplier_err.to_string().contains("DollarsMultiplier"));

        let err = validate_report_fields("top-100-rank", Some("ticker")).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("ticker"));
        assert!(err.to_string().contains("Ticker"));
    }

    #[test]
    fn validate_report_fields_skips_empty_or_unknown_metadata_paths() {
        validate_report_fields("top-100-rank", None).unwrap();
        validate_report_fields("top-100-rank", Some("all")).unwrap();
        validate_report_fields("unknown-report", Some("Ticker")).unwrap();
    }

    #[tokio::test]
    async fn execute_preset_rejects_invalid_fields_before_auth() {
        let flags = ReportFlags {
            tickers: None,
            start_date: None,
            end_date: None,
            days: None,
            limit: None,
            summary_group: None,
            fields: Some("ticker".to_string()),
            all_fields: false,
        };
        let args = ReportArgs {
            command: ReportCommand::Top100Rank(flags),
        };

        assert_eq!(
            execute_preset(&args).await,
            crate::cli::error::EXIT_USAGE_ERROR
        );
    }
}
