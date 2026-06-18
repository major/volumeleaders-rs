//! Trade commands: trades, dashboards, sentiment, clusters, alerts, and levels.

use std::collections::HashMap;

mod dashboard;
mod filters;
mod presets;
mod sentiment;

use crate::{
    TradeClusterBombsRequest, TradeClustersRequest, TradeLevelTouchesRequest, TradesRequest,
};
use clap::{Args, Subcommand};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use tracing::instrument;

use crate::cli::TradeArgs;
use crate::cli::common::DATE_FMT;
use crate::cli::common::auth::make_client;
use crate::cli::common::dates::resolve_date_range;
use crate::cli::common::tickers::{parse_single_ticker, parse_tickers};
use crate::cli::common::trade_record_kind::TradeRecordKind;
use crate::cli::common::types::{OrderDirection, SummaryGroup, TriStateFilter};
use crate::cli::error::{CliExit, usage_error};
use crate::cli::field_metadata::{
    self, ALERT_HEADERS, BOMB_HEADERS, CLUSTER_HEADERS, LEVEL_HEADERS, TRADE_HEADERS,
};
use crate::cli::output::{finish_output, print_json, print_records_with_allowed_fields};

use self::dashboard::{TradeDashboard, dashboard_output_value};
use self::filters::{
    K_END_DATE, K_SECTOR_INDUSTRY, K_START_DATE, apply_trade_filter_args, apply_trade_list_ranges,
    apply_trade_ranges, cluster_bomb_filters, cluster_filters, dashboard_bombs_request,
    dashboard_clusters_request, dashboard_levels_request, dashboard_trades_request,
    default_trade_filters, default_trade_list_filters, level_touch_filters, parse_tri_state_filter,
    set_filter, set_ticker_filters, validate_trade_level_count,
};
use self::presets::{apply_preset_filters, find_trade_preset};
use self::sentiment::summarize_trade_sentiment;

#[cfg(test)]
use self::dashboard::parse_dashboard_fields;
#[cfg(test)]
use self::sentiment::{
    SentimentSide, TradeSentimentSignal, classify_trade_sentiment_side, sentiment_signal,
};

const DEFAULT_TRADE_LIMIT: usize = 1_000;
pub(super) const DEFAULT_DASHBOARD_COUNT: usize = 10;
const DEFAULT_DASHBOARD_LOOKBACK_DAYS: u32 = 365;
const DEFAULT_LEVEL_COUNT: usize = 10;
const DEFAULT_LEVEL_TOUCH_COUNT: usize = 50;
const DEFAULT_CLUSTER_LENGTH: i32 = 1_000;
const DEFAULT_CLUSTER_BOMB_LENGTH: i32 = 100;
pub(super) const DEFAULT_MAX_VOLUME: i64 = 2_000_000_000;
pub(super) const DEFAULT_MAX_PRICE: f64 = 100_000.0;
pub(super) const DEFAULT_MAX_DOLLARS: f64 = 30_000_000_000.0;
pub(super) const HAR_TRADE_MIN_VOLUME: i64 = 10_000;
pub(super) const HAR_TRADE_MAX_DOLLARS: f64 = 100_000_000_000.0;

#[derive(Debug, Serialize)]
struct DateRange {
    start: String,
    end: String,
}

/// Trade subcommands.
#[derive(Debug, Subcommand)]
pub enum TradeCommand {
    /// Query institutional trades.
    #[command(
        long_about = "Query institutional trades with optional ticker, date, range, and output filters.\n\nExamples:\n  volumeleaders-agent trade list NVDA\n  volumeleaders-agent trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields FullTimeString24,Volume,Price,Dollars,DollarsMultiplier"
    )]
    List(ListArgs),
    /// Query a ticker institutional dashboard.
    #[command(
        long_about = "Query a ticker institutional dashboard with trades, clusters, and levels.\n\nExamples:\n  volumeleaders-agent trade dashboard NVDA\n  volumeleaders-agent trade dashboard NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields trades.Ticker,trades.Price,clusters.Dollars"
    )]
    Dashboard(DashboardArgs),
    /// Summarize leveraged ETF bull and bear flow by day.
    #[command(
        long_about = "Summarize leveraged ETF bull and bear flow by day.\n\nExamples:\n  volumeleaders-agent trade sentiment\n  volumeleaders-agent trade sentiment --start-date 2026-05-01 --end-date 2026-05-27 --min-dollars 1000000"
    )]
    Sentiment(SentimentArgs),
    /// Query aggregated trade clusters.
    #[command(
        long_about = "Query aggregated trade clusters with optional ticker and rank filters.\n\nExamples:\n  volumeleaders-agent trade clusters NVDA\n  volumeleaders-agent trade clusters NVDA AAPL --start-date 2026-05-01 --end-date 2026-05-27 --relative-size 10 --fields Ticker,Date,Dollars"
    )]
    Clusters(ClustersArgs),
    /// Query trade cluster bombs.
    #[command(
        name = "cluster-bombs",
        long_about = "Query trade cluster bombs with optional ticker and dollar filters.\n\nExamples:\n  volumeleaders-agent trade cluster-bombs NVDA\n  volumeleaders-agent trade cluster-bombs NVDA --start-date 2026-05-01 --end-date 2026-05-27 --min-dollars 5000000 --start 50"
    )]
    ClusterBombs(ClusterBombsArgs),
    /// Query trade alerts for a date.
    #[command(
        long_about = "Query trade alerts for a specific trading date.\n\nExamples:\n  volumeleaders-agent trade alerts --date 2026-05-27\n  volumeleaders-agent trade alerts --date 2026-05-27 --start 50 --length 50 --fields Ticker,Price,Volume"
    )]
    Alerts(AlertsArgs),
    /// Query trade cluster alerts for a date.
    #[command(
        name = "cluster-alerts",
        long_about = "Query trade cluster alerts for a specific trading date.\n\nExamples:\n  volumeleaders-agent trade cluster-alerts --date 2026-05-27\n  volumeleaders-agent trade cluster-alerts --date 2026-05-27 --start 50 --length 50 --all-fields"
    )]
    ClusterAlerts(AlertsArgs),
    /// Query significant price levels for a ticker.
    #[command(
        long_about = "Query significant price levels for a ticker.\n\nExamples:\n  volumeleaders-agent trade levels NVDA\n  volumeleaders-agent trade levels NVDA --start-date 2026-05-01 --end-date 2026-05-27 --trade-level-count 10 --fields Ticker,Price,TradeLevelRank"
    )]
    Levels(LevelsArgs),
    /// Query trade events at notable price levels.
    #[command(
        name = "level-touches",
        long_about = "Query trade events at notable price levels for a ticker.\n\nExamples:\n  volumeleaders-agent trade level-touches NVDA\n  volumeleaders-agent trade level-touches NVDA --start-date 2026-05-01 --end-date 2026-05-27 --trade-level-rank 5 --relative-size 10"
    )]
    LevelTouches(LevelTouchesArgs),
}

/// Arguments for `trade list`.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Tickers as comma-separated or space-separated symbols.
    pub tickers: Vec<String>,

    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    #[command(flatten)]
    pub ranges: TradeRangeArgs,
    #[command(flatten)]
    pub filters: TradeFilterArgs,

    /// Apply a built-in filter preset by display name.
    #[arg(long)]
    pub preset: Option<String>,
    /// Return aggregate metrics instead of individual trades.
    #[arg(long)]
    pub summary: bool,
    /// Summary grouping. Valid only with --summary.
    #[arg(long = "group-by", value_enum)]
    pub group_by: Option<SummaryGroup>,
    /// Maximum number of trades to return.
    #[arg(long, default_value_t = DEFAULT_TRADE_LIMIT)]
    pub limit: usize,
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields trade list`.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every raw API field.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `trade dashboard`.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct DashboardArgs {
    /// Ticker symbol.
    pub ticker: String,
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    #[command(flatten)]
    pub ranges: TradeRangeArgs,
    #[command(flatten)]
    pub filters: DashboardFilterArgs,
    /// Rows to return per dashboard section.
    #[arg(long, default_value_t = DEFAULT_DASHBOARD_COUNT)]
    pub count: usize,
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields trade dashboard`.
    ///
    /// Use section-qualified fields like `trades.Date,clusters.Dollars`.
    /// Unqualified fields are applied to every row section.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every raw API field.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `trade sentiment`.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct SentimentArgs {
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    #[command(flatten)]
    pub ranges: TradeRangeArgs,
    #[command(flatten)]
    pub filters: TradeFilterArgs,
}

/// Arguments for `trade clusters`.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct ClustersArgs {
    /// Tickers as comma-separated or space-separated symbols.
    pub tickers: Vec<String>,
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    #[command(flatten)]
    pub ranges: TradeRangeArgs,
    /// Minimum volume-concentration delta score to include.
    #[arg(long)]
    pub vcd: Option<i32>,
    /// Security type code, such as 1 for stocks, 26 for ETFs, or 4 for REITs.
    #[arg(long = "security-type")]
    pub security_type: Option<i32>,
    /// Minimum relative-size bucket to include, such as 5, 10, 25, 50, or 100.
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    /// Sector or industry text filter accepted by the VolumeLeaders API.
    #[arg(long)]
    pub sector: Option<String>,
    /// Maximum trade-cluster rank to include; lower ranks are more significant.
    #[arg(long = "trade-cluster-rank", default_value_t = 100)]
    pub trade_cluster_rank: i32,
    #[command(flatten)]
    pub page: FixedPageArgs,
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields trade clusters`.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every raw API field.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `trade cluster-bombs`.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct ClusterBombsArgs {
    /// Tickers as comma-separated or space-separated symbols.
    pub tickers: Vec<String>,
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    #[command(flatten)]
    pub ranges: VolumeDollarRangeArgs,
    /// Minimum volume-concentration delta score to include.
    #[arg(long)]
    pub vcd: Option<i32>,
    /// Security type code, such as 1 for stocks, 26 for ETFs, or 4 for REITs.
    #[arg(long = "security-type")]
    pub security_type: Option<i32>,
    /// Minimum relative-size bucket to include, such as 5, 10, 25, 50, or 100.
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    /// Sector or industry text filter accepted by the VolumeLeaders API.
    #[arg(long)]
    pub sector: Option<String>,
    /// Maximum trade-cluster-bomb rank to include; lower ranks are more significant.
    #[arg(long = "trade-cluster-bomb-rank", default_value_t = -1)]
    pub trade_cluster_bomb_rank: i32,
    #[command(flatten)]
    pub page: FixedPageArgs,
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields trade cluster-bombs`.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every raw API field.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for trade alert commands.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct AlertsArgs {
    /// Alert date, YYYY-MM-DD.
    #[arg(long, required = true)]
    pub date: String,
    #[command(flatten)]
    pub page: PageArgs,
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields trade alerts` or `fields trade clusters`.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every raw API field.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `trade levels`.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct LevelsArgs {
    /// Ticker symbol.
    pub ticker: String,
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    /// Number of price levels to return. Accepted values: 5, 10, 20, or 50.
    #[arg(long = "trade-level-count", default_value_t = DEFAULT_LEVEL_COUNT)]
    pub trade_level_count: usize,
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields trade levels`.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every raw API field.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `trade level-touches`.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct LevelTouchesArgs {
    /// Ticker symbol.
    pub ticker: String,
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    #[command(flatten)]
    pub ranges: TradeRangeArgs,
    /// Maximum trade-level rank to include; lower ranks are more significant.
    #[arg(long = "trade-level-rank", default_value_t = 5)]
    pub trade_level_rank: i32,
    /// Number of levels to include. Accepted values: 5, 10, 20, or 50.
    #[arg(long = "trade-level-count", default_value_t = DEFAULT_LEVEL_TOUCH_COUNT)]
    pub trade_level_count: usize,
    /// Minimum volume-concentration delta score to include.
    #[arg(long)]
    pub vcd: Option<i32>,
    /// Minimum relative-size bucket to include, such as 5, 10, 25, 50, or 100.
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    #[command(flatten)]
    pub page: PageArgs,
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields trade levels`.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every raw API field.
    #[arg(long)]
    pub all_fields: bool,
}

/// Optional date range flags.
#[derive(Debug, Args)]
pub struct OptionalDateRangeArgs {
    /// Start date, YYYY-MM-DD.
    #[arg(long)]
    pub start_date: Option<String>,
    /// End date, YYYY-MM-DD.
    #[arg(long)]
    pub end_date: Option<String>,
    /// Look back this many days from --end-date or today.
    #[arg(long)]
    pub days: Option<u32>,
}

/// Trade volume, price, and dollars range flags.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct TradeRangeArgs {
    /// Minimum share volume to include.
    #[arg(long = "min-volume")]
    pub min_volume: Option<i64>,
    /// Maximum share volume to include.
    #[arg(long = "max-volume")]
    pub max_volume: Option<i64>,
    /// Minimum trade price to include.
    #[arg(long = "min-price")]
    pub min_price: Option<f64>,
    /// Maximum trade price to include.
    #[arg(long = "max-price")]
    pub max_price: Option<f64>,
    /// Minimum trade dollar value to include.
    #[arg(long = "min-dollars")]
    pub min_dollars: Option<f64>,
    /// Maximum trade dollar value to include.
    #[arg(long = "max-dollars")]
    pub max_dollars: Option<f64>,
}

/// Trade volume and dollars range flags.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct VolumeDollarRangeArgs {
    /// Minimum share volume to include.
    #[arg(long = "min-volume")]
    pub min_volume: Option<i64>,
    /// Maximum share volume to include.
    #[arg(long = "max-volume")]
    pub max_volume: Option<i64>,
    /// Minimum trade or cluster dollar value to include.
    #[arg(long = "min-dollars")]
    pub min_dollars: Option<f64>,
    /// Maximum trade or cluster dollar value to include.
    #[arg(long = "max-dollars")]
    pub max_dollars: Option<f64>,
}

/// Shared trade filter flags.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct TradeFilterArgs {
    /// Trade condition code filter accepted by the VolumeLeaders API.
    #[arg(long)]
    pub conditions: Option<String>,
    /// Minimum volume-concentration delta score to include.
    #[arg(long)]
    pub vcd: Option<i32>,
    /// Security type code, such as 1 for stocks, 26 for ETFs, or 4 for REITs.
    #[arg(long = "security-type")]
    pub security_type: Option<i32>,
    /// Minimum relative-size bucket to include, such as 5, 10, 25, 50, or 100.
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    /// Filter dark-pool trades: true includes only dark-pool prints, false excludes them, omitted includes both.
    #[arg(long = "dark-pools", value_parser = parse_tri_state_filter)]
    pub dark_pools: Option<TriStateFilter>,
    /// Filter sweep trades: true includes only sweeps, false excludes them, omitted includes both.
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub sweeps: Option<TriStateFilter>,
    /// Filter late prints: true includes only late prints, false excludes them, omitted includes both.
    #[arg(long = "late-prints", value_parser = parse_tri_state_filter)]
    pub late_prints: Option<TriStateFilter>,
    /// Filter signature prints: true includes only signature prints, false excludes them, omitted includes both.
    #[arg(long = "sig-prints", value_parser = parse_tri_state_filter)]
    pub sig_prints: Option<TriStateFilter>,
    /// Filter even-share prints: true includes only even-share prints, false excludes them, omitted includes both.
    #[arg(long = "even-shared", value_parser = parse_tri_state_filter)]
    pub even_shared: Option<TriStateFilter>,
    /// Maximum trade rank to include, where lower ranks are more significant.
    #[arg(long = "trade-rank")]
    pub trade_rank: Option<i32>,
    /// Snapshot rank bucket to include, where lower ranks are more significant.
    #[arg(long = "rank-snapshot")]
    pub rank_snapshot: Option<i32>,
    /// Market-cap bucket code accepted by the VolumeLeaders API.
    #[arg(long = "market-cap")]
    pub market_cap: Option<i32>,
    /// Filter premarket trades: true includes only premarket prints, false excludes them, omitted includes both.
    #[arg(long = "premarket", value_parser = parse_tri_state_filter)]
    pub premarket: Option<TriStateFilter>,
    /// Filter regular-hours trades: true includes only regular-session prints, false excludes them, omitted includes both.
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub rth: Option<TriStateFilter>,
    /// Filter after-hours trades: true includes only after-hours prints, false excludes them, omitted includes both.
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub ah: Option<TriStateFilter>,
    /// Filter opening trades: true includes only opening prints, false excludes them, omitted includes both.
    #[arg(long = "opening", value_parser = parse_tri_state_filter)]
    pub opening: Option<TriStateFilter>,
    /// Filter closing trades: true includes only closing prints, false excludes them, omitted includes both.
    #[arg(long = "closing", value_parser = parse_tri_state_filter)]
    pub closing: Option<TriStateFilter>,
    /// Filter phantom prints: true includes only phantom prints, false excludes them, omitted includes both.
    #[arg(long = "phantom", value_parser = parse_tri_state_filter)]
    pub phantom: Option<TriStateFilter>,
    /// Filter offsetting prints: true includes only offsetting prints, false excludes them, omitted includes both.
    #[arg(long = "offsetting", value_parser = parse_tri_state_filter)]
    pub offsetting: Option<TriStateFilter>,
    /// Sector or industry text filter accepted by the VolumeLeaders API.
    #[arg(long)]
    pub sector: Option<String>,
}

/// Dashboard trade filters. The chart endpoint does not accept the heavier
/// list-only filters such as security type, even-share, rank snapshot, or
/// market-cap filters.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct DashboardFilterArgs {
    /// Trade condition code filter accepted by the chart endpoint.
    #[arg(long)]
    pub conditions: Option<String>,
    /// Minimum volume-concentration delta score to include.
    #[arg(long)]
    pub vcd: Option<i32>,
    /// Minimum relative-size bucket to include, such as 5, 10, 25, 50, or 100.
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    /// Filter dark-pool trades: true includes only dark-pool prints, false excludes them, omitted includes both.
    #[arg(long = "dark-pools", value_parser = parse_tri_state_filter)]
    pub dark_pools: Option<TriStateFilter>,
    /// Filter sweep trades: true includes only sweeps, false excludes them, omitted includes both.
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub sweeps: Option<TriStateFilter>,
    /// Filter late prints: true includes only late prints, false excludes them, omitted includes both.
    #[arg(long = "late-prints", value_parser = parse_tri_state_filter)]
    pub late_prints: Option<TriStateFilter>,
    /// Filter signature prints: true includes only signature prints, false excludes them, omitted includes both.
    #[arg(long = "sig-prints", value_parser = parse_tri_state_filter)]
    pub sig_prints: Option<TriStateFilter>,
    /// Maximum trade rank to include, where lower ranks are more significant.
    #[arg(long = "trade-rank")]
    pub trade_rank: Option<i32>,
    /// Filter premarket trades: true includes only premarket prints, false excludes them, omitted includes both.
    #[arg(long = "premarket", value_parser = parse_tri_state_filter)]
    pub premarket: Option<TriStateFilter>,
    /// Filter regular-hours trades: true includes only regular-session prints, false excludes them, omitted includes both.
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub rth: Option<TriStateFilter>,
    /// Filter after-hours trades: true includes only after-hours prints, false excludes them, omitted includes both.
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub ah: Option<TriStateFilter>,
    /// Filter opening trades: true includes only opening prints, false excludes them, omitted includes both.
    #[arg(long = "opening", value_parser = parse_tri_state_filter)]
    pub opening: Option<TriStateFilter>,
    /// Filter closing trades: true includes only closing prints, false excludes them, omitted includes both.
    #[arg(long = "closing", value_parser = parse_tri_state_filter)]
    pub closing: Option<TriStateFilter>,
    /// Filter phantom prints: true includes only phantom prints, false excludes them, omitted includes both.
    #[arg(long = "phantom", value_parser = parse_tri_state_filter)]
    pub phantom: Option<TriStateFilter>,
    /// Filter offsetting prints: true includes only offsetting prints, false excludes them, omitted includes both.
    #[arg(long = "offsetting", value_parser = parse_tri_state_filter)]
    pub offsetting: Option<TriStateFilter>,
    /// Sector or industry text filter accepted by the chart endpoint.
    #[arg(long)]
    pub sector: Option<String>,
}

/// DataTables page flags with length.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct PageArgs {
    /// Zero-based row offset for paged API requests.
    #[arg(long, default_value_t = 0)]
    pub start: i32,
    /// Number of rows to request from the API.
    #[arg(long, default_value_t = 100)]
    pub length: i32,
    /// Zero-based API sort column index.
    #[arg(long = "order-col", default_value_t = 1)]
    pub order_col: i32,
    /// API sort direction for the selected order column.
    #[arg(long = "order-dir", value_enum, default_value = "desc")]
    pub order_dir: OrderDirection,
}

/// DataTables page flags without length.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct FixedPageArgs {
    /// Zero-based row offset for paged API requests.
    #[arg(long, default_value_t = 0)]
    pub start: i32,
    /// Zero-based API sort column index.
    #[arg(long = "order-col", default_value_t = 1)]
    pub order_col: i32,
    /// API sort direction for the selected order column.
    #[arg(long = "order-dir", value_enum, default_value = "desc")]
    pub order_dir: OrderDirection,
}

/// Handles the trade command group.
#[instrument(skip_all)]
pub async fn handle(args: &TradeArgs) -> Result<(), CliExit> {
    match &args.command {
        TradeCommand::List(list_args) => execute_list(list_args).await,
        TradeCommand::Dashboard(dashboard_args) => execute_dashboard(dashboard_args).await,
        TradeCommand::Sentiment(sentiment_args) => execute_sentiment(sentiment_args).await,
        TradeCommand::Clusters(cluster_args) => execute_clusters(cluster_args).await,
        TradeCommand::ClusterBombs(bomb_args) => execute_cluster_bombs(bomb_args).await,
        TradeCommand::Alerts(alert_args) => execute_alerts(alert_args).await,
        TradeCommand::ClusterAlerts(alert_args) => execute_cluster_alerts(alert_args).await,
        TradeCommand::Levels(level_args) => execute_levels(level_args).await,
        TradeCommand::LevelTouches(touch_args) => execute_level_touches(touch_args).await,
    }
}

pub(super) fn parse_ticker_args(args: &[String]) -> Vec<String> {
    parse_tickers(&args.join(","))
}

fn resolve_with_default(args: &OptionalDateRangeArgs, default_days: u32) -> (String, String) {
    let days = args.days.or_else(|| {
        if args.start_date.is_none() && args.end_date.is_none() {
            Some(default_days)
        } else {
            None
        }
    });
    resolve_date_range(args.start_date.as_deref(), args.end_date.as_deref(), days)
}

fn resolve_required_range(args: &OptionalDateRangeArgs) -> Result<(String, String), String> {
    if args.days.is_some() {
        return Ok(resolve_date_range(
            args.start_date.as_deref(),
            args.end_date.as_deref(),
            args.days,
        ));
    }
    if args.start_date.is_none() || args.end_date.is_none() {
        return Err("--start-date and --end-date are required unless --days is set".to_string());
    }
    Ok(resolve_date_range(
        args.start_date.as_deref(),
        args.end_date.as_deref(),
        None,
    ))
}

fn resolve_trade_list_range(args: &OptionalDateRangeArgs) -> (String, String) {
    if args.start_date.is_none() && args.end_date.is_none() && args.days.is_none() {
        return ("Today".to_string(), "Today".to_string());
    }
    resolve_date_range(
        args.start_date.as_deref(),
        args.end_date.as_deref(),
        args.days,
    )
}

#[instrument(skip_all)]
async fn execute_list(args: &ListArgs) -> Result<(), CliExit> {
    if args.group_by.is_some() && !args.summary {
        return Err(usage_error("--group-by only works with --summary"));
    }
    if args.summary && (args.fields.is_some() || args.all_fields) {
        return Err(usage_error(
            "--fields and --all-fields cannot be used with --summary",
        ));
    }

    let tickers = parse_ticker_args(&args.tickers);
    let (start, end) = resolve_trade_list_range(&args.dates);
    let mut filters = default_trade_list_filters();
    apply_trade_list_ranges(&mut filters, &args.ranges);
    apply_trade_filter_args(&mut filters, &args.filters);

    if let Some(preset_name) = &args.preset {
        let preset = match find_trade_preset(preset_name) {
            Some(preset) => preset,
            None => {
                return Err(usage_error(format!("unknown trade preset: {preset_name}")));
            }
        };
        filters = default_trade_list_filters();
        apply_preset_filters(&mut filters, preset);
        apply_trade_list_ranges(&mut filters, &args.ranges);
        apply_trade_filter_args(&mut filters, &args.filters);
    }
    set_filter(&mut filters, K_START_DATE, start.clone());
    set_filter(&mut filters, K_END_DATE, end.clone());
    set_ticker_filters(&mut filters, &tickers, "Tickers");

    let length = i32::try_from(args.limit).unwrap_or(i32::MAX);
    let request = TradesRequest::new()
        .with_length(length)
        .with_search("", false)
        .with_order(1, "DESC", "FullTimeString24")
        .with_trade_filters(filters);
    let client = make_client().await?;
    let mut trades = client.get_trades(&request).await?.data;
    trades.truncate(args.limit);

    let output = if args.summary {
        let group = args.group_by.unwrap_or(SummaryGroup::Ticker);
        let summary = build_summary(&trades, group, &start, &end);
        print_json(&summary)
    } else {
        print_trade_records(
            &trades,
            TradeRecordKind::Trade,
            TRADE_HEADERS,
            args.fields.as_deref(),
            args.all_fields,
            "trade list",
        )
    };
    finish_output(output)
}

#[instrument(skip_all)]
async fn execute_dashboard(args: &DashboardArgs) -> Result<(), CliExit> {
    let ticker = parse_single_ticker(&args.ticker);
    let (start, end) = resolve_with_default(&args.dates, DEFAULT_DASHBOARD_LOOKBACK_DAYS);
    let client = make_client().await?;

    let trades_req = dashboard_trades_request(args, &ticker, &start, &end);
    let clusters_req = dashboard_clusters_request(args, &ticker, &start, &end);
    let levels_req = dashboard_levels_request(&ticker, &start, &end, args.count);
    let bombs_req = dashboard_bombs_request(args, &ticker, &start, &end);

    let (trades_result, clusters_result, levels_result, bombs_result) = tokio::join!(
        client.get_trades(&trades_req),
        client.get_trade_clusters(&clusters_req),
        client.get_chart0_trade_levels(&levels_req),
        client.get_trade_cluster_bombs(&bombs_req),
    );

    let trades = trades_result?.data;
    let clusters = clusters_result?.data;
    let mut levels = levels_result?.data;
    levels.truncate(args.count);
    let cluster_bombs = bombs_result?.data;

    let dashboard = TradeDashboard {
        ticker,
        date_range: DateRange { start, end },
        count: args.count,
        trades,
        clusters,
        levels,
        cluster_bombs,
    };
    let dashboard = dashboard_output_value(&dashboard, args)
        .map_err(|message| usage_error(format!("field error: {message}")))?;
    finish_output(print_json(&dashboard))
}

#[instrument(skip_all)]
async fn execute_sentiment(args: &SentimentArgs) -> Result<(), CliExit> {
    let (start, end) = resolve_required_range(&args.dates).map_err(usage_error)?;
    let mut filters = default_trade_filters(args.ranges.min_dollars.unwrap_or(5_000_000.0), 97);
    apply_trade_ranges(&mut filters, &args.ranges, 5_000_000.0);
    apply_trade_filter_args(&mut filters, &args.filters);
    set_filter(&mut filters, K_START_DATE, start.clone());
    set_filter(&mut filters, K_END_DATE, end.clone());
    set_filter(&mut filters, K_SECTOR_INDUSTRY, "X B".to_string());

    let request = TradesRequest::new()
        .with_length(50)
        .with_order(1, "desc", "FullTimeString24")
        .with_trade_filters(filters);
    let client = make_client().await?;
    let trades = client.get_trades_limit(&request, usize::MAX).await?;
    let sentiment = summarize_trade_sentiment(&trades, &start, &end);
    finish_output(print_json(&sentiment))
}

#[instrument(skip_all)]
async fn execute_clusters(args: &ClustersArgs) -> Result<(), CliExit> {
    let (start, end) = resolve_required_range(&args.dates).map_err(usage_error)?;
    let request = TradeClustersRequest::new()
        .with_start(args.page.start)
        .with_length(DEFAULT_CLUSTER_LENGTH)
        .with_search("", false)
        .with_order(
            args.page.order_col,
            args.page.order_dir.as_str().to_ascii_uppercase(),
            cluster_order_name(args.page.order_col),
        )
        .with_cluster_filters(cluster_filters(args, &start, &end));
    let client = make_client().await?;
    let response = client.get_trade_clusters(&request).await?;
    output_trade_records(
        &response.data,
        TradeRecordKind::Cluster,
        CLUSTER_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade clusters",
    )
}

#[instrument(skip_all)]
async fn execute_cluster_bombs(args: &ClusterBombsArgs) -> Result<(), CliExit> {
    let (start, end) = resolve_required_range(&args.dates).map_err(usage_error)?;
    let request = TradeClusterBombsRequest::new()
        .with_start(args.page.start)
        .with_length(DEFAULT_CLUSTER_BOMB_LENGTH)
        .with_search("", false)
        .with_order(
            args.page.order_col,
            args.page.order_dir.as_str().to_ascii_uppercase(),
            cluster_bomb_order_name(args.page.order_col),
        )
        .with_cluster_bomb_filters(cluster_bomb_filters(args, &start, &end));
    let client = make_client().await?;
    let response = client.get_trade_cluster_bombs(&request).await?;
    output_trade_records(
        &response.data,
        TradeRecordKind::ClusterBomb,
        BOMB_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade clusters",
    )
}

#[instrument(skip_all)]
async fn execute_alerts(args: &AlertsArgs) -> Result<(), CliExit> {
    let request = crate::TradeAlertsRequest::new()
        .with_start(args.page.start)
        .with_length(args.page.length)
        .with_order(args.page.order_col, args.page.order_dir.as_str(), "")
        .with_date(args.date.clone());
    let client = make_client().await?;
    let response = client.get_trade_alerts(&request).await?;
    output_trade_records(
        &response.data,
        TradeRecordKind::Trade,
        ALERT_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade alerts",
    )
}

#[instrument(skip_all)]
async fn execute_cluster_alerts(args: &AlertsArgs) -> Result<(), CliExit> {
    let request = crate::TradeClusterAlertsRequest::new()
        .with_start(args.page.start)
        .with_length(args.page.length)
        .with_order(args.page.order_col, args.page.order_dir.as_str(), "")
        .with_date(args.date.clone());
    let client = make_client().await?;
    let response = client.get_trade_cluster_alerts(&request).await?;
    output_trade_records(
        &response.data,
        TradeRecordKind::Cluster,
        CLUSTER_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade clusters",
    )
}

#[instrument(skip_all)]
async fn execute_levels(args: &LevelsArgs) -> Result<(), CliExit> {
    if !validate_trade_level_count(args.trade_level_count) {
        return Err(usage_error(
            "--trade-level-count must be one of 5, 10, 20, or 50 for trade level retrieval",
        ));
    }
    let ticker = parse_single_ticker(&args.ticker);
    let (start, end) = resolve_with_default(&args.dates, 365);
    let request =
        dashboard_levels_request(&ticker, &start, &end, args.trade_level_count).with_length(-1);
    let client = make_client().await?;
    let response = client.get_chart0_trade_levels(&request).await?;
    let mut levels = response.data;
    levels.truncate(args.trade_level_count);
    output_trade_records(
        &levels,
        TradeRecordKind::Level,
        LEVEL_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade levels",
    )
}

#[instrument(skip_all)]
async fn execute_level_touches(args: &LevelTouchesArgs) -> Result<(), CliExit> {
    if !validate_trade_level_count(args.trade_level_count) {
        return Err(usage_error(
            "--trade-level-count must be one of 5, 10, 20, or 50 for trade level retrieval",
        ));
    }
    if !(1..=50).contains(&args.page.length) {
        return Err(usage_error(
            "--length must be between 1 and 50 for trade level touch retrieval",
        ));
    }
    let (start, end) = resolve_required_range(&args.dates).map_err(usage_error)?;
    let ticker = parse_single_ticker(&args.ticker);
    let request = TradeLevelTouchesRequest::new()
        .with_start(args.page.start)
        .with_length(args.page.length)
        .with_order(args.page.order_col, args.page.order_dir.as_str(), "")
        .with_level_touch_filters(level_touch_filters(args, &ticker, &start, &end));
    let client = make_client().await?;
    let response = client.get_trade_level_touches(&request).await?;
    output_trade_records(
        &response.data,
        TradeRecordKind::Level,
        LEVEL_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade levels",
    )
}

fn output_trade_records<T: Serialize>(
    records: &[T],
    kind: TradeRecordKind,
    headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    command_path: &str,
) -> Result<(), CliExit> {
    Ok(print_trade_records(
        records,
        kind,
        headers,
        fields,
        all_fields,
        command_path,
    )?)
}

fn cluster_order_name(order_col: i32) -> &'static str {
    if order_col == 1 {
        "MinFullTimeString24"
    } else {
        ""
    }
}

fn cluster_bomb_order_name(order_col: i32) -> &'static str {
    if order_col == 1 {
        "MinFullTimeString24"
    } else {
        ""
    }
}

fn print_trade_records<T: Serialize>(
    records: &[T],
    kind: TradeRecordKind,
    headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    command_path: &str,
) -> std::io::Result<()> {
    let allowed_fields = trade_output_field_names(kind, command_path);
    print_records_with_allowed_fields(
        records,
        headers,
        fields,
        all_fields,
        allowed_fields.as_deref(),
    )
}

fn trade_output_field_names(kind: TradeRecordKind, command_path: &str) -> Option<Vec<String>> {
    let command_path = if matches!(kind, TradeRecordKind::ClusterBomb) {
        "trade cluster-bombs"
    } else {
        command_path
    };
    field_metadata::field_names(command_path)
}

/// Clamp an arbitrary count to the nearest API-supported level count.
///
/// The VolumeLeaders levels endpoint only accepts {5, 10, 20, 50}.
/// Dashboard uses this to send a valid `Levels` filter while still
/// truncating results client-side to the user's requested count.
fn nearest_level_count(count: usize) -> usize {
    const VALID: [usize; 4] = [5, 10, 20, 50];
    *VALID.iter().find(|&&v| v >= count).unwrap_or(&50)
}

#[derive(Debug, Serialize)]
struct TradeSummary {
    date_range: DateRange,
    total_trades: usize,
    total_dollars: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    by_ticker: Option<HashMap<String, TradeGroupSummary>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    by_day: Option<HashMap<String, TradeGroupSummary>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    by_ticker_day: Option<HashMap<String, TradeGroupSummary>>,
}

#[derive(Clone, Copy, Debug, Default)]
struct TradeGroupAccumulator {
    trades: usize,
    dollars: f64,
    dollars_multiplier: f64,
    dark_pool: usize,
    sweep: usize,
    cumulative_distribution: f64,
}

#[derive(Debug, Serialize)]
struct TradeGroupSummary {
    trades: usize,
    dollars: f64,
    avg_dollars_multiplier: f64,
    pct_dark_pool: f64,
    pct_sweep: f64,
    avg_cumulative_distribution: f64,
}

fn build_summary(
    trades: &[crate::Trade],
    group: SummaryGroup,
    start: &str,
    end: &str,
) -> TradeSummary {
    let mut groups = HashMap::<String, TradeGroupAccumulator>::new();
    let mut total_dollars = 0.0;
    for trade in trades {
        total_dollars += trade.dollars.and_then(|d| d.to_f64()).unwrap_or(0.0);
        let key = summary_key(trade, group);
        add_summary_group(groups.entry(key).or_default(), trade);
    }
    let summarized = summarize_groups(groups);
    let mut summary = TradeSummary {
        date_range: DateRange {
            start: start.to_string(),
            end: end.to_string(),
        },
        total_trades: trades.len(),
        total_dollars,
        by_ticker: None,
        by_day: None,
        by_ticker_day: None,
    };
    match group {
        SummaryGroup::Ticker => summary.by_ticker = Some(summarized),
        SummaryGroup::Day => summary.by_day = Some(summarized),
        SummaryGroup::TickerDay => summary.by_ticker_day = Some(summarized),
    }
    summary
}

fn summary_key(trade: &crate::Trade, group: SummaryGroup) -> String {
    match group {
        SummaryGroup::Ticker => trade.ticker.as_deref().unwrap_or("unknown").to_string(),
        SummaryGroup::Day => trade_day(trade),
        SummaryGroup::TickerDay => format!(
            "{}|{}",
            trade.ticker.as_deref().unwrap_or("unknown"),
            trade_day(trade)
        ),
    }
}

pub(super) fn trade_day(trade: &crate::Trade) -> String {
    trade
        .date
        .as_ref()
        .and_then(|date| date.0.map(|dt| dt.format(DATE_FMT).to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn add_summary_group(acc: &mut TradeGroupAccumulator, trade: &crate::Trade) {
    acc.trades += 1;
    acc.dollars += trade.dollars.and_then(|d| d.to_f64()).unwrap_or(0.0);
    acc.dollars_multiplier += trade.dollars_multiplier.unwrap_or(0.0);
    acc.cumulative_distribution += trade.cumulative_distribution.unwrap_or(0.0);
    if trade
        .dark_pool
        .as_ref()
        .is_some_and(|value| value.0 == Some(true))
    {
        acc.dark_pool += 1;
    }
    if trade
        .sweep
        .as_ref()
        .is_some_and(|value| value.0 == Some(true))
    {
        acc.sweep += 1;
    }
}

fn summarize_groups(
    groups: HashMap<String, TradeGroupAccumulator>,
) -> HashMap<String, TradeGroupSummary> {
    groups
        .into_iter()
        .map(|(key, acc)| (key, summarize_group(acc)))
        .collect()
}

fn summarize_group(acc: TradeGroupAccumulator) -> TradeGroupSummary {
    if acc.trades == 0 {
        return TradeGroupSummary {
            trades: 0,
            dollars: 0.0,
            avg_dollars_multiplier: 0.0,
            pct_dark_pool: 0.0,
            pct_sweep: 0.0,
            avg_cumulative_distribution: 0.0,
        };
    }
    let count = acc.trades as f64;
    TradeGroupSummary {
        trades: acc.trades,
        dollars: acc.dollars,
        avg_dollars_multiplier: acc.dollars_multiplier / count,
        pct_dark_pool: acc.dark_pool as f64 / count * 100.0,
        pct_sweep: acc.sweep as f64 / count * 100.0,
        avg_cumulative_distribution: acc.cumulative_distribution / count,
    }
}

#[cfg(test)]
#[path = "trade_tests.rs"]
mod tests;
