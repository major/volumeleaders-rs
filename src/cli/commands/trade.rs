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
use crate::cli::common::auth::{handle_api_error, make_client};
use crate::cli::common::dates::resolve_date_range;
use crate::cli::common::tickers::{parse_single_ticker, parse_tickers};
use crate::cli::common::trade_transforms::TradeRecordKind;
use crate::cli::common::types::{OrderDirection, SummaryGroup, TriStateFilter};
use crate::cli::common::{DATE_FMT, TRADE_HEADERS};
use crate::cli::error::usage_error;
use crate::cli::field_metadata;
use crate::cli::output::{
    finish_output, print_json, print_transformed_record_values_with_allowed_fields,
};

use self::dashboard::{TradeDashboard, dashboard_output_value};
use self::filters::{
    apply_trade_filter_args, apply_trade_list_ranges, apply_trade_ranges, cluster_bomb_filters,
    cluster_filters, dashboard_bombs_request, dashboard_clusters_request, dashboard_levels_request,
    dashboard_trades_request, default_trade_filters, default_trade_list_filters,
    level_touch_filters, parse_tri_state_filter, set_filter, set_ticker_filters,
    validate_trade_level_count,
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

const CLUSTER_HEADERS: [&str; 10] = [
    "Date",
    "Ticker",
    "Price",
    "Dollars",
    "TradeCount",
    "DollarsMultiplier",
    "CumulativeDistribution",
    "TradeClusterRank",
    "window",
    "events",
];
const BOMB_HEADERS: [&str; 9] = [
    "Date",
    "Ticker",
    "Dollars",
    "TradeCount",
    "DollarsMultiplier",
    "CumulativeDistribution",
    "TradeClusterBombRank",
    "window",
    "events",
];
const LEVEL_HEADERS: [&str; 7] = [
    "Ticker",
    "Price",
    "Dollars",
    "Trades",
    "RelativeSize",
    "CumulativeDistribution",
    "TradeLevelRank",
];
const ALERT_HEADERS: [&str; 12] = [
    "Ticker",
    "Date",
    "Time",
    "AlertType",
    "TradeID",
    "Price",
    "Volume",
    "Dollars",
    "TradeRank",
    "type",
    "venue",
    "events",
];

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
        long_about = "Query institutional trades with optional ticker, date, range, and output filters.\n\nExamples:\n  volumeleaders-agent trade list NVDA\n  volumeleaders-agent trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields Ticker,DateTime,Price,Dollars,venue"
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
    /// Return every field after semantic trade transforms.
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
    /// Return every field (semantic transforms still apply).
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
    /// Return every field after semantic trade transforms.
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
    /// Return every field after semantic trade transforms.
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
    /// Return every field after semantic trade transforms.
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
    /// Number of price levels to return.
    #[arg(long = "trade-level-count", default_value_t = DEFAULT_LEVEL_COUNT)]
    pub trade_level_count: usize,
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields trade levels`.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field after semantic trade transforms.
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
    /// Number of levels to include.
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
    /// Return every field after semantic trade transforms.
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
pub async fn handle(args: &TradeArgs) -> i32 {
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
async fn execute_list(args: &ListArgs) -> i32 {
    if args.group_by.is_some() && !args.summary {
        return usage_error("--group-by only works with --summary");
    }
    if args.summary && (args.fields.is_some() || args.all_fields) {
        return usage_error("--fields and --all-fields cannot be used with --summary");
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
                return usage_error(format!("unknown trade preset: {preset_name}"));
            }
        };
        filters = default_trade_list_filters();
        apply_preset_filters(&mut filters, preset);
        apply_trade_list_ranges(&mut filters, &args.ranges);
        apply_trade_filter_args(&mut filters, &args.filters);
    }
    set_filter(&mut filters, "StartDate", start.clone());
    set_filter(&mut filters, "EndDate", end.clone());
    set_ticker_filters(&mut filters, &tickers, "Tickers");

    let length = i32::try_from(args.limit).unwrap_or(i32::MAX);
    let request = TradesRequest::new()
        .with_length(length)
        .with_search("", false)
        .with_order(1, "DESC", "FullTimeString24")
        .with_trade_filters(filters);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let mut trades = match client.get_trades(&request).await {
        Ok(response) => response.data,
        Err(err) => return handle_api_error(err),
    };
    trades.truncate(args.limit);

    let output = if args.summary {
        let group = args.group_by.unwrap_or(SummaryGroup::Ticker);
        let summary = build_summary(&trades, group, &start, &end);
        print_json(&summary)
    } else {
        print_trade_records(
            &trades,
            TradeRecordKind::Trade,
            &TRADE_HEADERS,
            args.fields.as_deref(),
            args.all_fields,
            "trade list",
        )
    };
    finish_output(output)
}

#[instrument(skip_all)]
async fn execute_dashboard(args: &DashboardArgs) -> i32 {
    let ticker = parse_single_ticker(&args.ticker);
    let (start, end) = resolve_with_default(&args.dates, DEFAULT_DASHBOARD_LOOKBACK_DAYS);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };

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

    let trades = match trades_result {
        Ok(response) => response.data,
        Err(err) => return handle_api_error(err),
    };
    let clusters = match clusters_result {
        Ok(response) => response.data,
        Err(err) => return handle_api_error(err),
    };
    let mut levels = match levels_result {
        Ok(response) => response.data,
        Err(err) => return handle_api_error(err),
    };
    levels.truncate(args.count);
    let cluster_bombs = match bombs_result {
        Ok(response) => response.data,
        Err(err) => return handle_api_error(err),
    };

    let dashboard = TradeDashboard {
        ticker,
        date_range: DateRange { start, end },
        count: args.count,
        trades,
        clusters,
        levels,
        cluster_bombs,
    };
    let dashboard = match dashboard_output_value(&dashboard, args) {
        Ok(value) => value,
        Err(message) => return usage_error(format!("field error: {message}")),
    };
    finish_output(print_json(&dashboard))
}

#[instrument(skip_all)]
async fn execute_sentiment(args: &SentimentArgs) -> i32 {
    let (start, end) = match resolve_required_range(&args.dates) {
        Ok(range) => range,
        Err(message) => return usage_error(message),
    };
    let mut filters = default_trade_filters(args.ranges.min_dollars.unwrap_or(5_000_000.0), 97);
    apply_trade_ranges(&mut filters, &args.ranges, 5_000_000.0);
    apply_trade_filter_args(&mut filters, &args.filters);
    set_filter(&mut filters, "StartDate", start.clone());
    set_filter(&mut filters, "EndDate", end.clone());
    set_filter(&mut filters, "SectorIndustry", "X B".to_string());

    let request = TradesRequest::new()
        .with_length(50)
        .with_order(1, "desc", "FullTimeString24")
        .with_trade_filters(filters);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let trades = match client.get_trades_limit(&request, usize::MAX).await {
        Ok(trades) => trades,
        Err(err) => return handle_api_error(err),
    };
    let sentiment = summarize_trade_sentiment(&trades, &start, &end);
    finish_output(print_json(&sentiment))
}

#[instrument(skip_all)]
async fn execute_clusters(args: &ClustersArgs) -> i32 {
    let (start, end) = match resolve_required_range(&args.dates) {
        Ok(range) => range,
        Err(message) => return usage_error(message),
    };
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
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_clusters(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_trade_records(
        &response.data,
        TradeRecordKind::Cluster,
        &CLUSTER_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade clusters",
    )
}

#[instrument(skip_all)]
async fn execute_cluster_bombs(args: &ClusterBombsArgs) -> i32 {
    let (start, end) = match resolve_required_range(&args.dates) {
        Ok(range) => range,
        Err(message) => return usage_error(message),
    };
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
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_cluster_bombs(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_trade_records(
        &response.data,
        TradeRecordKind::ClusterBomb,
        &BOMB_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade clusters",
    )
}

#[instrument(skip_all)]
async fn execute_alerts(args: &AlertsArgs) -> i32 {
    let request = crate::TradeAlertsRequest::new()
        .with_start(args.page.start)
        .with_length(args.page.length)
        .with_order(args.page.order_col, args.page.order_dir.as_str(), "")
        .with_date(args.date.clone());
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_alerts(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_trade_records(
        &response.data,
        TradeRecordKind::Trade,
        &ALERT_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade alerts",
    )
}

#[instrument(skip_all)]
async fn execute_cluster_alerts(args: &AlertsArgs) -> i32 {
    let request = crate::TradeClusterAlertsRequest::new()
        .with_start(args.page.start)
        .with_length(args.page.length)
        .with_order(args.page.order_col, args.page.order_dir.as_str(), "")
        .with_date(args.date.clone());
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_cluster_alerts(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_trade_records(
        &response.data,
        TradeRecordKind::Cluster,
        &CLUSTER_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade clusters",
    )
}

#[instrument(skip_all)]
async fn execute_levels(args: &LevelsArgs) -> i32 {
    if !validate_trade_level_count(args.trade_level_count) {
        return usage_error(
            "--trade-level-count must be one of 5, 10, 20, or 50 for trade level retrieval",
        );
    }
    let ticker = parse_single_ticker(&args.ticker);
    let (start, end) = resolve_with_default(&args.dates, 365);
    let request =
        dashboard_levels_request(&ticker, &start, &end, args.trade_level_count).with_length(-1);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_chart0_trade_levels(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    let mut levels = response.data;
    levels.truncate(args.trade_level_count);
    output_trade_records(
        &levels,
        TradeRecordKind::Level,
        &LEVEL_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
        "trade levels",
    )
}

#[instrument(skip_all)]
async fn execute_level_touches(args: &LevelTouchesArgs) -> i32 {
    if !validate_trade_level_count(args.trade_level_count) {
        return usage_error(
            "--trade-level-count must be one of 5, 10, 20, or 50 for trade level retrieval",
        );
    }
    if !(1..=50).contains(&args.page.length) {
        return usage_error("--length must be between 1 and 50 for trade level touch retrieval");
    }
    let (start, end) = match resolve_required_range(&args.dates) {
        Ok(range) => range,
        Err(message) => return usage_error(message),
    };
    let ticker = parse_single_ticker(&args.ticker);
    let request = TradeLevelTouchesRequest::new()
        .with_start(args.page.start)
        .with_length(args.page.length)
        .with_order(args.page.order_col, args.page.order_dir.as_str(), "")
        .with_level_touch_filters(level_touch_filters(args, &ticker, &start, &end));
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_level_touches(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_trade_records(
        &response.data,
        TradeRecordKind::Level,
        &LEVEL_HEADERS,
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
) -> i32 {
    let result = print_trade_records(records, kind, headers, fields, all_fields, command_path);
    finish_output(result)
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
    print_transformed_record_values_with_allowed_fields(
        records,
        kind,
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
mod tests {
    use crate::{Trade, TradeCluster, TradeClusterAlert, TradeClusterBomb, TradeLevel};
    use serde_json::json;

    use crate::cli::output::write_record_values;

    use super::*;

    fn trade(value: serde_json::Value) -> Trade {
        serde_json::from_value(value).unwrap()
    }

    fn cluster(value: serde_json::Value) -> TradeCluster {
        serde_json::from_value(value).unwrap()
    }

    fn cluster_alert(value: serde_json::Value) -> TradeClusterAlert {
        serde_json::from_value(value).unwrap()
    }

    fn level(value: serde_json::Value) -> TradeLevel {
        serde_json::from_value(value).unwrap()
    }

    fn cluster_bomb(value: serde_json::Value) -> TradeClusterBomb {
        serde_json::from_value(value).unwrap()
    }

    fn cluster_fixture() -> TradeCluster {
        cluster(json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Price": 199.125,
            "Dollars": 20_000_000.126,
            "Volume": 100_000,
            "TradeCount": 4,
            "DollarsMultiplier": 12.345,
            "TradeClusterRank": 2,
            "MinFullDateTime": "2026-01-02T16:00:00+00:00",
            "MaxFullDateTime": "2026-01-02T16:49:31+00:00",
            "EOM": true,
            "OPEX": false,
            "SecurityKey": 123
        }))
    }

    fn cluster_alert_fixture() -> TradeClusterAlert {
        cluster_alert(json!({
            "Ticker": "MSFT",
            "Date": "/Date(1767312000000)/",
            "Price": 312.25,
            "Dollars": 12_000_000.0,
            "Volume": 75_000,
            "TradeCount": 3,
            "TradeClusterRank": 8,
            "MinFullDateTime": "2026-01-02T15:00:00+00:00",
            "MaxFullDateTime": "2026-01-02T15:07:30+00:00",
            "VOLEX": true
        }))
    }

    fn render_cluster_json(fields: Option<&str>, all_fields: bool) -> serde_json::Value {
        let values = crate::cli::common::trade_transforms::transformed_trade_values(
            &[cluster_fixture()],
            TradeRecordKind::Cluster,
        )
        .expect("cluster serializes");
        let mut output = Vec::new();
        write_record_values(
            &mut output,
            &values,
            &CLUSTER_HEADERS,
            fields,
            all_fields,
            None,
        )
        .expect("cluster output renders");

        serde_json::from_slice(&output).expect("valid cluster json")
    }

    fn dashboard_args() -> DashboardArgs {
        DashboardArgs {
            ticker: "aapl".to_string(),
            dates: OptionalDateRangeArgs {
                start_date: None,
                end_date: None,
                days: None,
            },
            ranges: TradeRangeArgs {
                min_volume: None,
                max_volume: None,
                min_price: None,
                max_price: None,
                min_dollars: None,
                max_dollars: None,
            },
            filters: DashboardFilterArgs {
                conditions: None,
                vcd: None,
                relative_size: None,
                dark_pools: None,
                sweeps: None,
                late_prints: None,
                sig_prints: None,
                trade_rank: None,
                premarket: None,
                rth: None,
                ah: None,
                opening: None,
                closing: None,
                phantom: None,
                offsetting: None,
                sector: None,
            },
            count: DEFAULT_DASHBOARD_COUNT,
            fields: None,
            all_fields: false,
        }
    }

    fn dashboard_fixture() -> TradeDashboard {
        TradeDashboard {
            ticker: "AAPL".to_string(),
            date_range: DateRange {
                start: "2026-01-01".to_string(),
                end: "2026-01-02".to_string(),
            },
            count: DEFAULT_DASHBOARD_COUNT,
            trades: vec![trade(json!({
                "Ticker": "AAPL",
                "Date": "/Date(1767312000000)/",
                "FullTimeString24": "16:00:00",
                "Price": 200.0,
                "Dollars": 10_000_000.0,
                "Volume": 50_000,
                "TradeRank": 3,
                "RSIDay": 45.67,
                "RSIHour": 0.0,
                "DarkPool": false,
                "Sweep": true,
                "ClosingTrade": true,
                "SecurityKey": 0,
                "TradeConditions": null
            }))],
            clusters: vec![cluster(json!({
                "Ticker": "AAPL",
                "Date": "/Date(1767312000000)/",
                "Dollars": 20_000_000.0,
                "Volume": 100_000,
                "TradeCount": 4,
                "TradeClusterRank": 2,
                "MinFullDateTime": "2026-01-02T16:00:00+00:00",
                "MaxFullDateTime": "2026-01-02T16:49:31+00:00",
                "SecurityKey": 0
            }))],
            levels: vec![level(json!({
                "Ticker": "AAPL",
                "Price": 199.5,
                "Dollars": 30_000_000.0,
                "Volume": 150_000,
                "Trades": 5,
                "RelativeSize": 0.0,
                "TradeLevelRank": 1,
                "Name": null
            }))],
            cluster_bombs: vec![cluster_bomb(json!({
                "Ticker": "AAPL",
                "Date": "/Date(1767312000000)/",
                "Dollars": 40_000_000.0,
                "Volume": 200_000,
                "TradeCount": 6,
                "TradeClusterBombRank": 1,
                "ExternalFeed": false
            }))],
        }
    }

    #[test]
    fn sentiment_classification_uses_strings_and_ticker_lists() {
        let bear = trade(json!({"Ticker":"ABC","Sector":"Leveraged Bear","Dollars":1.0}));
        let bull = trade(json!({"Ticker":"ABC","Name":"Mega Bull ETF","Dollars":1.0}));
        let fallback = trade(json!({"Ticker":"SQQQ","Dollars":1.0}));
        let unknown = trade(json!({"Ticker":"ABC","Dollars":1.0}));

        assert!(matches!(
            classify_trade_sentiment_side(&bear),
            Some(SentimentSide::Bear)
        ));
        assert!(matches!(
            classify_trade_sentiment_side(&bull),
            Some(SentimentSide::Bull)
        ));
        assert!(matches!(
            classify_trade_sentiment_side(&fallback),
            Some(SentimentSide::Bear)
        ));
        assert!(classify_trade_sentiment_side(&unknown).is_none());
    }

    #[test]
    fn sentiment_signal_thresholds_and_zero_bear_edge_cases() {
        assert_eq!(
            sentiment_signal(Some(0.1), 1.0, 10.0),
            TradeSentimentSignal::ExtremeBear
        );
        assert_eq!(
            sentiment_signal(Some(0.3), 3.0, 10.0),
            TradeSentimentSignal::ModerateBear
        );
        assert_eq!(
            sentiment_signal(Some(1.0), 10.0, 10.0),
            TradeSentimentSignal::Neutral
        );
        assert_eq!(
            sentiment_signal(Some(3.0), 30.0, 10.0),
            TradeSentimentSignal::ModerateBull
        );
        assert_eq!(
            sentiment_signal(Some(6.0), 60.0, 10.0),
            TradeSentimentSignal::ExtremeBull
        );
        assert_eq!(
            sentiment_signal(None, 1.0, 0.0),
            TradeSentimentSignal::ExtremeBull
        );
        assert_eq!(
            sentiment_signal(None, 0.0, 0.0),
            TradeSentimentSignal::Neutral
        );
    }

    #[test]
    fn dashboard_filter_building_matches_endpoint_requirements() {
        let args = DashboardArgs {
            ticker: "aapl".to_string(),
            dates: OptionalDateRangeArgs {
                start_date: None,
                end_date: None,
                days: None,
            },
            ranges: TradeRangeArgs {
                min_volume: Some(10),
                max_volume: Some(20),
                min_price: Some(1.0),
                max_price: Some(2.0),
                min_dollars: Some(3.0),
                max_dollars: Some(4.0),
            },
            filters: DashboardFilterArgs {
                conditions: None,
                vcd: Some(7),
                relative_size: Some(8),
                dark_pools: None,
                sweeps: None,
                late_prints: None,
                sig_prints: None,
                trade_rank: None,
                premarket: None,
                rth: None,
                ah: None,
                opening: None,
                closing: None,
                phantom: None,
                offsetting: None,
                sector: None,
            },
            count: DEFAULT_DASHBOARD_COUNT,
            fields: None,
            all_fields: false,
        };
        let trades = dashboard_trades_request(&args, "AAPL", "2026-01-01", "2026-01-02");
        let clusters = dashboard_clusters_request(&args, "AAPL", "2026-01-01", "2026-01-02");
        let levels =
            dashboard_levels_request("AAPL", "2026-01-01", "2026-01-02", DEFAULT_DASHBOARD_COUNT);
        let bombs = dashboard_bombs_request(&args, "AAPL", "2026-01-01", "2026-01-02");

        assert!(has_filter(trades.extra_values(), "Sort", "Dollars"));
        assert!(
            !trades
                .extra_values()
                .iter()
                .any(|(key, _)| key == "SecurityTypeKey")
        );
        assert!(has_filter(
            clusters.extra_values(),
            "TradeClusterRank",
            "-1"
        ));
        assert!(has_filter(
            levels.extra_values(),
            "Levels",
            &DEFAULT_DASHBOARD_COUNT.to_string()
        ));
        assert!(levels.encode().contains("length=10"));
        assert!(has_filter(
            bombs.extra_values(),
            "TradeClusterBombRank",
            "-1"
        ));
        assert!(
            !bombs
                .extra_values()
                .iter()
                .any(|(key, _)| key == "MinPrice")
        );
    }

    #[test]
    fn dashboard_output_defaults_to_compact_decision_fields() {
        let args = dashboard_args();
        let output = dashboard_output_value(&dashboard_fixture(), &args).unwrap();

        assert_eq!(output["ticker"], "AAPL");
        assert_eq!(output["date_range"]["start"], "2026-01-01");
        assert_eq!(output["count"], DEFAULT_DASHBOARD_COUNT);

        let trade = output["trades"][0].as_object().unwrap();
        assert_eq!(trade["Date"], "2026-01-02");
        assert_eq!(trade["Dollars"], 10_000_000.0);
        assert_eq!(trade["venue"], "lit_sweep");
        assert_eq!(trade["type"], "closing");
        assert!(!trade.contains_key("FullTimeString24"));
        assert!(!trade.contains_key("Time"));
        assert!(!trade.contains_key("DollarsMultiplier"));
        assert!(!trade.contains_key("Ticker"));
        assert!(!trade.contains_key("SecurityKey"));
        assert!(!trade.contains_key("RSI"));
        assert!(!trade.contains_key("RSIDay"));
        assert!(!trade.contains_key("RSIHour"));
        assert!(!trade.contains_key("DarkPool"));
        assert!(!trade.contains_key("Sweep"));
        assert!(!trade.contains_key("ClosingTrade"));
        assert!(!trade.contains_key("TradeConditions"));

        let cluster = output["clusters"][0].as_object().unwrap();
        assert_eq!(cluster["Date"], "2026-01-02");
        assert_eq!(cluster["TradeClusterRank"], 2);
        assert_eq!(cluster["window"], "16:00:00-16:49:31");
        assert!(!cluster.contains_key("MinFullDateTime"));
        assert!(!cluster.contains_key("MaxFullDateTime"));
        assert!(!cluster.contains_key("MinDateTime"));
        assert!(!cluster.contains_key("MaxDateTime"));
        assert!(!cluster.contains_key("DollarsMultiplier"));
        assert!(!cluster.contains_key("Ticker"));

        let level = output["levels"][0].as_object().unwrap();
        assert_eq!(level["TradeLevelRank"], 1);
        assert!(!level.contains_key("RelativeSize"));
        assert!(!level.contains_key("Ticker"));

        let bomb = output["cluster_bombs"][0].as_object().unwrap();
        assert_eq!(bomb["TradeClusterBombRank"], 1);
        assert!(!bomb.contains_key("ExternalFeed"));
    }

    #[test]
    fn compact_headers_use_transformed_trade_fields() {
        assert!(CLUSTER_HEADERS.contains(&"window"));
        assert!(CLUSTER_HEADERS.contains(&"events"));
        assert!(!CLUSTER_HEADERS.contains(&"MinFullDateTime"));
        assert!(!CLUSTER_HEADERS.contains(&"MaxFullDateTime"));
        assert!(!CLUSTER_HEADERS.contains(&"MinDateTime"));
        assert!(!CLUSTER_HEADERS.contains(&"MaxDateTime"));

        assert!(BOMB_HEADERS.contains(&"window"));
        assert!(BOMB_HEADERS.contains(&"events"));
        assert!(!BOMB_HEADERS.contains(&"MinFullDateTime"));
        assert!(!BOMB_HEADERS.contains(&"MaxFullDateTime"));
        assert!(!BOMB_HEADERS.contains(&"MinDateTime"));
        assert!(!BOMB_HEADERS.contains(&"MaxDateTime"));

        assert!(ALERT_HEADERS.contains(&"Time"));
        assert!(ALERT_HEADERS.contains(&"type"));
        assert!(ALERT_HEADERS.contains(&"venue"));
        assert!(ALERT_HEADERS.contains(&"events"));
        assert!(!ALERT_HEADERS.contains(&"FullTimeString24"));
    }

    #[test]
    fn cluster_output_defaults_to_transformed_compact_fields() {
        let output = render_cluster_json(None, false);
        let row = output[0].as_object().unwrap();

        assert_eq!(row["Date"], "2026-01-02");
        assert_eq!(row["Ticker"], "AAPL");
        assert_eq!(row["Price"], 199.13);
        assert_eq!(row["Dollars"], 20_000_000.13);
        assert_eq!(row["TradeCount"], 4);
        assert_eq!(row["TradeClusterRank"], 2);
        assert_eq!(row["window"], "16:00:00-16:49:31");
        assert_eq!(row["events"], json!(["EOM"]));
        assert!(!row.contains_key("MinFullDateTime"));
        assert!(!row.contains_key("MaxFullDateTime"));
        assert!(!row.contains_key("MinDateTime"));
        assert!(!row.contains_key("MaxDateTime"));
        assert!(!row.contains_key("EOM"));
        assert!(!row.contains_key("OPEX"));
        assert!(!row.contains_key("SecurityKey"));
    }

    #[test]
    fn cluster_output_accepts_custom_transformed_fields() {
        let output = render_cluster_json(Some("Date,TradeCount,window"), false);
        let row = output[0].as_object().unwrap();

        assert_eq!(row.len(), 3);
        assert_eq!(row["Date"], "2026-01-02");
        assert_eq!(row["TradeCount"], 4);
        assert_eq!(row["window"], "16:00:00-16:49:31");
    }

    #[test]
    fn trade_output_accepts_discovered_metadata_field_absent_from_rows() {
        let records = vec![trade(json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Price": 200.0,
            "Dollars": 10_000_000.0
        }))];

        print_trade_records(
            &records,
            TradeRecordKind::Trade,
            &TRADE_HEADERS,
            Some("events"),
            false,
            "trade list",
        )
        .expect("discovered trade fields validate before row filtering");
    }

    #[test]
    fn trade_output_rejects_field_missing_from_metadata() {
        let records = vec![trade(json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Price": 200.0,
            "Dollars": 10_000_000.0
        }))];

        let err = print_trade_records(
            &records,
            TradeRecordKind::Trade,
            &TRADE_HEADERS,
            Some("NotAField"),
            false,
            "trade list",
        )
        .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("NotAField"));
    }

    #[test]
    fn output_trade_records_finishes_metadata_validated_output() {
        let records = vec![trade(json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Price": 200.0,
            "Dollars": 10_000_000.0
        }))];

        assert_eq!(
            output_trade_records(
                &records,
                TradeRecordKind::Trade,
                &TRADE_HEADERS,
                Some("events"),
                false,
                "trade list",
            ),
            0
        );
    }

    #[test]
    fn cluster_bomb_output_accepts_cluster_bomb_rank_metadata() {
        let records = vec![cluster_bomb(json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Dollars": 40_000_000.0,
            "TradeCount": 6,
            "TradeClusterBombRank": 1
        }))];

        print_trade_records(
            &records,
            TradeRecordKind::ClusterBomb,
            &BOMB_HEADERS,
            Some("TradeClusterBombRank"),
            false,
            "trade cluster-bombs",
        )
        .expect("cluster-bomb metadata includes cluster-bomb rank");
    }

    #[test]
    fn cluster_output_all_fields_keeps_extra_fields_after_transforms() {
        let output = render_cluster_json(None, true);
        let row = output[0].as_object().unwrap();

        assert_eq!(row["SecurityKey"], 123);
        assert_eq!(row["window"], "16:00:00-16:49:31");
        assert_eq!(row["events"], json!(["EOM"]));
        assert!(!row.contains_key("MinFullDateTime"));
        assert!(!row.contains_key("MaxFullDateTime"));
        assert!(!row.contains_key("MinDateTime"));
        assert!(!row.contains_key("MaxDateTime"));
        assert!(!row.contains_key("EOM"));
        assert!(!row.contains_key("OPEX"));
    }

    #[test]
    fn cluster_alert_output_uses_cluster_transform_headers() {
        let values = crate::cli::common::trade_transforms::transformed_trade_values(
            &[cluster_alert_fixture()],
            TradeRecordKind::Cluster,
        )
        .expect("cluster alert serializes");
        let mut output = Vec::new();
        write_record_values(&mut output, &values, &CLUSTER_HEADERS, None, false, None)
            .expect("cluster alert output renders");
        let output: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let row = output[0].as_object().unwrap();

        assert!(
            CLUSTER_HEADERS
                .iter()
                .all(|header| row.contains_key(*header))
        );
        assert_eq!(row["Ticker"], "MSFT");
        assert_eq!(row["window"], "15:00:00-15:07:30");
        assert_eq!(row["events"], json!(["VOLEX"]));
        assert!(!row.contains_key("MinFullDateTime"));
        assert!(!row.contains_key("MaxFullDateTime"));
        assert!(!row.contains_key("MinDateTime"));
        assert!(!row.contains_key("MaxDateTime"));
    }

    #[test]
    fn dashboard_output_accepts_section_qualified_custom_fields() {
        let mut args = dashboard_args();
        args.fields = Some(
            "trades.Date,trades.Dollars,clusters.TradeCount,levels.Price,cluster_bombs.Volume"
                .to_string(),
        );

        let output = dashboard_output_value(&dashboard_fixture(), &args).unwrap();
        let trade = output["trades"][0].as_object().unwrap();
        assert_eq!(trade.len(), 2);
        assert_eq!(trade["Date"], "2026-01-02");
        assert_eq!(trade["Dollars"], 10_000_000.0);

        let cluster = output["clusters"][0].as_object().unwrap();
        assert_eq!(cluster.len(), 1);
        assert_eq!(cluster["TradeCount"], 4);

        let level = output["levels"][0].as_object().unwrap();
        assert_eq!(level.len(), 1);
        assert_eq!(level["Price"], 199.5);

        let bomb = output["cluster_bombs"][0].as_object().unwrap();
        assert_eq!(bomb.len(), 1);
        assert_eq!(bomb["Volume"], 200_000);
    }

    #[test]
    fn dashboard_output_accepts_discovered_field_absent_from_rows() {
        let mut args = dashboard_args();
        args.fields = Some("trades.venue".to_string());
        let mut dashboard = dashboard_fixture();
        dashboard.trades = vec![trade(json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Price": 200.0,
            "Dollars": 10_000_000.0,
            "DarkPool": false,
            "Sweep": false
        }))];

        let output = dashboard_output_value(&dashboard, &args).unwrap();

        let trade = output["trades"][0].as_object().unwrap();
        assert!(trade.is_empty());
    }

    #[test]
    fn dashboard_output_applies_unqualified_custom_fields_to_all_sections() {
        let mut args = dashboard_args();
        args.fields = Some("Dollars,Volume".to_string());

        let output = dashboard_output_value(&dashboard_fixture(), &args).unwrap();
        for section in ["trades", "clusters", "levels", "cluster_bombs"] {
            let row = output[section][0].as_object().unwrap();
            assert_eq!(row.len(), 2);
            assert!(row.contains_key("Dollars"));
            assert!(row.contains_key("Volume"));
        }
    }

    #[test]
    fn dashboard_output_all_fields_applies_transforms() {
        let mut args = dashboard_args();
        args.all_fields = true;

        let output = dashboard_output_value(&dashboard_fixture(), &args).unwrap();
        let trade = output["trades"][0].as_object().unwrap();
        assert_eq!(trade["Ticker"], "AAPL");
        assert_eq!(trade["SecurityKey"], 0);
        assert_eq!(trade["venue"], "lit_sweep");
        assert_eq!(trade["type"], "closing");
        assert!(!trade.contains_key("FullTimeString24"));
        assert!(!trade.contains_key("Time"));
        assert!(!trade.contains_key("DarkPool"));
        assert!(!trade.contains_key("Sweep"));
        assert!(!trade.contains_key("ClosingTrade"));
        assert!(trade.contains_key("TradeConditions"));
        assert!(trade["TradeConditions"].is_null());

        let cluster = output["clusters"][0].as_object().unwrap();
        assert_eq!(cluster["window"], "16:00:00-16:49:31");
        assert!(!cluster.contains_key("MinFullDateTime"));
        assert!(!cluster.contains_key("MaxFullDateTime"));
        assert!(!cluster.contains_key("MinDateTime"));
        assert!(!cluster.contains_key("MaxDateTime"));
    }

    #[test]
    fn parse_dashboard_fields_splits_section_qualified_names() {
        let fields =
            parse_dashboard_fields("trades.Date,cluster.Dollars,levels.Price,bombs.Volume,Dollars")
                .unwrap();

        assert_eq!(fields.trades, vec!["Date"]);
        assert_eq!(fields.clusters, vec!["Dollars"]);
        assert_eq!(fields.levels, vec!["Price"]);
        assert_eq!(fields.cluster_bombs, vec!["Volume"]);
        assert_eq!(fields.unqualified, vec!["Dollars"]);
    }

    #[test]
    fn parse_dashboard_fields_rejects_unknown_section_names() {
        let err = parse_dashboard_fields("cluster-bomb.Date").unwrap_err();

        assert!(err.contains("unknown dashboard field section `cluster-bomb`"));
        assert!(err.contains("trades, clusters, levels, or cluster_bombs"));
    }

    #[test]
    fn dashboard_output_rejects_case_mismatched_custom_fields() {
        let mut args = dashboard_args();
        args.fields = Some("trades.price".to_string());

        let err = dashboard_output_value(&dashboard_fixture(), &args).unwrap_err();

        assert!(err.contains("no requested dashboard fields matched `trades` rows"));
        assert!(err.contains("case-sensitive"));
    }

    fn has_filter(filters: &[(String, String)], key: &str, value: &str) -> bool {
        filters
            .iter()
            .any(|(filter_key, filter_value)| filter_key == key && filter_value == value)
    }

    #[test]
    fn trade_list_preset_lookup_is_case_insensitive() {
        let preset = find_trade_preset("top-100 rank").expect("preset found");
        assert_eq!(preset.name, "Top-100 Rank");
        assert_eq!(preset.group, "Common");
        assert!(find_trade_preset("does not exist").is_none());
    }

    #[test]
    fn validate_trade_level_count_accepts_only_supported_values() {
        for count in [5, 10, 20, 50] {
            assert!(validate_trade_level_count(count));
        }
        for count in [0, 7, 100] {
            assert!(!validate_trade_level_count(count));
        }
    }

    #[test]
    fn nearest_level_count_clamps_to_valid_api_values() {
        assert_eq!(nearest_level_count(1), 5);
        assert_eq!(nearest_level_count(5), 5);
        assert_eq!(nearest_level_count(7), 10);
        assert_eq!(nearest_level_count(10), 10);
        assert_eq!(nearest_level_count(15), 20);
        assert_eq!(nearest_level_count(20), 20);
        assert_eq!(nearest_level_count(30), 50);
        assert_eq!(nearest_level_count(50), 50);
        assert_eq!(nearest_level_count(100), 50);
    }

    #[test]
    fn tri_state_filter_conversion_and_parser_round_trip() {
        assert_eq!(TriStateFilter::All.as_i8(), -1);
        assert_eq!(TriStateFilter::Enabled.as_i8(), 1);
        assert_eq!(TriStateFilter::Disabled.as_i8(), 0);

        assert_eq!(parse_tri_state_filter("all").unwrap(), TriStateFilter::All);
        assert_eq!(
            parse_tri_state_filter("only").unwrap(),
            TriStateFilter::Enabled
        );
        assert_eq!(
            parse_tri_state_filter("disabled").unwrap(),
            TriStateFilter::Disabled
        );
    }

    #[test]
    fn apply_preset_common_base_sets_default_filters() {
        let preset = find_trade_preset("All Trades").expect("preset found");
        let mut filters = Vec::new();

        apply_preset_filters(&mut filters, preset);

        assert!(has_filter(
            &filters,
            "Conditions",
            "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"
        ));
        assert!(has_filter(&filters, "IncludeOffsetting", "-1"));
        assert!(has_filter(&filters, "IncludePhantom", "-1"));
        assert!(has_filter(&filters, "MaxDollars", "10000000000"));
        assert!(has_filter(&filters, "MinVolume", "10000"));
        assert!(has_filter(&filters, "RelativeSize", "0"));
        assert!(has_filter(&filters, "TradeCount", "3"));
    }

    #[test]
    fn apply_preset_large_base_sets_default_filters() {
        let preset =
            find_trade_preset("All Disproportionately Large Trades").expect("preset found");
        let mut filters = Vec::new();

        apply_preset_filters(&mut filters, preset);

        assert!(has_filter(
            &filters,
            "Conditions",
            "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"
        ));
        assert!(has_filter(&filters, "IncludeOffsetting", "-1"));
        assert!(has_filter(&filters, "IncludePhantom", "-1"));
        assert!(has_filter(&filters, "MaxDollars", "10000000000"));
        assert!(has_filter(&filters, "MinVolume", "10000"));
        assert!(has_filter(&filters, "TradeCount", "3"));
        assert!(
            !filters
                .iter()
                .any(|(filter_key, _)| filter_key == "RelativeSize")
        );
    }

    #[test]
    fn apply_preset_none_base_only_applies_preset_filters() {
        let preset = find_trade_preset("Top-100 Rank; Dark Pool Sweeps").expect("preset found");
        let mut filters = Vec::new();

        apply_preset_filters(&mut filters, preset);

        assert!(has_filter(&filters, "DarkPools", "1"));
        assert!(has_filter(&filters, "Sweeps", "1"));
        assert!(has_filter(&filters, "TradeRank", "100"));
        assert!(has_filter(&filters, "RelativeSize", "0"));
    }

    #[test]
    fn apply_preset_with_extra_filters_overrides_base() {
        let preset = find_trade_preset("Top-10 Rank").expect("preset found");
        let mut filters = Vec::new();

        apply_preset_filters(&mut filters, preset);

        assert!(has_filter(
            &filters,
            "Conditions",
            "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"
        ));
        assert!(has_filter(&filters, "IncludeOffsetting", "-1"));
        assert!(has_filter(&filters, "IncludePhantom", "-1"));
        assert!(has_filter(&filters, "MaxDollars", "10000000000"));
        assert!(has_filter(&filters, "MinVolume", "10000"));
        assert!(has_filter(&filters, "RelativeSize", "0"));
        assert!(has_filter(&filters, "TradeCount", "3"));
        assert!(has_filter(&filters, "TradeRank", "10"));
    }

    #[test]
    fn apply_preset_large_sector_adds_sector_filter() {
        let preset = find_trade_preset("Bear Leverage").expect("preset found");
        let mut filters = Vec::new();

        apply_preset_filters(&mut filters, preset);

        assert!(has_filter(
            &filters,
            "Conditions",
            "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"
        ));
        assert!(has_filter(&filters, "IncludeOffsetting", "-1"));
        assert!(has_filter(&filters, "IncludePhantom", "-1"));
        assert!(has_filter(&filters, "MaxDollars", "10000000000"));
        assert!(has_filter(&filters, "MinVolume", "10000"));
        assert!(has_filter(&filters, "TradeCount", "3"));
        assert!(has_filter(&filters, "SectorIndustry", "X Bear"));
        assert!(has_filter(&filters, "VCD", "97.00"));
    }

    #[test]
    fn classify_trade_sentiment_side_detects_bear_in_sector() {
        let trade = trade(json!({
            "Ticker": "ABC",
            "Sector": "Bear 3x"
        }));

        assert!(matches!(
            classify_trade_sentiment_side(&trade),
            Some(SentimentSide::Bear)
        ));
    }

    #[test]
    fn classify_trade_sentiment_side_detects_bull_in_name() {
        let trade = trade(json!({
            "Ticker": "ABC",
            "Name": "Bull 2x ETF"
        }));

        assert!(matches!(
            classify_trade_sentiment_side(&trade),
            Some(SentimentSide::Bull)
        ));
    }

    #[test]
    fn classify_trade_sentiment_side_detects_bear_in_industry() {
        let trade = trade(json!({
            "Ticker": "ABC",
            "Industry": "Bear Leveraged"
        }));

        assert!(matches!(
            classify_trade_sentiment_side(&trade),
            Some(SentimentSide::Bear)
        ));
    }

    #[test]
    fn classify_trade_sentiment_side_falls_back_to_etf_ticker_bear() {
        let trade = trade(json!({
            "Ticker": "SQQQ",
            "Sector": "Technology",
            "Name": "Nasdaq ETF",
            "Industry": "Exchange Traded Fund"
        }));

        assert!(matches!(
            classify_trade_sentiment_side(&trade),
            Some(SentimentSide::Bear)
        ));
    }

    #[test]
    fn classify_trade_sentiment_side_falls_back_to_etf_ticker_bull() {
        let trade = trade(json!({
            "Ticker": "TQQQ",
            "Sector": "Technology",
            "Name": "Nasdaq ETF",
            "Industry": "Exchange Traded Fund"
        }));

        assert!(matches!(
            classify_trade_sentiment_side(&trade),
            Some(SentimentSide::Bull)
        ));
    }

    #[test]
    fn classify_trade_sentiment_side_returns_none_for_unknown() {
        let trade = trade(json!({
            "Ticker": "AAPL",
            "Sector": "Technology"
        }));

        assert!(classify_trade_sentiment_side(&trade).is_none());
    }

    #[test]
    fn sentiment_signal_extreme_bear() {
        assert_eq!(
            sentiment_signal(Some(0.1), 100.0, 1_000.0),
            TradeSentimentSignal::ExtremeBear
        );
    }

    #[test]
    fn sentiment_signal_moderate_bear() {
        assert_eq!(
            sentiment_signal(Some(0.3), 300.0, 1_000.0),
            TradeSentimentSignal::ModerateBear
        );
    }

    #[test]
    fn sentiment_signal_neutral_ratio() {
        assert_eq!(
            sentiment_signal(Some(1.0), 500.0, 500.0),
            TradeSentimentSignal::Neutral
        );
    }

    #[test]
    fn sentiment_signal_moderate_bull() {
        assert_eq!(
            sentiment_signal(Some(3.0), 3_000.0, 1_000.0),
            TradeSentimentSignal::ModerateBull
        );
    }

    #[test]
    fn sentiment_signal_extreme_bull_ratio() {
        assert_eq!(
            sentiment_signal(Some(6.0), 6_000.0, 1_000.0),
            TradeSentimentSignal::ExtremeBull
        );
    }

    #[test]
    fn sentiment_signal_no_ratio_bull_only() {
        assert_eq!(
            sentiment_signal(None, 100.0, 0.0),
            TradeSentimentSignal::ExtremeBull
        );
    }

    #[test]
    fn sentiment_signal_no_ratio_bear_only() {
        assert_eq!(
            sentiment_signal(None, 0.0, 100.0),
            TradeSentimentSignal::ExtremeBear
        );
    }

    #[test]
    fn sentiment_signal_no_ratio_no_dollars() {
        assert_eq!(
            sentiment_signal(None, 0.0, 0.0),
            TradeSentimentSignal::Neutral
        );
    }

    #[test]
    fn summarize_trade_sentiment_groups_by_day() {
        let trades = vec![
            trade(json!({
                "Ticker": "TQQQ",
                "Date": "/Date(1767312000000)/",
                "Dollars": 1_000_000
            })),
            trade(json!({
                "Ticker": "SOXL",
                "Date": "/Date(1767312000000)/",
                "Dollars": 2_000_000
            })),
            trade(json!({
                "Ticker": "SQQQ",
                "Date": "/Date(1767398400000)/",
                "Dollars": 3_000_000
            })),
        ];

        let summary = summarize_trade_sentiment(&trades, "2026-01-02", "2026-01-03");
        let value = serde_json::to_value(summary).expect("sentiment summary serializes");

        assert_eq!(value["daily"].as_array().unwrap().len(), 2);
        assert_eq!(value["daily"][0]["date"], "2026-01-02");
        assert_eq!(value["daily"][0]["bull"]["trades"], 2);
        assert_eq!(
            value["daily"][0]["bull"]["top_tickers"],
            json!(["SOXL", "TQQQ"])
        );
        assert_eq!(value["daily"][1]["date"], "2026-01-03");
        assert_eq!(value["daily"][1]["bear"]["trades"], 1);
        assert_eq!(value["daily"][1]["bear"]["top_tickers"], json!(["SQQQ"]));
        assert_eq!(value["totals"]["bull"]["trades"], 2);
        assert_eq!(value["totals"]["bear"]["trades"], 1);
        assert_eq!(
            value["totals"]["bull"]["top_tickers"],
            json!(["SOXL", "TQQQ"])
        );
        assert_eq!(value["totals"]["bear"]["top_tickers"], json!(["SQQQ"]));
    }

    #[test]
    fn summarize_trade_sentiment_skips_unknown_tickers() {
        let trades = vec![trade(json!({
            "Ticker": "AAPL",
            "Sector": "Technology",
            "Date": "/Date(1767312000000)/",
            "Dollars": 1_000_000
        }))];

        let summary = summarize_trade_sentiment(&trades, "2026-01-02", "2026-01-02");
        let value = serde_json::to_value(summary).expect("sentiment summary serializes");

        assert_eq!(value["daily"].as_array().unwrap().len(), 0);
        assert_eq!(value["totals"]["bull"]["trades"], 0);
        assert_eq!(value["totals"]["bear"]["trades"], 0);
        assert_eq!(value["totals"]["bull"]["dollars"], 0.0);
        assert_eq!(value["totals"]["bear"]["dollars"], 0.0);
        assert_eq!(value["totals"]["bull"]["top_tickers"], json!([]));
        assert_eq!(value["totals"]["bear"]["top_tickers"], json!([]));
    }

    fn empty_optional_dates() -> OptionalDateRangeArgs {
        OptionalDateRangeArgs {
            start_date: None,
            end_date: None,
            days: None,
        }
    }

    fn empty_trade_ranges() -> TradeRangeArgs {
        TradeRangeArgs {
            min_volume: None,
            max_volume: None,
            min_price: None,
            max_price: None,
            min_dollars: None,
            max_dollars: None,
        }
    }

    fn empty_volume_dollar_ranges() -> VolumeDollarRangeArgs {
        VolumeDollarRangeArgs {
            min_volume: None,
            max_volume: None,
            min_dollars: None,
            max_dollars: None,
        }
    }

    fn default_fixed_page() -> FixedPageArgs {
        FixedPageArgs {
            start: 0,
            order_col: 1,
            order_dir: OrderDirection::Desc,
        }
    }

    fn default_page() -> PageArgs {
        PageArgs {
            start: 0,
            length: 100,
            order_col: 1,
            order_dir: OrderDirection::Desc,
        }
    }

    fn empty_trade_filter_args() -> TradeFilterArgs {
        TradeFilterArgs {
            conditions: None,
            vcd: None,
            security_type: None,
            relative_size: None,
            dark_pools: None,
            sweeps: None,
            late_prints: None,
            sig_prints: None,
            even_shared: None,
            trade_rank: None,
            rank_snapshot: None,
            market_cap: None,
            premarket: None,
            rth: None,
            ah: None,
            opening: None,
            closing: None,
            phantom: None,
            offsetting: None,
            sector: None,
        }
    }

    fn default_clusters_args() -> ClustersArgs {
        ClustersArgs {
            tickers: vec!["AAPL".to_string()],
            dates: empty_optional_dates(),
            ranges: empty_trade_ranges(),
            vcd: None,
            security_type: None,
            relative_size: None,
            sector: None,
            trade_cluster_rank: 100,
            page: default_fixed_page(),
            fields: None,
            all_fields: false,
        }
    }

    fn default_cluster_bombs_args() -> ClusterBombsArgs {
        ClusterBombsArgs {
            tickers: vec!["AAPL".to_string()],
            dates: empty_optional_dates(),
            ranges: empty_volume_dollar_ranges(),
            vcd: None,
            security_type: None,
            relative_size: None,
            sector: None,
            trade_cluster_bomb_rank: -1,
            page: default_fixed_page(),
            fields: None,
            all_fields: false,
        }
    }

    fn default_level_touches_args() -> LevelTouchesArgs {
        LevelTouchesArgs {
            ticker: "AAPL".to_string(),
            dates: empty_optional_dates(),
            ranges: empty_trade_ranges(),
            trade_level_rank: 5,
            trade_level_count: DEFAULT_LEVEL_TOUCH_COUNT,
            vcd: None,
            relative_size: None,
            page: default_page(),
            fields: None,
            all_fields: false,
        }
    }

    #[test]
    fn default_trade_filters_contains_expected_keys() {
        let filters = default_trade_filters(1_000_000.0, 90);

        assert!(has_filter(&filters, "MinVolume", "0"));
        assert!(has_filter(
            &filters,
            "MaxVolume",
            &DEFAULT_MAX_VOLUME.to_string()
        ));
        assert!(has_filter(&filters, "MinDollars", "1000000"));
        assert!(has_filter(&filters, "VCD", "90"));
        assert!(has_filter(&filters, "RelativeSize", "5"));
        assert!(has_filter(&filters, "DarkPools", "-1"));
        assert!(has_filter(&filters, "Sweeps", "-1"));
        assert!(has_filter(&filters, "IncludePremarket", "1"));
    }

    #[test]
    fn default_trade_list_filters_match_har_criteria() {
        let filters = default_trade_list_filters();

        assert!(has_filter(&filters, "MinVolume", "10000"));
        assert!(has_filter(&filters, "MaxVolume", "2000000000"));
        assert!(has_filter(&filters, "MinPrice", "0"));
        assert!(has_filter(&filters, "MaxPrice", "100000"));
        assert!(has_filter(&filters, "MinDollars", "500000"));
        assert!(has_filter(&filters, "MaxDollars", "100000000000"));
        assert!(has_filter(&filters, "Conditions", "0"));
        assert!(has_filter(&filters, "VCD", "0"));
        assert!(has_filter(&filters, "SecurityTypeKey", "-1"));
        assert!(has_filter(&filters, "RelativeSize", "0"));
        assert!(has_filter(&filters, "DarkPools", "-1"));
        assert!(has_filter(&filters, "Sweeps", "-1"));
        assert!(has_filter(&filters, "LatePrints", "-1"));
        assert!(has_filter(&filters, "SignaturePrints", "-1"));
        assert!(has_filter(&filters, "EvenShared", "-1"));
        assert!(has_filter(&filters, "TradeRank", "100"));
        assert!(has_filter(&filters, "TradeRankSnapshot", "-1"));
        assert!(has_filter(&filters, "MarketCap", "0"));
        assert!(has_filter(&filters, "IncludePremarket", "1"));
        assert!(has_filter(&filters, "IncludeRTH", "1"));
        assert!(has_filter(&filters, "IncludeAH", "1"));
        assert!(has_filter(&filters, "IncludeOpening", "1"));
        assert!(has_filter(&filters, "IncludeClosing", "1"));
        assert!(has_filter(&filters, "IncludePhantom", "1"));
        assert!(has_filter(&filters, "IncludeOffsetting", "1"));
    }

    #[test]
    fn trade_list_default_request_matches_har_shape() {
        let (start, end) = resolve_trade_list_range(&empty_optional_dates());
        let mut filters = default_trade_list_filters();
        set_filter(&mut filters, "StartDate", start);
        set_filter(&mut filters, "EndDate", end);

        let request = TradesRequest::new()
            .with_length(DEFAULT_TRADE_LIMIT as i32)
            .with_search("", false)
            .with_order(1, "DESC", "FullTimeString24")
            .with_trade_filters(filters);
        let encoded = request.encode();

        assert!(encoded.contains("length=1000"));
        assert!(encoded.contains("order[0][dir]=DESC"));
        assert!(encoded.contains("order[0][name]=FullTimeString24"));
        assert!(encoded.contains("search[value]="));
        assert!(encoded.contains("search[regex]=false"));
        assert!(encoded.contains("StartDate=Today"));
        assert!(encoded.contains("EndDate=Today"));
        assert!(encoded.contains("MinVolume=10000"));
        assert!(encoded.contains("MaxDollars=100000000000"));
        assert!(encoded.contains("TradeRank=100"));
    }

    #[test]
    fn set_filter_replaces_existing() {
        let mut filters = vec![("VCD".to_string(), "90".to_string())];

        set_filter(&mut filters, "VCD", "95".to_string());

        assert_eq!(filters.iter().filter(|(key, _)| key == "VCD").count(), 1);
        assert!(has_filter(&filters, "VCD", "95"));
    }

    #[test]
    fn set_filter_removes_on_empty_value() {
        let mut filters = vec![("VCD".to_string(), "90".to_string())];

        set_filter(&mut filters, "VCD", String::new());

        assert!(!filters.iter().any(|(key, _)| key == "VCD"));
    }

    #[test]
    fn set_ticker_filters_replaces_existing() {
        let mut filters = vec![("Tickers".to_string(), "AAPL".to_string())];
        let tickers = vec!["MSFT".to_string(), "GOOG".to_string()];

        set_ticker_filters(&mut filters, &tickers, "Tickers");

        assert!(!has_filter(&filters, "Tickers", "AAPL"));
        assert!(has_filter(&filters, "Tickers", "MSFT"));
        assert!(has_filter(&filters, "Tickers", "GOOG"));
        assert_eq!(
            filters.iter().filter(|(key, _)| key == "Tickers").count(),
            2
        );
    }

    #[test]
    fn apply_trade_ranges_sets_volume_and_price_bounds() {
        let mut filters = default_trade_filters(1_000_000.0, 90);
        let ranges = TradeRangeArgs {
            min_volume: Some(123),
            max_volume: None,
            min_price: Some(12.5),
            max_price: None,
            min_dollars: Some(2_500_000.0),
            max_dollars: Some(50_000_000.0),
        };

        apply_trade_ranges(&mut filters, &ranges, 1_000_000.0);

        assert!(has_filter(&filters, "MinVolume", "123"));
        assert!(has_filter(
            &filters,
            "MaxVolume",
            &DEFAULT_MAX_VOLUME.to_string()
        ));
        assert!(has_filter(&filters, "MinPrice", "12.5"));
        assert!(has_filter(
            &filters,
            "MaxPrice",
            &DEFAULT_MAX_PRICE.to_string()
        ));
        assert!(has_filter(&filters, "MinDollars", "2500000"));
        assert!(has_filter(&filters, "MaxDollars", "50000000"));
    }

    #[test]
    fn apply_trade_filter_args_sets_conditions_and_tri_states() {
        let mut filters = default_trade_filters(1_000_000.0, 90);
        let args = TradeFilterArgs {
            conditions: Some("ISO".to_string()),
            vcd: Some(95),
            dark_pools: Some(TriStateFilter::Enabled),
            sweeps: Some(TriStateFilter::Disabled),
            ..empty_trade_filter_args()
        };

        apply_trade_filter_args(&mut filters, &args);

        assert!(has_filter(&filters, "Conditions", "ISO"));
        assert!(has_filter(&filters, "VCD", "95"));
        assert!(has_filter(&filters, "DarkPools", "1"));
        assert!(has_filter(&filters, "Sweeps", "0"));
    }

    #[test]
    fn cluster_filters_builds_correct_filter_set() {
        let args = default_clusters_args();

        let filters = cluster_filters(&args, "2026-01-01", "2026-01-02");

        assert!(has_filter(&filters, "Tickers", "AAPL"));
        assert!(has_filter(&filters, "StartDate", "2026-01-01"));
        assert!(has_filter(&filters, "EndDate", "2026-01-02"));
        assert!(has_filter(&filters, "VCD", "0"));
        assert!(has_filter(&filters, "SecurityTypeKey", "-1"));
        assert!(has_filter(&filters, "RelativeSize", "0"));
        assert!(has_filter(&filters, "TradeClusterRank", "100"));
    }

    #[test]
    fn cluster_request_matches_browser_defaults() {
        let args = default_clusters_args();
        let request = TradeClustersRequest::new()
            .with_start(args.page.start)
            .with_length(DEFAULT_CLUSTER_LENGTH)
            .with_search("", false)
            .with_order(
                args.page.order_col,
                args.page.order_dir.as_str().to_ascii_uppercase(),
                cluster_order_name(args.page.order_col),
            )
            .with_cluster_filters(cluster_filters(&args, "2026-05-20", "2026-05-20"));
        let encoded = request.encode();

        assert!(encoded.contains("length=1000"));
        assert!(encoded.contains("search[value]="));
        assert!(encoded.contains("search[regex]=false"));
        assert!(encoded.contains("order[0][column]=1"));
        assert!(encoded.contains("order[0][dir]=DESC"));
        assert!(encoded.contains("order[0][name]=MinFullTimeString24"));
        assert!(encoded.contains("MinVolume=10000"));
        assert!(encoded.contains("MinDollars=500000"));
        assert!(encoded.contains("RelativeSize=0"));
        assert!(encoded.contains("TradeClusterRank=100"));
    }

    #[test]
    fn cluster_custom_order_omits_default_order_name() {
        assert_eq!(cluster_order_name(1), "MinFullTimeString24");
        assert_eq!(cluster_order_name(3), "");
    }

    #[test]
    fn cluster_bomb_filters_builds_correct_filter_set() {
        let args = default_cluster_bombs_args();

        let filters = cluster_bomb_filters(&args, "2026-01-01", "2026-01-02");

        assert!(has_filter(&filters, "SecurityTypeKey", "0"));
        assert!(has_filter(&filters, "RelativeSize", "0"));
        assert!(has_filter(&filters, "TradeClusterBombRank", "-1"));
        assert!(!filters.iter().any(|(key, _)| key == "MinPrice"));
        assert!(!filters.iter().any(|(key, _)| key == "MaxPrice"));
    }

    #[test]
    fn cluster_bomb_request_matches_browser_defaults() {
        let args = default_cluster_bombs_args();
        let request = TradeClusterBombsRequest::new()
            .with_start(args.page.start)
            .with_length(DEFAULT_CLUSTER_BOMB_LENGTH)
            .with_search("", false)
            .with_order(
                args.page.order_col,
                args.page.order_dir.as_str().to_ascii_uppercase(),
                cluster_bomb_order_name(args.page.order_col),
            )
            .with_cluster_bomb_filters(cluster_bomb_filters(&args, "2026-05-20", "2026-05-20"));
        let encoded = request.encode();

        assert!(encoded.contains("length=100"));
        assert!(encoded.contains("search[value]="));
        assert!(encoded.contains("search[regex]=false"));
        assert!(encoded.contains("columns[12][name]=Charts"));
        assert!(encoded.contains("order[0][column]=1"));
        assert!(encoded.contains("order[0][dir]=DESC"));
        assert!(encoded.contains("order[0][name]=MinFullTimeString24"));
        assert!(encoded.contains("SecurityTypeKey=0"));
        assert!(encoded.contains("RelativeSize=0"));
    }

    #[test]
    fn cluster_bomb_custom_order_omits_default_order_name() {
        assert_eq!(cluster_bomb_order_name(1), "MinFullTimeString24");
        assert_eq!(cluster_bomb_order_name(10), "");
    }

    #[test]
    fn level_touch_filters_includes_level_count() {
        let args = default_level_touches_args();

        let filters = level_touch_filters(&args, "AAPL", "2026-01-01", "2026-01-02");

        assert!(has_filter(&filters, "TradeLevelRank", "5"));
        assert!(has_filter(
            &filters,
            "Levels",
            &DEFAULT_LEVEL_TOUCH_COUNT.to_string()
        ));
    }

    #[test]
    fn dashboard_trades_request_removes_security_type_key() {
        let args = dashboard_args();

        let request = dashboard_trades_request(&args, "AAPL", "2026-01-01", "2026-01-02");

        assert!(has_filter(request.extra_values(), "Tickers", "AAPL"));
        assert!(
            !request
                .extra_values()
                .iter()
                .any(|(key, _)| key == "SecurityTypeKey")
        );
    }

    #[test]
    fn dashboard_clusters_request_sets_cluster_filters() {
        let args = dashboard_args();

        let request = dashboard_clusters_request(&args, "AAPL", "2026-01-01", "2026-01-02");

        assert!(has_filter(request.extra_values(), "Tickers", "AAPL"));
        assert!(has_filter(request.extra_values(), "TradeClusterRank", "-1"));
    }

    #[test]
    fn dashboard_levels_request_uses_display_count_for_length() {
        let request = dashboard_levels_request("AAPL", "2026-01-01", "2026-01-02", 10);

        assert!(request.encode().contains("length=10"));
        assert!(has_filter(request.extra_values(), "Levels", "10"));
    }

    #[test]
    fn dashboard_bombs_request_adds_bomb_rank_filter() {
        let args = dashboard_args();

        let request = dashboard_bombs_request(&args, "AAPL", "2026-01-01", "2026-01-02");

        assert!(has_filter(
            request.extra_values(),
            "TradeClusterBombRank",
            "-1"
        ));
        assert!(
            !request
                .extra_values()
                .iter()
                .any(|(key, _)| key == "TradeClusterRank")
        );
    }
}
