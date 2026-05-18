//! Trade commands: trades, dashboards, sentiment, clusters, alerts, and levels.

use std::collections::HashMap;

mod dashboard;
mod filters;
mod presets;
mod sentiment;

use clap::{Args, Subcommand};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use tracing::instrument;
use volumeleaders_client::{
    TradeClusterBombsRequest, TradeClustersRequest, TradeLevelTouchesRequest, TradesRequest,
};

use crate::cli::TradeArgs;
use crate::common::auth::{handle_api_error, make_client};
use crate::common::dates::resolve_date_range;
use crate::common::tickers::{parse_single_ticker, parse_tickers};
use crate::common::trade_transforms::TradeRecordKind;
use crate::common::types::{OrderDirection, SummaryGroup, TriStateFilter};
use crate::common::{DATE_FMT, TRADE_HEADERS};
use crate::output::{finish_output, print_json, print_transformed_record_values};

use self::dashboard::{TradeDashboard, dashboard_output_value};
use self::filters::{
    apply_trade_filter_args, apply_trade_ranges, cluster_bomb_filters, cluster_filters,
    dashboard_bombs_request, dashboard_clusters_request, dashboard_levels_request,
    dashboard_trades_request, default_trade_filters, level_touch_filters, parse_tri_state_filter,
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
const TRADE_LIST_TICKER_LOOKBACK_DAYS: u32 = 90;
pub(super) const DEFAULT_DASHBOARD_COUNT: usize = 10;
const DEFAULT_DASHBOARD_LOOKBACK_DAYS: u32 = 365;
const DEFAULT_LEVEL_COUNT: usize = 10;
const DEFAULT_LEVEL_TOUCH_COUNT: usize = 50;
pub(super) const DEFAULT_MAX_VOLUME: i64 = 2_000_000_000;
pub(super) const DEFAULT_MAX_PRICE: f64 = 100_000.0;
pub(super) const DEFAULT_MAX_DOLLARS: f64 = 30_000_000_000.0;

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
    "FullTimeString24",
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
    List(ListArgs),
    /// Query a ticker institutional dashboard.
    Dashboard(DashboardArgs),
    /// Summarize leveraged ETF bull and bear flow by day.
    Sentiment(SentimentArgs),
    /// Query aggregated trade clusters.
    Clusters(ClustersArgs),
    /// Query trade cluster bombs.
    #[command(name = "cluster-bombs")]
    ClusterBombs(ClusterBombsArgs),
    /// Query trade alerts for a date.
    Alerts(AlertsArgs),
    /// Query trade cluster alerts for a date.
    #[command(name = "cluster-alerts")]
    ClusterAlerts(AlertsArgs),
    /// Query significant price levels for a ticker.
    Levels(LevelsArgs),
    /// Query trade events at notable price levels.
    #[command(name = "level-touches")]
    LevelTouches(LevelTouchesArgs),
}

/// Arguments for `trade list`.
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
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field after semantic trade transforms.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `trade dashboard`.
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
    /// Comma-separated field list for dashboard output.
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
#[derive(Debug, Args)]
pub struct ClustersArgs {
    /// Tickers as comma-separated or space-separated symbols.
    pub tickers: Vec<String>,
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    #[command(flatten)]
    pub ranges: TradeRangeArgs,
    /// VCD filter.
    #[arg(long)]
    pub vcd: Option<i32>,
    /// Security type key.
    #[arg(long = "security-type")]
    pub security_type: Option<i32>,
    /// Relative size threshold.
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    /// Sector/industry filter.
    #[arg(long)]
    pub sector: Option<String>,
    /// Trade cluster rank filter.
    #[arg(long = "trade-cluster-rank", default_value_t = -1)]
    pub trade_cluster_rank: i32,
    #[command(flatten)]
    pub page: FixedPageArgs,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field after semantic trade transforms.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `trade cluster-bombs`.
#[derive(Debug, Args)]
pub struct ClusterBombsArgs {
    /// Tickers as comma-separated or space-separated symbols.
    pub tickers: Vec<String>,
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    #[command(flatten)]
    pub ranges: VolumeDollarRangeArgs,
    /// VCD filter.
    #[arg(long)]
    pub vcd: Option<i32>,
    /// Security type key.
    #[arg(long = "security-type")]
    pub security_type: Option<i32>,
    /// Relative size threshold.
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    /// Sector/industry filter.
    #[arg(long)]
    pub sector: Option<String>,
    /// Trade cluster bomb rank filter.
    #[arg(long = "trade-cluster-bomb-rank", default_value_t = -1)]
    pub trade_cluster_bomb_rank: i32,
    #[command(flatten)]
    pub page: FixedPageArgs,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field after semantic trade transforms.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for trade alert commands.
#[derive(Debug, Args)]
pub struct AlertsArgs {
    /// Alert date, YYYY-MM-DD.
    #[arg(long, required = true)]
    pub date: String,
    #[command(flatten)]
    pub page: PageArgs,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field after semantic trade transforms.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `trade levels`.
#[derive(Debug, Args)]
pub struct LevelsArgs {
    /// Ticker symbol.
    pub ticker: String,
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    /// Number of price levels to return.
    #[arg(long = "trade-level-count", default_value_t = DEFAULT_LEVEL_COUNT)]
    pub trade_level_count: usize,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field after semantic trade transforms.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `trade level-touches`.
#[derive(Debug, Args)]
pub struct LevelTouchesArgs {
    /// Ticker symbol.
    pub ticker: String,
    #[command(flatten)]
    pub dates: OptionalDateRangeArgs,
    #[command(flatten)]
    pub ranges: TradeRangeArgs,
    /// Trade level rank filter.
    #[arg(long = "trade-level-rank", default_value_t = 5)]
    pub trade_level_rank: i32,
    /// Number of levels to include.
    #[arg(long = "trade-level-count", default_value_t = DEFAULT_LEVEL_TOUCH_COUNT)]
    pub trade_level_count: usize,
    /// VCD filter.
    #[arg(long)]
    pub vcd: Option<i32>,
    /// Relative size threshold.
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    #[command(flatten)]
    pub page: PageArgs,
    /// Comma-separated field list for output.
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
#[derive(Debug, Args)]
pub struct TradeRangeArgs {
    #[arg(long = "min-volume")]
    pub min_volume: Option<i64>,
    #[arg(long = "max-volume")]
    pub max_volume: Option<i64>,
    #[arg(long = "min-price")]
    pub min_price: Option<f64>,
    #[arg(long = "max-price")]
    pub max_price: Option<f64>,
    #[arg(long = "min-dollars")]
    pub min_dollars: Option<f64>,
    #[arg(long = "max-dollars")]
    pub max_dollars: Option<f64>,
}

/// Trade volume and dollars range flags.
#[derive(Debug, Args)]
pub struct VolumeDollarRangeArgs {
    #[arg(long = "min-volume")]
    pub min_volume: Option<i64>,
    #[arg(long = "max-volume")]
    pub max_volume: Option<i64>,
    #[arg(long = "min-dollars")]
    pub min_dollars: Option<f64>,
    #[arg(long = "max-dollars")]
    pub max_dollars: Option<f64>,
}

/// Shared trade filter flags.
#[derive(Debug, Args)]
pub struct TradeFilterArgs {
    #[arg(long)]
    pub conditions: Option<String>,
    #[arg(long)]
    pub vcd: Option<i32>,
    #[arg(long = "security-type")]
    pub security_type: Option<i32>,
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    #[arg(long = "dark-pools", value_parser = parse_tri_state_filter)]
    pub dark_pools: Option<TriStateFilter>,
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub sweeps: Option<TriStateFilter>,
    #[arg(long = "late-prints", value_parser = parse_tri_state_filter)]
    pub late_prints: Option<TriStateFilter>,
    #[arg(long = "sig-prints", value_parser = parse_tri_state_filter)]
    pub sig_prints: Option<TriStateFilter>,
    #[arg(long = "even-shared", value_parser = parse_tri_state_filter)]
    pub even_shared: Option<TriStateFilter>,
    #[arg(long = "trade-rank")]
    pub trade_rank: Option<i32>,
    #[arg(long = "rank-snapshot")]
    pub rank_snapshot: Option<i32>,
    #[arg(long = "market-cap")]
    pub market_cap: Option<i32>,
    #[arg(long = "premarket", value_parser = parse_tri_state_filter)]
    pub premarket: Option<TriStateFilter>,
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub rth: Option<TriStateFilter>,
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub ah: Option<TriStateFilter>,
    #[arg(long = "opening", value_parser = parse_tri_state_filter)]
    pub opening: Option<TriStateFilter>,
    #[arg(long = "closing", value_parser = parse_tri_state_filter)]
    pub closing: Option<TriStateFilter>,
    #[arg(long = "phantom", value_parser = parse_tri_state_filter)]
    pub phantom: Option<TriStateFilter>,
    #[arg(long = "offsetting", value_parser = parse_tri_state_filter)]
    pub offsetting: Option<TriStateFilter>,
    #[arg(long)]
    pub sector: Option<String>,
}

/// Dashboard trade filters. The chart endpoint does not accept the heavier
/// list-only filters such as security type, even-share, rank snapshot, or
/// market-cap filters.
#[derive(Debug, Args)]
pub struct DashboardFilterArgs {
    #[arg(long)]
    pub conditions: Option<String>,
    #[arg(long)]
    pub vcd: Option<i32>,
    #[arg(long = "relative-size")]
    pub relative_size: Option<i32>,
    #[arg(long = "dark-pools", value_parser = parse_tri_state_filter)]
    pub dark_pools: Option<TriStateFilter>,
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub sweeps: Option<TriStateFilter>,
    #[arg(long = "late-prints", value_parser = parse_tri_state_filter)]
    pub late_prints: Option<TriStateFilter>,
    #[arg(long = "sig-prints", value_parser = parse_tri_state_filter)]
    pub sig_prints: Option<TriStateFilter>,
    #[arg(long = "trade-rank")]
    pub trade_rank: Option<i32>,
    #[arg(long = "premarket", value_parser = parse_tri_state_filter)]
    pub premarket: Option<TriStateFilter>,
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub rth: Option<TriStateFilter>,
    #[arg(long, value_parser = parse_tri_state_filter)]
    pub ah: Option<TriStateFilter>,
    #[arg(long = "opening", value_parser = parse_tri_state_filter)]
    pub opening: Option<TriStateFilter>,
    #[arg(long = "closing", value_parser = parse_tri_state_filter)]
    pub closing: Option<TriStateFilter>,
    #[arg(long = "phantom", value_parser = parse_tri_state_filter)]
    pub phantom: Option<TriStateFilter>,
    #[arg(long = "offsetting", value_parser = parse_tri_state_filter)]
    pub offsetting: Option<TriStateFilter>,
    #[arg(long)]
    pub sector: Option<String>,
}

/// DataTables page flags with length.
#[derive(Debug, Args)]
pub struct PageArgs {
    #[arg(long, default_value_t = 0)]
    pub start: i32,
    #[arg(long, default_value_t = 100)]
    pub length: i32,
    #[arg(long = "order-col", default_value_t = 1)]
    pub order_col: i32,
    #[arg(long = "order-dir", value_enum, default_value = "desc")]
    pub order_dir: OrderDirection,
}

/// DataTables page flags without length.
#[derive(Debug, Args)]
pub struct FixedPageArgs {
    #[arg(long, default_value_t = 0)]
    pub start: i32,
    #[arg(long = "order-col", default_value_t = 1)]
    pub order_col: i32,
    #[arg(long = "order-dir", value_enum, default_value = "desc")]
    pub order_dir: OrderDirection,
}

/// Handles the trade command group.
#[instrument(skip_all)]
pub async fn handle(args: &TradeArgs, pretty: bool) -> i32 {
    match &args.command {
        TradeCommand::List(list_args) => execute_list(list_args, pretty).await,
        TradeCommand::Dashboard(dashboard_args) => execute_dashboard(dashboard_args, pretty).await,
        TradeCommand::Sentiment(sentiment_args) => execute_sentiment(sentiment_args, pretty).await,
        TradeCommand::Clusters(cluster_args) => execute_clusters(cluster_args, pretty).await,
        TradeCommand::ClusterBombs(bomb_args) => execute_cluster_bombs(bomb_args, pretty).await,
        TradeCommand::Alerts(alert_args) => execute_alerts(alert_args, pretty).await,
        TradeCommand::ClusterAlerts(alert_args) => execute_cluster_alerts(alert_args, pretty).await,
        TradeCommand::Levels(level_args) => execute_levels(level_args, pretty).await,
        TradeCommand::LevelTouches(touch_args) => execute_level_touches(touch_args, pretty).await,
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

#[instrument(skip_all)]
async fn execute_list(args: &ListArgs, pretty: bool) -> i32 {
    if args.group_by.is_some() && !args.summary {
        eprintln!("--group-by only works with --summary");
        return 1;
    }
    if args.summary && (args.fields.is_some() || args.all_fields) {
        eprintln!("--fields and --all-fields cannot be used with --summary");
        return 1;
    }

    let tickers = parse_ticker_args(&args.tickers);
    let default_days = if tickers.is_empty() {
        0
    } else {
        TRADE_LIST_TICKER_LOOKBACK_DAYS
    };
    let (start, end) = resolve_with_default(&args.dates, default_days);
    let mut filters = default_trade_filters(args.ranges.min_dollars.unwrap_or(500_000.0), 97);
    apply_trade_ranges(&mut filters, &args.ranges, 500_000.0);
    apply_trade_filter_args(&mut filters, &args.filters);

    if let Some(preset_name) = &args.preset {
        let preset = match find_trade_preset(preset_name) {
            Some(preset) => preset,
            None => {
                eprintln!("unknown trade preset: {preset_name}");
                return 1;
            }
        };
        filters = default_trade_filters(args.ranges.min_dollars.unwrap_or(500_000.0), 97);
        apply_preset_filters(&mut filters, preset);
        apply_trade_ranges(&mut filters, &args.ranges, 500_000.0);
        apply_trade_filter_args(&mut filters, &args.filters);
    }
    set_filter(&mut filters, "StartDate", start.clone());
    set_filter(&mut filters, "EndDate", end.clone());
    set_ticker_filters(&mut filters, &tickers, "Tickers");

    let request = TradesRequest::new().with_trade_filters(filters);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let trades = match client.get_trades_limit(&request, args.limit).await {
        Ok(trades) => trades,
        Err(err) => return handle_api_error(err),
    };

    let output = if args.summary {
        let group = args.group_by.unwrap_or(SummaryGroup::Ticker);
        let summary = build_summary(&trades, group, &start, &end);
        print_json(&summary, pretty)
    } else {
        print_trade_records(
            &trades,
            TradeRecordKind::Trade,
            pretty,
            &TRADE_HEADERS,
            args.fields.as_deref(),
            args.all_fields,
        )
    };
    finish_output(output)
}

#[instrument(skip_all)]
async fn execute_dashboard(args: &DashboardArgs, pretty: bool) -> i32 {
    let ticker = parse_single_ticker(&args.ticker);
    let (start, end) = resolve_with_default(&args.dates, DEFAULT_DASHBOARD_LOOKBACK_DAYS);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };

    let trades_req = dashboard_trades_request(args, &ticker, &start, &end);
    let clusters_req = dashboard_clusters_request(args, &ticker, &start, &end);
    let levels_req =
        dashboard_levels_request(&ticker, &start, &end, nearest_level_count(args.count));
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
        Err(message) => {
            eprintln!("field error: {message}");
            return 1;
        }
    };
    finish_output(print_json(&dashboard, pretty))
}

#[instrument(skip_all)]
async fn execute_sentiment(args: &SentimentArgs, pretty: bool) -> i32 {
    let (start, end) = match resolve_required_range(&args.dates) {
        Ok(range) => range,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
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
    finish_output(print_json(&sentiment, pretty))
}

#[instrument(skip_all)]
async fn execute_clusters(args: &ClustersArgs, pretty: bool) -> i32 {
    let (start, end) = match resolve_required_range(&args.dates) {
        Ok(range) => range,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };
    let request = TradeClustersRequest::new()
        .with_start(args.page.start)
        .with_length(-1)
        .with_order(args.page.order_col, args.page.order_dir.as_str(), "")
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
        pretty,
        &CLUSTER_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
    )
}

#[instrument(skip_all)]
async fn execute_cluster_bombs(args: &ClusterBombsArgs, pretty: bool) -> i32 {
    let (start, end) = match resolve_required_range(&args.dates) {
        Ok(range) => range,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };
    let request = TradeClusterBombsRequest::new()
        .with_start(args.page.start)
        .with_length(-1)
        .with_order(args.page.order_col, args.page.order_dir.as_str(), "")
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
        pretty,
        &BOMB_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
    )
}

#[instrument(skip_all)]
async fn execute_alerts(args: &AlertsArgs, pretty: bool) -> i32 {
    let request = volumeleaders_client::TradeAlertsRequest::new()
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
        pretty,
        &ALERT_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
    )
}

#[instrument(skip_all)]
async fn execute_cluster_alerts(args: &AlertsArgs, pretty: bool) -> i32 {
    let request = volumeleaders_client::TradeClusterAlertsRequest::new()
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
        pretty,
        &CLUSTER_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
    )
}

#[instrument(skip_all)]
async fn execute_levels(args: &LevelsArgs, pretty: bool) -> i32 {
    if !validate_trade_level_count(args.trade_level_count) {
        eprintln!("--trade-level-count must be one of 5, 10, 20, or 50 for trade level retrieval");
        return 1;
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
        pretty,
        &LEVEL_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
    )
}

#[instrument(skip_all)]
async fn execute_level_touches(args: &LevelTouchesArgs, pretty: bool) -> i32 {
    if !validate_trade_level_count(args.trade_level_count) {
        eprintln!("--trade-level-count must be one of 5, 10, 20, or 50 for trade level retrieval");
        return 1;
    }
    if !(1..=50).contains(&args.page.length) {
        eprintln!("--length must be between 1 and 50 for trade level touch retrieval");
        return 1;
    }
    let (start, end) = match resolve_required_range(&args.dates) {
        Ok(range) => range,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
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
        pretty,
        &LEVEL_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
    )
}

fn output_trade_records<T: Serialize>(
    records: &[T],
    kind: TradeRecordKind,
    pretty: bool,
    headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> i32 {
    let result = print_trade_records(records, kind, pretty, headers, fields, all_fields);
    finish_output(result)
}

fn print_trade_records<T: Serialize>(
    records: &[T],
    kind: TradeRecordKind,
    pretty: bool,
    headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> std::io::Result<()> {
    print_transformed_record_values(records, kind, pretty, headers, fields, all_fields)
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
    trades: &[volumeleaders_client::Trade],
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

fn summary_key(trade: &volumeleaders_client::Trade, group: SummaryGroup) -> String {
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

pub(super) fn trade_day(trade: &volumeleaders_client::Trade) -> String {
    trade
        .date
        .as_ref()
        .and_then(|date| date.0.map(|dt| dt.format(DATE_FMT).to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn add_summary_group(acc: &mut TradeGroupAccumulator, trade: &volumeleaders_client::Trade) {
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
    use serde_json::json;
    use volumeleaders_client::{
        Trade, TradeCluster, TradeClusterAlert, TradeClusterBomb, TradeLevel,
    };

    use crate::output::write_record_values;

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
        let values = crate::common::trade_transforms::transformed_trade_values(
            &[cluster_fixture()],
            TradeRecordKind::Cluster,
        )
        .expect("cluster serializes");
        let mut output = Vec::new();
        write_record_values(
            &mut output,
            &values,
            false,
            &CLUSTER_HEADERS,
            fields,
            all_fields,
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
        assert!(levels.encode().contains("length=-1"));
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
        assert!(!trade.contains_key("DollarsMultiplier"));
        assert!(!trade.contains_key("Ticker"));
        assert!(!trade.contains_key("SecurityKey"));
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

        assert!(BOMB_HEADERS.contains(&"window"));
        assert!(BOMB_HEADERS.contains(&"events"));
        assert!(!BOMB_HEADERS.contains(&"MinFullDateTime"));
        assert!(!BOMB_HEADERS.contains(&"MaxFullDateTime"));

        assert!(ALERT_HEADERS.contains(&"type"));
        assert!(ALERT_HEADERS.contains(&"venue"));
        assert!(ALERT_HEADERS.contains(&"events"));
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
    fn cluster_output_all_fields_keeps_extra_fields_after_transforms() {
        let output = render_cluster_json(None, true);
        let row = output[0].as_object().unwrap();

        assert_eq!(row["SecurityKey"], 123);
        assert_eq!(row["window"], "16:00:00-16:49:31");
        assert_eq!(row["events"], json!(["EOM"]));
        assert!(!row.contains_key("MinFullDateTime"));
        assert!(!row.contains_key("MaxFullDateTime"));
        assert!(!row.contains_key("EOM"));
        assert!(!row.contains_key("OPEX"));
    }

    #[test]
    fn cluster_alert_output_uses_cluster_transform_headers() {
        let values = crate::common::trade_transforms::transformed_trade_values(
            &[cluster_alert_fixture()],
            TradeRecordKind::Cluster,
        )
        .expect("cluster alert serializes");
        let mut output = Vec::new();
        write_record_values(&mut output, &values, false, &CLUSTER_HEADERS, None, false)
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
        assert!(!trade.contains_key("DarkPool"));
        assert!(!trade.contains_key("Sweep"));
        assert!(!trade.contains_key("ClosingTrade"));
        assert!(trade.contains_key("TradeConditions"));
        assert!(trade["TradeConditions"].is_null());

        let cluster = output["clusters"][0].as_object().unwrap();
        assert_eq!(cluster["window"], "16:00:00-16:49:31");
        assert!(!cluster.contains_key("MinFullDateTime"));
        assert!(!cluster.contains_key("MaxFullDateTime"));
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
}
