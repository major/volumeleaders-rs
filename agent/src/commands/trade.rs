//! Trade commands: trades, dashboards, sentiment, clusters, alerts, and levels.

use std::collections::HashMap;

use clap::{Args, Subcommand};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use serde_json::{Map, Value};
use tracing::instrument;
use volumeleaders_client::{
    DataTablesColumn, TradeClusterBombsRequest, TradeClustersRequest, TradeLevelTouchesRequest,
    TradeLevelsRequest, TradesRequest,
};

use crate::cli::TradeArgs;
use crate::common::auth::{handle_api_error, make_client};
use crate::common::dates::resolve_date_range;
use crate::common::tickers::{parse_single_ticker, parse_tickers};
use crate::common::types::{OrderDirection, OutputFormat, SummaryGroup, TriStateFilter};
use crate::output::{finish_output, print_delimited, print_json, print_records};

const DEFAULT_TRADE_LIMIT: usize = 1_000;
const TRADE_LIST_TICKER_LOOKBACK_DAYS: u32 = 90;
const DEFAULT_DASHBOARD_COUNT: usize = 10;
const DEFAULT_DASHBOARD_LOOKBACK_DAYS: u32 = 365;
const DEFAULT_LEVEL_COUNT: usize = 10;
const DEFAULT_LEVEL_TOUCH_COUNT: usize = 50;
const DEFAULT_MAX_VOLUME: i64 = 2_000_000_000;
const DEFAULT_MAX_PRICE: f64 = 100_000.0;
const DEFAULT_MAX_DOLLARS: f64 = 30_000_000_000.0;

const TRADE_HEADERS: [&str; 15] = [
    "Ticker",
    "Date",
    "FullTimeString24",
    "Price",
    "Volume",
    "Dollars",
    "DollarsMultiplier",
    "CumulativeDistribution",
    "TradeRank",
    "RelativeSize",
    "DarkPool",
    "Sweep",
    "Sector",
    "Industry",
    "TradeConditions",
];
const CLUSTER_HEADERS: [&str; 11] = [
    "Date",
    "Ticker",
    "Price",
    "Dollars",
    "Volume",
    "TradeCount",
    "DollarsMultiplier",
    "CumulativeDistribution",
    "TradeClusterRank",
    "MinFullDateTime",
    "MaxFullDateTime",
];
const BOMB_HEADERS: [&str; 10] = [
    "Date",
    "Ticker",
    "Dollars",
    "Volume",
    "TradeCount",
    "DollarsMultiplier",
    "CumulativeDistribution",
    "TradeClusterBombRank",
    "MinFullDateTime",
    "MaxFullDateTime",
];
const LEVEL_HEADERS: [&str; 8] = [
    "Ticker",
    "Price",
    "Dollars",
    "Volume",
    "Trades",
    "RelativeSize",
    "CumulativeDistribution",
    "TradeLevelRank",
];
const ALERT_HEADERS: [&str; 9] = [
    "Ticker",
    "Date",
    "FullTimeString24",
    "AlertType",
    "TradeID",
    "Price",
    "Volume",
    "Dollars",
    "TradeRank",
];
const SENTIMENT_HEADERS: [&str; 9] = [
    "date",
    "bear_trades",
    "bear_dollars",
    "bear_top_tickers",
    "bull_trades",
    "bull_dollars",
    "bull_top_tickers",
    "ratio",
    "signal",
];

const DASHBOARD_TOP_LEVEL_FIELDS: [&str; 3] = ["ticker", "date_range", "count"];
const DASHBOARD_COMPACT_TRADE_FIELDS: [&str; 11] = [
    "Date",
    "FullTimeString24",
    "Price",
    "Dollars",
    "Volume",
    "TradeRank",
    "TradeCount",
    "type",
    "venue",
    "events",
    "TradeConditions",
];
const DASHBOARD_COMPACT_CLUSTER_FIELDS: [&str; 8] = [
    "Date",
    "Price",
    "Dollars",
    "Volume",
    "TradeCount",
    "TradeClusterRank",
    "window",
    "events",
];
const DASHBOARD_COMPACT_LEVEL_FIELDS: [&str; 6] = [
    "Price",
    "Dollars",
    "Volume",
    "Trades",
    "RelativeSize",
    "TradeLevelRank",
];
const DASHBOARD_COMPACT_BOMB_FIELDS: [&str; 7] = [
    "Date",
    "Dollars",
    "Volume",
    "TradeCount",
    "TradeClusterBombRank",
    "window",
    "events",
];

const BULL_TICKERS: &[&str] = &[
    "AAPU", "AMDL", "BITU", "BOIL", "BRZU", "CURE", "CWEB", "DFEN", "DIG", "DPST", "DRN", "EDC",
    "ERX", "FAS", "FNGU", "GUSH", "HIBL", "LABU", "MIDU", "NAIL", "NVDL", "QLD", "ROM", "SOXL",
    "SPXL", "SSO", "TECL", "TMF", "TNA", "TQQQ", "TSLL", "TURB", "UDOW", "UMDD", "UPRO", "URTY",
    "USD", "UWM", "WEBL", "YINN",
];
const BEAR_TICKERS: &[&str] = &[
    "AAPD", "AMDD", "BERZ", "BITI", "BNKD", "BZQ", "DUST", "EDZ", "ERY", "FAZ", "HIBS", "KOLD",
    "LABD", "MEXZ", "MYY", "NVDD", "QID", "REK", "REW", "RXD", "SARK", "SCO", "SDD", "SDOW", "SDS",
    "SEF", "SH", "SMDD", "SOXS", "SPDN", "SPXU", "SPXS", "SQQQ", "SRS", "SSG", "SVIX", "TSDD",
    "TSLQ", "TSLS", "TZA", "UVIX", "WEBS", "YANG", "YCS", "ZSL",
];

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
    /// Output format.
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,
    /// Maximum number of trades to return.
    #[arg(long, default_value_t = DEFAULT_TRADE_LIMIT)]
    pub limit: usize,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
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
    /// Output format.
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,
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
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
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
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
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
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
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
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
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
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
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

#[instrument(skip_all)]
async fn execute_list(args: &ListArgs, pretty: bool) -> i32 {
    if args.group_by.is_some() && !args.summary {
        eprintln!("--group-by only works with --summary");
        return 1;
    }
    if args.summary && args.format != OutputFormat::Json {
        eprintln!("summary mode only supports JSON output");
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
        print_records(
            &trades,
            args.format,
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
    let output = match args.format {
        OutputFormat::Json => print_json(&sentiment, pretty),
        OutputFormat::Csv | OutputFormat::Tsv => {
            let rows = flatten_sentiment(&sentiment);
            print_delimited(&rows, args.format, &SENTIMENT_HEADERS)
        }
    };
    finish_output(output)
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
        .with_order(args.page.order_col, order_dir_str(args.page.order_dir), "")
        .with_cluster_filters(cluster_filters(args, &start, &end));
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_clusters(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_records(
        &response.data,
        args.format,
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
        .with_order(args.page.order_col, order_dir_str(args.page.order_dir), "")
        .with_cluster_bomb_filters(cluster_bomb_filters(args, &start, &end));
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_cluster_bombs(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_records(
        &response.data,
        args.format,
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
        .with_order(args.page.order_col, order_dir_str(args.page.order_dir), "")
        .with_date(args.date.clone());
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_alerts(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_records(
        &response.data,
        args.format,
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
        .with_order(args.page.order_col, order_dir_str(args.page.order_dir), "")
        .with_date(args.date.clone());
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_cluster_alerts(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_records(
        &response.data,
        args.format,
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
    output_records(
        &levels,
        args.format,
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
        .with_order(args.page.order_col, order_dir_str(args.page.order_dir), "")
        .with_level_touch_filters(level_touch_filters(args, &ticker, &start, &end));
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let response = match client.get_trade_level_touches(&request).await {
        Ok(response) => response,
        Err(err) => return handle_api_error(err),
    };
    output_records(
        &response.data,
        args.format,
        pretty,
        &LEVEL_HEADERS,
        args.fields.as_deref(),
        args.all_fields,
    )
}

fn output_records<T: Serialize>(
    records: &[T],
    format: OutputFormat,
    pretty: bool,
    headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> i32 {
    let result = print_records(records, format, pretty, headers, fields, all_fields);
    finish_output(result)
}

fn parse_tri_state_filter(value: &str) -> Result<TriStateFilter, String> {
    match value.to_ascii_lowercase().as_str() {
        "all" | "-1" => Ok(TriStateFilter::All),
        "only" | "enabled" | "1" => Ok(TriStateFilter::Enabled),
        "disabled" | "0" => Ok(TriStateFilter::Disabled),
        _ => Err("expected all, only, disabled, -1, 1, or 0".to_string()),
    }
}

fn parse_ticker_args(args: &[String]) -> Vec<String> {
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

fn default_trade_filters(min_dollars: f64, vcd: i32) -> Vec<(String, String)> {
    vec![
        pair("MinVolume", "0"),
        pair("MaxVolume", DEFAULT_MAX_VOLUME.to_string()),
        pair("MinPrice", "0"),
        pair("MaxPrice", format_float(DEFAULT_MAX_PRICE)),
        pair("MinDollars", format_float(min_dollars)),
        pair("MaxDollars", format_float(DEFAULT_MAX_DOLLARS)),
        pair("Conditions", "-1"),
        pair("VCD", vcd.to_string()),
        pair("SecurityTypeKey", "-1"),
        pair("RelativeSize", "5"),
        pair("DarkPools", "-1"),
        pair("Sweeps", "-1"),
        pair("LatePrints", "-1"),
        pair("SignaturePrints", "-1"),
        pair("EvenShared", "-1"),
        pair("TradeRank", "-1"),
        pair("TradeRankSnapshot", "-1"),
        pair("MarketCap", "0"),
        pair("IncludePremarket", "1"),
        pair("IncludeRTH", "1"),
        pair("IncludeAH", "1"),
        pair("IncludeOpening", "1"),
        pair("IncludeClosing", "1"),
        pair("IncludePhantom", "1"),
        pair("IncludeOffsetting", "1"),
    ]
}

fn apply_trade_ranges(
    filters: &mut Vec<(String, String)>,
    args: &TradeRangeArgs,
    default_min: f64,
) {
    set_filter(
        filters,
        "MinVolume",
        args.min_volume.unwrap_or(0).to_string(),
    );
    set_filter(
        filters,
        "MaxVolume",
        args.max_volume.unwrap_or(DEFAULT_MAX_VOLUME).to_string(),
    );
    set_filter(
        filters,
        "MinPrice",
        format_float(args.min_price.unwrap_or(0.0)),
    );
    set_filter(
        filters,
        "MaxPrice",
        format_float(args.max_price.unwrap_or(DEFAULT_MAX_PRICE)),
    );
    set_filter(
        filters,
        "MinDollars",
        format_float(args.min_dollars.unwrap_or(default_min)),
    );
    set_filter(
        filters,
        "MaxDollars",
        format_float(args.max_dollars.unwrap_or(DEFAULT_MAX_DOLLARS)),
    );
}

fn apply_trade_filter_args(filters: &mut Vec<(String, String)>, args: &TradeFilterArgs) {
    if let Some(value) = &args.conditions {
        set_filter(filters, "Conditions", value.clone());
    }
    if let Some(value) = args.vcd {
        set_filter(filters, "VCD", value.to_string());
    }
    if let Some(value) = args.security_type {
        set_filter(filters, "SecurityTypeKey", value.to_string());
    }
    if let Some(value) = args.relative_size {
        set_filter(filters, "RelativeSize", value.to_string());
    }
    apply_tri_state(filters, "DarkPools", args.dark_pools);
    apply_tri_state(filters, "Sweeps", args.sweeps);
    apply_tri_state(filters, "LatePrints", args.late_prints);
    apply_tri_state(filters, "SignaturePrints", args.sig_prints);
    apply_tri_state(filters, "EvenShared", args.even_shared);
    if let Some(value) = args.trade_rank {
        set_filter(filters, "TradeRank", value.to_string());
    }
    if let Some(value) = args.rank_snapshot {
        set_filter(filters, "TradeRankSnapshot", value.to_string());
    }
    if let Some(value) = args.market_cap {
        set_filter(filters, "MarketCap", value.to_string());
    }
    apply_tri_state(filters, "IncludePremarket", args.premarket);
    apply_tri_state(filters, "IncludeRTH", args.rth);
    apply_tri_state(filters, "IncludeAH", args.ah);
    apply_tri_state(filters, "IncludeOpening", args.opening);
    apply_tri_state(filters, "IncludeClosing", args.closing);
    apply_tri_state(filters, "IncludePhantom", args.phantom);
    apply_tri_state(filters, "IncludeOffsetting", args.offsetting);
    if let Some(value) = &args.sector {
        set_filter(filters, "SectorIndustry", value.clone());
    }
}

fn apply_dashboard_filter_args(filters: &mut Vec<(String, String)>, args: &DashboardFilterArgs) {
    if let Some(value) = &args.conditions {
        set_filter(filters, "Conditions", value.clone());
    }
    if let Some(value) = args.vcd {
        set_filter(filters, "VCD", value.to_string());
    }
    if let Some(value) = args.relative_size {
        set_filter(filters, "RelativeSize", value.to_string());
    }
    apply_tri_state(filters, "DarkPools", args.dark_pools);
    apply_tri_state(filters, "Sweeps", args.sweeps);
    apply_tri_state(filters, "LatePrints", args.late_prints);
    apply_tri_state(filters, "SignaturePrints", args.sig_prints);
    if let Some(value) = args.trade_rank {
        set_filter(filters, "TradeRank", value.to_string());
    }
    apply_tri_state(filters, "IncludePremarket", args.premarket);
    apply_tri_state(filters, "IncludeRTH", args.rth);
    apply_tri_state(filters, "IncludeAH", args.ah);
    apply_tri_state(filters, "IncludeOpening", args.opening);
    apply_tri_state(filters, "IncludeClosing", args.closing);
    apply_tri_state(filters, "IncludePhantom", args.phantom);
    apply_tri_state(filters, "IncludeOffsetting", args.offsetting);
    if let Some(value) = &args.sector {
        set_filter(filters, "SectorIndustry", value.clone());
    }
}

fn apply_tri_state(
    filters: &mut Vec<(String, String)>,
    key: &'static str,
    value: Option<TriStateFilter>,
) {
    if let Some(value) = value {
        set_filter(filters, key, value.as_i8().to_string());
    }
}

fn set_ticker_filters(filters: &mut Vec<(String, String)>, tickers: &[String], key: &'static str) {
    filters.retain(|(existing_key, _)| existing_key != key);
    for ticker in tickers {
        filters.push((key.to_string(), ticker.clone()));
    }
}

fn set_filter(filters: &mut Vec<(String, String)>, key: &'static str, value: String) {
    filters.retain(|(existing_key, _)| existing_key != key);
    if !value.is_empty() {
        filters.push((key.to_string(), value));
    }
}

fn pair(key: &'static str, value: impl Into<String>) -> (String, String) {
    (key.to_string(), value.into())
}

fn format_float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn order_dir_str(dir: OrderDirection) -> &'static str {
    match dir {
        OrderDirection::Asc => "asc",
        OrderDirection::Desc => "desc",
    }
}

fn dashboard_trades_request(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> TradesRequest {
    let mut filters = default_trade_filters(args.ranges.min_dollars.unwrap_or(0.0), 0);
    apply_trade_ranges(&mut filters, &args.ranges, 0.0);
    apply_dashboard_filter_args(&mut filters, &args.filters);
    set_filter(&mut filters, "Tickers", ticker.to_string());
    set_filter(&mut filters, "StartDate", start.to_string());
    set_filter(&mut filters, "EndDate", end.to_string());
    set_filter(&mut filters, "Sort", "Dollars".to_string());
    remove_filters(
        &mut filters,
        &[
            "SecurityTypeKey",
            "EvenShared",
            "TradeRankSnapshot",
            "MarketCap",
        ],
    );
    TradesRequest::new()
        .with_columns(trade_chart_columns())
        .with_length(args.count as i32)
        .with_search("", false)
        .with_order(0, "desc", "FullTimeString24")
        .with_trade_filters(filters)
}

fn dashboard_clusters_request(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> TradeClustersRequest {
    TradeClustersRequest::new()
        .with_columns(trade_cluster_chart_columns())
        .with_length(args.count as i32)
        .with_search("", false)
        .with_order(3, "desc", "Sh")
        .with_cluster_filters(dashboard_cluster_filters(args, ticker, start, end))
}

fn dashboard_levels_request(
    ticker: &str,
    start: &str,
    end: &str,
    count: usize,
) -> TradeLevelsRequest {
    TradeLevelsRequest::new()
        .with_columns(trade_level_chart_columns())
        .with_length(-1)
        .with_search("", false)
        .with_order(0, "desc", "Price")
        .with_chart_filters(ticker, start, end, count)
}

fn dashboard_bombs_request(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> TradeClusterBombsRequest {
    TradeClusterBombsRequest::new()
        .with_columns(trade_cluster_bomb_chart_columns())
        .with_length(args.count as i32)
        .with_search("", false)
        .with_order(2, "desc", "Sh")
        .with_cluster_bomb_filters(dashboard_bomb_filters(args, ticker, start, end))
}

fn dashboard_cluster_filters(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> Vec<(String, String)> {
    vec![
        pair("Tickers", ticker.to_string()),
        pair("StartDate", start.to_string()),
        pair("EndDate", end.to_string()),
        pair("MinVolume", args.ranges.min_volume.unwrap_or(0).to_string()),
        pair(
            "MaxVolume",
            args.ranges
                .max_volume
                .unwrap_or(DEFAULT_MAX_VOLUME)
                .to_string(),
        ),
        pair(
            "MinPrice",
            format_float(args.ranges.min_price.unwrap_or(0.0)),
        ),
        pair(
            "MaxPrice",
            format_float(args.ranges.max_price.unwrap_or(DEFAULT_MAX_PRICE)),
        ),
        pair(
            "MinDollars",
            format_float(args.ranges.min_dollars.unwrap_or(500_000.0)),
        ),
        pair(
            "MaxDollars",
            format_float(args.ranges.max_dollars.unwrap_or(DEFAULT_MAX_DOLLARS)),
        ),
        pair("VCD", args.filters.vcd.unwrap_or(0).to_string()),
        pair(
            "RelativeSize",
            args.filters.relative_size.unwrap_or(0).to_string(),
        ),
        pair("TradeClusterRank", "-1"),
        pair("Sort", "Dollars"),
    ]
}

fn dashboard_bomb_filters(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> Vec<(String, String)> {
    let mut filters = dashboard_cluster_filters(args, ticker, start, end);
    remove_filters(&mut filters, &["MinPrice", "MaxPrice", "TradeClusterRank"]);
    filters.push(pair("TradeClusterBombRank", "-1"));
    filters
}

fn remove_filters(filters: &mut Vec<(String, String)>, names: &[&str]) {
    filters.retain(|(key, _)| !names.contains(&key.as_str()));
}

fn trade_chart_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("FullTimeString24", "FullTimeString24", true, true),
        DataTablesColumn::new("Volume", "Sh", true, true),
        DataTablesColumn::new("Price", "Price", true, true),
        DataTablesColumn::new("Dollars", "$$", true, true),
        DataTablesColumn::new("DollarsMultiplier", "RS", true, true),
        DataTablesColumn::new("TradeRank", "R", true, true),
        DataTablesColumn::new("LastComparibleTradeDate", "Last Comp", true, true),
    ]
}

fn trade_cluster_chart_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("MinFullTimeString24", "MinFullTimeString24", true, true),
        DataTablesColumn::new("Price", "Price", true, true),
        DataTablesColumn::new("TradeCount", "TradeCount", true, true),
        DataTablesColumn::new("Volume", "Sh", true, true),
        DataTablesColumn::new("Dollars", "$$", true, true),
        DataTablesColumn::new("DollarsMultiplier", "RS", true, true),
        DataTablesColumn::new("TradeClusterRank", "R", true, true),
        DataTablesColumn::new("LastComparibleTradeClusterDate", "Last Comp", true, true),
    ]
}

fn trade_level_chart_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("Price", "Price", true, true),
        DataTablesColumn::new("Dollars", "$$", true, true),
        DataTablesColumn::new("Volume", "Sh", true, true),
        DataTablesColumn::new("Trades", "Trades", true, true),
        DataTablesColumn::new("RelativeSize", "RS", true, true),
        DataTablesColumn::new("CumulativeDistribution", "PCT", true, true),
        DataTablesColumn::new("TradeLevelRank", "Rank", true, true),
        DataTablesColumn::new("Dates", "Dates", true, true),
    ]
}

fn trade_cluster_bomb_chart_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("MinFullTimeString24", "MinFullTimeString24", true, true),
        DataTablesColumn::new("TradeCount", "TradeCount", true, true),
        DataTablesColumn::new("Volume", "Sh", true, true),
        DataTablesColumn::new("Dollars", "$$", true, true),
        DataTablesColumn::new("DollarsMultiplier", "RS", true, true),
        DataTablesColumn::new("CumulativeDistribution", "PCT", true, true),
        DataTablesColumn::new("TradeClusterBombRank", "R", true, true),
        DataTablesColumn::new(
            "LastComparableTradeClusterBombDate",
            "Last Comp",
            true,
            true,
        ),
    ]
}

fn cluster_filters(args: &ClustersArgs, start: &str, end: &str) -> Vec<(String, String)> {
    let tickers = parse_ticker_args(&args.tickers).join(",");
    let mut filters = range_base_filters(
        &tickers,
        start,
        end,
        &args.ranges,
        args.ranges.min_dollars.unwrap_or(10_000_000.0),
    );
    filters.push(pair("VCD", args.vcd.unwrap_or(0).to_string()));
    filters.push(pair(
        "SecurityTypeKey",
        args.security_type.unwrap_or(-1).to_string(),
    ));
    filters.push(pair(
        "RelativeSize",
        args.relative_size.unwrap_or(5).to_string(),
    ));
    filters.push(pair(
        "TradeClusterRank",
        args.trade_cluster_rank.to_string(),
    ));
    if let Some(sector) = &args.sector {
        filters.push(pair("SectorIndustry", sector.clone()));
    }
    filters
}

fn cluster_bomb_filters(args: &ClusterBombsArgs, start: &str, end: &str) -> Vec<(String, String)> {
    let tickers = parse_ticker_args(&args.tickers).join(",");
    let mut filters = vec![
        pair("Tickers", tickers),
        pair("StartDate", start.to_string()),
        pair("EndDate", end.to_string()),
        pair("MinVolume", args.ranges.min_volume.unwrap_or(0).to_string()),
        pair(
            "MaxVolume",
            args.ranges
                .max_volume
                .unwrap_or(DEFAULT_MAX_VOLUME)
                .to_string(),
        ),
        pair(
            "MinDollars",
            format_float(args.ranges.min_dollars.unwrap_or(0.0)),
        ),
        pair(
            "MaxDollars",
            format_float(args.ranges.max_dollars.unwrap_or(DEFAULT_MAX_DOLLARS)),
        ),
        pair("VCD", args.vcd.unwrap_or(0).to_string()),
        pair(
            "SecurityTypeKey",
            args.security_type.unwrap_or(-1).to_string(),
        ),
        pair("RelativeSize", args.relative_size.unwrap_or(5).to_string()),
        pair(
            "TradeClusterBombRank",
            args.trade_cluster_bomb_rank.to_string(),
        ),
    ];
    if let Some(sector) = &args.sector {
        filters.push(pair("SectorIndustry", sector.clone()));
    }
    filters
}

fn range_base_filters(
    tickers: &str,
    start: &str,
    end: &str,
    ranges: &TradeRangeArgs,
    default_min_dollars: f64,
) -> Vec<(String, String)> {
    vec![
        pair("Tickers", tickers.to_string()),
        pair("StartDate", start.to_string()),
        pair("EndDate", end.to_string()),
        pair("MinVolume", ranges.min_volume.unwrap_or(0).to_string()),
        pair(
            "MaxVolume",
            ranges.max_volume.unwrap_or(DEFAULT_MAX_VOLUME).to_string(),
        ),
        pair("MinPrice", format_float(ranges.min_price.unwrap_or(0.0))),
        pair(
            "MaxPrice",
            format_float(ranges.max_price.unwrap_or(DEFAULT_MAX_PRICE)),
        ),
        pair(
            "MinDollars",
            format_float(ranges.min_dollars.unwrap_or(default_min_dollars)),
        ),
        pair(
            "MaxDollars",
            format_float(ranges.max_dollars.unwrap_or(DEFAULT_MAX_DOLLARS)),
        ),
    ]
}

fn level_touch_filters(
    args: &LevelTouchesArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> Vec<(String, String)> {
    let mut filters = range_base_filters(ticker, start, end, &args.ranges, 500_000.0);
    filters.push(pair("VCD", args.vcd.unwrap_or(0).to_string()));
    filters.push(pair(
        "RelativeSize",
        args.relative_size.unwrap_or(5).to_string(),
    ));
    filters.push(pair("TradeLevelRank", args.trade_level_rank.to_string()));
    filters.push(pair("Levels", args.trade_level_count.to_string()));
    filters
}

fn validate_trade_level_count(count: usize) -> bool {
    matches!(count, 5 | 10 | 20 | 50)
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
struct TradeDashboard {
    ticker: String,
    date_range: DateRange,
    count: usize,
    trades: Vec<volumeleaders_client::Trade>,
    clusters: Vec<volumeleaders_client::TradeCluster>,
    levels: Vec<volumeleaders_client::TradeLevel>,
    cluster_bombs: Vec<volumeleaders_client::TradeClusterBomb>,
}

#[derive(Debug, Serialize)]
struct DateRange {
    start: String,
    end: String,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct DashboardFieldSelection {
    unqualified: Vec<String>,
    trades: Vec<String>,
    clusters: Vec<String>,
    levels: Vec<String>,
    cluster_bombs: Vec<String>,
}

const CURRENCY_FIELDS: &[&str] = &[
    "Price",
    "Dollars",
    "ClosePrice",
    "Bid",
    "Ask",
    "AverageBlockSizeDollars",
    "AHInstitutionalDollars",
    "TotalInstitutionalDollars",
    "ClosingTradeDollars",
    "TotalDollars",
];

const NON_CURRENCY_FLOAT_FIELDS: &[&str] = &[
    "DollarsMultiplier",
    "PercentDailyVolume",
    "RelativeSize",
    "CumulativeDistribution",
    "RSIHour",
    "RSIDay",
];

const RANK_SENTINEL_FIELDS: &[&str] = &[
    "TradeRank",
    "TradeClusterRank",
    "TradeLevelRank",
    "TradeClusterBombRank",
];

/// Apply semantic transforms to dashboard rows before field filtering.
///
/// Transforms run universally (including `--all-fields`). They collapse
/// redundant boolean groups into richer fields (e.g. `DarkPool` + `Sweep`
/// become `venue`) and strip noise (sentinels, excess precision, verbose
/// timestamps). The raw API keys they replace are not preserved.
fn transform_dashboard(map: &mut Map<String, Value>) {
    for section in ["trades", "clusters", "levels", "cluster_bombs"] {
        let Some(Value::Array(rows)) = map.get_mut(section) else {
            continue;
        };
        for row in rows {
            let Some(row_map) = row.as_object_mut() else {
                continue;
            };
            if section == "trades" {
                collapse_trade_type(row_map);
                collapse_venue(row_map);
                omit_redundant_time(row_map);
            }
            if section == "clusters" || section == "cluster_bombs" {
                collapse_time_window(row_map);
            }
            collapse_calendar_events(row_map);
            omit_sentinel_ranks(row_map);
            round_currency_fields(row_map);
            round_float_fields(row_map);
            compact_date_timezone(row_map);
        }
    }
}

/// Collapse `OpeningTrade` and `ClosingTrade` booleans into a single
/// `"type"` field: `"opening"`, `"closing"`, or omitted when neither.
fn collapse_trade_type(row: &mut Map<String, Value>) {
    let opening = row
        .remove("OpeningTrade")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let closing = row
        .remove("ClosingTrade")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if opening {
        row.insert("type".to_string(), Value::String("opening".to_string()));
    } else if closing {
        row.insert("type".to_string(), Value::String("closing".to_string()));
    }
}

/// Remove rank fields whose value is a sentinel (9999 or 0 both mean unranked).
fn omit_sentinel_ranks(row: &mut Map<String, Value>) {
    for &field in RANK_SENTINEL_FIELDS {
        let is_sentinel = row
            .get(field)
            .and_then(Value::as_i64)
            .is_some_and(|n| n == 9999 || n == 0);
        if is_sentinel {
            row.remove(field);
        }
    }
}

/// Round currency fields to 2 decimal places.
fn round_currency_fields(row: &mut Map<String, Value>) {
    for &field in CURRENCY_FIELDS {
        let rounded = row
            .get(field)
            .and_then(Value::as_f64)
            .map(|f| (f * 100.0).round() / 100.0);
        if let Some(n) = rounded.and_then(serde_json::Number::from_f64) {
            row.insert(field.to_string(), Value::Number(n));
        }
    }
}

/// Round non-currency float fields to 2 decimal places.
fn round_float_fields(row: &mut Map<String, Value>) {
    for &field in NON_CURRENCY_FLOAT_FIELDS {
        let rounded = row
            .get(field)
            .and_then(Value::as_f64)
            .map(|f| (f * 100.0).round() / 100.0);
        if let Some(n) = rounded.and_then(serde_json::Number::from_f64) {
            row.insert(field.to_string(), Value::Number(n));
        }
    }
}

/// Compact date-time string values: strip `+00:00` to `Z`, and collapse
/// midnight timestamps (`T00:00:00Z`) to date-only (`2026-05-08`).
fn compact_date_timezone(row: &mut Map<String, Value>) {
    for value in row.values_mut() {
        let Some(s) = value.as_str() else { continue };
        if let Some(prefix) = s.strip_suffix("+00:00") {
            // Check for midnight after stripping timezone
            if let Some(date) = prefix.strip_suffix("T00:00:00") {
                *value = Value::String(date.to_string());
            } else {
                *value = Value::String(format!("{prefix}Z"));
            }
        } else if let Some(date) = s.strip_suffix("T00:00:00Z") {
            *value = Value::String(date.to_string());
        }
    }
}

const CALENDAR_EVENT_FIELDS: &[&str] = &["EOM", "EOQ", "EOY", "OPEX", "VOLEX"];

/// Collapse calendar-marker booleans into an `"events"` array.
///
/// Produces `"events": ["EOQ", "OPEX"]` when active; omitted entirely when
/// no calendar events apply.
fn collapse_calendar_events(row: &mut Map<String, Value>) {
    let mut events = Vec::new();
    for &field in CALENDAR_EVENT_FIELDS {
        let is_true = row.remove(field).and_then(|v| v.as_bool()).unwrap_or(false);
        if is_true {
            events.push(Value::String(field.to_string()));
        }
    }
    if !events.is_empty() {
        row.insert("events".to_string(), Value::Array(events));
    }
}

/// Collapse `DarkPool` and `Sweep` booleans into a single `"venue"` field.
///
/// Lit (the common case) is omitted to save tokens. Only non-default venues
/// are emitted: `"lit_sweep"`, `"dark_pool"`, `"dark_pool_sweep"`.
fn collapse_venue(row: &mut Map<String, Value>) {
    let dark_pool = row
        .remove("DarkPool")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let sweep = row
        .remove("Sweep")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let venue = match (dark_pool, sweep) {
        (false, false) => return,
        (false, true) => "lit_sweep",
        (true, false) => "dark_pool",
        (true, true) => "dark_pool_sweep",
    };
    row.insert("venue".to_string(), Value::String(venue.to_string()));
}

/// Remove `FullTimeString24` when its value is implied by the trade type:
/// closing trades are always `"16:00:00"`, opening trades always `"09:30:01"`.
fn omit_redundant_time(row: &mut Map<String, Value>) {
    let trade_type = row.get("type").and_then(Value::as_str);
    let time = row.get("FullTimeString24").and_then(Value::as_str);
    let redundant = matches!(
        (trade_type, time),
        (Some("closing"), Some("16:00:00")) | (Some("opening"), Some("09:30:01"))
    );
    if redundant {
        row.remove("FullTimeString24");
    }
}

/// Collapse `MinFullDateTime` and `MaxFullDateTime` into a single
/// `"window"` field showing the time range (e.g. `"16:00:00-16:49:31"`).
///
/// The date portion is redundant with `Date`. If either field is missing
/// or the time portion cannot be extracted, the originals are preserved.
fn collapse_time_window(row: &mut Map<String, Value>) {
    let extract_time = |v: &Value| -> Option<String> {
        let s = v.as_str()?;
        let after_t = s.split('T').nth(1)?;
        let time = after_t
            .strip_suffix("+00:00")
            .or_else(|| after_t.strip_suffix("Z"))
            .unwrap_or(after_t);
        Some(time.to_string())
    };

    let min_time = row.get("MinFullDateTime").and_then(&extract_time);
    let max_time = row.get("MaxFullDateTime").and_then(&extract_time);

    if let (Some(min), Some(max)) = (min_time, max_time) {
        row.remove("MinFullDateTime");
        row.remove("MaxFullDateTime");
        row.insert("window".to_string(), Value::String(format!("{min}-{max}")));
    }
}

fn dashboard_output_value(
    dashboard: &TradeDashboard,
    args: &DashboardArgs,
) -> Result<Value, String> {
    let mut value = serde_json::to_value(dashboard).unwrap_or(Value::Null);
    let Some(map) = value.as_object_mut() else {
        return Ok(value);
    };

    transform_dashboard(map);

    match args.fields.as_deref().map(str::trim) {
        _ if args.all_fields => Ok(value),
        Some(fields) if fields.eq_ignore_ascii_case("all") => Ok(value),
        Some(fields) if !fields.is_empty() => {
            let selection = parse_dashboard_fields(fields)?;
            apply_selected_dashboard_fields(map, &selection)?;
            Ok(value)
        }
        _ => {
            apply_compact_dashboard_fields(map);
            Ok(value)
        }
    }
}

fn parse_dashboard_fields(fields: &str) -> Result<DashboardFieldSelection, String> {
    let mut selection = DashboardFieldSelection::default();
    for field in fields
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
    {
        if let Some((section, name)) = field.split_once('.') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            match section.trim().to_ascii_lowercase().as_str() {
                "trades" | "trade" => selection.trades.push(name.to_string()),
                "clusters" | "cluster" => selection.clusters.push(name.to_string()),
                "levels" | "level" => selection.levels.push(name.to_string()),
                "cluster_bombs" | "cluster-bombs" | "bombs" | "bomb" => {
                    selection.cluster_bombs.push(name.to_string());
                }
                _ => {
                    return Err(format!(
                        "unknown dashboard field section `{}` in `{}`; use trades, clusters, levels, or cluster_bombs",
                        section.trim(),
                        field
                    ));
                }
            }
        } else {
            selection.unqualified.push(field.to_string());
        }
    }
    Ok(selection)
}

fn apply_compact_dashboard_fields(map: &mut Map<String, Value>) {
    retain_dashboard_top_level(map);
    filter_dashboard_section(map, "trades", &DASHBOARD_COMPACT_TRADE_FIELDS, true);
    filter_dashboard_section(map, "clusters", &DASHBOARD_COMPACT_CLUSTER_FIELDS, true);
    filter_dashboard_section(map, "levels", &DASHBOARD_COMPACT_LEVEL_FIELDS, true);
    filter_dashboard_section(map, "cluster_bombs", &DASHBOARD_COMPACT_BOMB_FIELDS, true);
}

fn apply_selected_dashboard_fields(
    map: &mut Map<String, Value>,
    selection: &DashboardFieldSelection,
) -> Result<(), String> {
    retain_dashboard_top_level(map);
    apply_selected_dashboard_section(map, "trades", &selection.trades, selection)?;
    apply_selected_dashboard_section(map, "clusters", &selection.clusters, selection)?;
    apply_selected_dashboard_section(map, "levels", &selection.levels, selection)?;
    apply_selected_dashboard_section(map, "cluster_bombs", &selection.cluster_bombs, selection)?;
    Ok(())
}

fn apply_selected_dashboard_section(
    map: &mut Map<String, Value>,
    section: &str,
    section_fields: &[String],
    selection: &DashboardFieldSelection,
) -> Result<(), String> {
    let fields = dashboard_section_fields(section_fields, &selection.unqualified);
    if fields.is_empty() {
        map.remove(section);
        return Ok(());
    }
    let matched = filter_dashboard_section(map, section, &fields, false);
    if matched == 0 && section_has_rows(map, section) {
        return Err(format!(
            "no requested dashboard fields matched `{section}` rows; field names are case-sensitive"
        ));
    }
    Ok(())
}

fn dashboard_section_fields(section_fields: &[String], unqualified: &[String]) -> Vec<String> {
    section_fields
        .iter()
        .chain(unqualified)
        .filter(|field| !field.trim().is_empty())
        .cloned()
        .collect()
}

fn retain_dashboard_top_level(map: &mut Map<String, Value>) {
    map.retain(|key, _| {
        DASHBOARD_TOP_LEVEL_FIELDS.contains(&key.as_str()) || is_dashboard_section(key)
    });
}

fn is_dashboard_section(key: &str) -> bool {
    matches!(key, "trades" | "clusters" | "levels" | "cluster_bombs")
}

fn filter_dashboard_section<F>(
    map: &mut Map<String, Value>,
    section: &str,
    fields: &[F],
    omit_empty: bool,
) -> usize
where
    F: AsRef<str>,
{
    let Some(Value::Array(rows)) = map.get_mut(section) else {
        return 0;
    };
    let mut matched = 0;
    for row in rows {
        let Some(row_map) = row.as_object_mut() else {
            continue;
        };
        row_map.retain(|key, value| {
            let selected = fields.iter().any(|field| field.as_ref() == key);
            if selected {
                matched += 1;
            }
            selected && (!omit_empty || !is_empty_dashboard_value(value))
        });
    }
    matched
}

fn section_has_rows(map: &Map<String, Value>, section: &str) -> bool {
    matches!(map.get(section), Some(Value::Array(rows)) if !rows.is_empty())
}

fn is_empty_dashboard_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(false) => true,
        Value::Number(_) => false,
        Value::String(value) => value.is_empty(),
        Value::Array(values) => values.is_empty(),
        Value::Object(values) => values.is_empty(),
        Value::Bool(true) => false,
    }
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

fn trade_day(trade: &volumeleaders_client::Trade) -> String {
    trade
        .date
        .as_ref()
        .and_then(|date| date.0.map(|dt| dt.format("%Y-%m-%d").to_string()))
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

#[derive(Debug, Serialize)]
struct TradeSentiment {
    date_range: DateRange,
    daily: Vec<TradeSentimentDay>,
    totals: TradeSentimentTotals,
}

#[derive(Debug, Serialize)]
struct TradeSentimentDay {
    date: String,
    bear: TradeSentimentSide,
    bull: TradeSentimentSide,
    ratio: Option<f64>,
    signal: TradeSentimentSignal,
}

#[derive(Debug, Serialize)]
struct TradeSentimentTotals {
    bear: TradeSentimentSide,
    bull: TradeSentimentSide,
    ratio: Option<f64>,
    signal: TradeSentimentSignal,
}

#[derive(Clone, Debug, Default, Serialize)]
struct TradeSentimentSide {
    trades: usize,
    dollars: f64,
    top_tickers: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
enum TradeSentimentSignal {
    ExtremeBear,
    ModerateBear,
    Neutral,
    ModerateBull,
    ExtremeBull,
}

#[derive(Default)]
struct SentimentAccumulator {
    trades: usize,
    dollars: f64,
    ticker_dollars: HashMap<String, f64>,
}

#[derive(Default)]
struct SentimentDayAccumulator {
    bear: SentimentAccumulator,
    bull: SentimentAccumulator,
}

#[derive(Debug, Serialize)]
struct SentimentRow {
    date: String,
    bear_trades: usize,
    bear_dollars: f64,
    bear_top_tickers: String,
    bull_trades: usize,
    bull_dollars: f64,
    bull_top_tickers: String,
    ratio: Option<f64>,
    signal: TradeSentimentSignal,
}

fn summarize_trade_sentiment(
    trades: &[volumeleaders_client::Trade],
    start: &str,
    end: &str,
) -> TradeSentiment {
    let mut days = HashMap::<String, SentimentDayAccumulator>::new();
    let mut totals = SentimentDayAccumulator::default();
    for trade in trades {
        let Some(side) = classify_trade_sentiment_side(trade) else {
            continue;
        };
        let day = trade_day(trade);
        if day == "unknown" {
            continue;
        }
        days.entry(day).or_default().add(side, trade);
        totals.add(side, trade);
    }
    let mut day_keys: Vec<String> = days.keys().cloned().collect();
    day_keys.sort();
    let daily = day_keys
        .into_iter()
        .filter_map(|day| days.remove(&day).map(|acc| acc.summary(day)))
        .collect();
    TradeSentiment {
        date_range: DateRange {
            start: start.to_string(),
            end: end.to_string(),
        },
        daily,
        totals: totals.summary_totals(),
    }
}

impl SentimentDayAccumulator {
    fn add(&mut self, side: SentimentSide, trade: &volumeleaders_client::Trade) {
        match side {
            SentimentSide::Bear => self.bear.add(trade),
            SentimentSide::Bull => self.bull.add(trade),
        }
    }

    fn summary(self, date: String) -> TradeSentimentDay {
        let bear_dollars = self.bear.dollars;
        let bull_dollars = self.bull.dollars;
        let ratio = sentiment_ratio(bull_dollars, bear_dollars);
        TradeSentimentDay {
            date,
            bear: self.bear.summary(),
            bull: self.bull.summary(),
            ratio,
            signal: sentiment_signal(ratio, bull_dollars, bear_dollars),
        }
    }

    fn summary_totals(self) -> TradeSentimentTotals {
        let bear_dollars = self.bear.dollars;
        let bull_dollars = self.bull.dollars;
        let ratio = sentiment_ratio(bull_dollars, bear_dollars);
        TradeSentimentTotals {
            bear: self.bear.summary(),
            bull: self.bull.summary(),
            ratio,
            signal: sentiment_signal(ratio, bull_dollars, bear_dollars),
        }
    }
}

impl SentimentAccumulator {
    fn add(&mut self, trade: &volumeleaders_client::Trade) {
        self.trades += 1;
        let dollars = trade.dollars.and_then(|d| d.to_f64()).unwrap_or(0.0);
        self.dollars += dollars;
        let ticker = trade.ticker.as_deref().unwrap_or("unknown").to_string();
        *self.ticker_dollars.entry(ticker).or_default() += dollars;
    }

    fn summary(self) -> TradeSentimentSide {
        TradeSentimentSide {
            trades: self.trades,
            dollars: self.dollars,
            top_tickers: top_sentiment_tickers(self.ticker_dollars, 3),
        }
    }
}

#[derive(Clone, Copy)]
enum SentimentSide {
    Bear,
    Bull,
}

fn classify_trade_sentiment_side(trade: &volumeleaders_client::Trade) -> Option<SentimentSide> {
    for field in [&trade.sector, &trade.name, &trade.industry]
        .into_iter()
        .filter_map(Option::as_deref)
    {
        let lower = field.to_ascii_lowercase();
        if lower.contains("bear") {
            return Some(SentimentSide::Bear);
        }
        if lower.contains("bull") {
            return Some(SentimentSide::Bull);
        }
    }
    leveraged_etf_direction(trade.ticker.as_deref().unwrap_or_default())
}

fn leveraged_etf_direction(ticker: &str) -> Option<SentimentSide> {
    let ticker = ticker.trim().to_ascii_uppercase();
    if BEAR_TICKERS.contains(&ticker.as_str()) {
        Some(SentimentSide::Bear)
    } else if BULL_TICKERS.contains(&ticker.as_str()) {
        Some(SentimentSide::Bull)
    } else {
        None
    }
}

fn sentiment_ratio(bull_dollars: f64, bear_dollars: f64) -> Option<f64> {
    if bear_dollars == 0.0 {
        None
    } else {
        Some(bull_dollars / bear_dollars)
    }
}

fn sentiment_signal(
    ratio: Option<f64>,
    bull_dollars: f64,
    bear_dollars: f64,
) -> TradeSentimentSignal {
    match ratio {
        None if bull_dollars > 0.0 => TradeSentimentSignal::ExtremeBull,
        None if bear_dollars > 0.0 => TradeSentimentSignal::ExtremeBear,
        None => TradeSentimentSignal::Neutral,
        Some(value) if value < 0.2 => TradeSentimentSignal::ExtremeBear,
        Some(value) if value < 0.5 => TradeSentimentSignal::ModerateBear,
        Some(value) if value <= 2.0 => TradeSentimentSignal::Neutral,
        Some(value) if value <= 5.0 => TradeSentimentSignal::ModerateBull,
        Some(_) => TradeSentimentSignal::ExtremeBull,
    }
}

fn top_sentiment_tickers(ticker_dollars: HashMap<String, f64>, limit: usize) -> Vec<String> {
    let mut totals: Vec<(String, f64)> = ticker_dollars.into_iter().collect();
    totals.sort_by(|(ticker_a, dollars_a), (ticker_b, dollars_b)| {
        dollars_b
            .total_cmp(dollars_a)
            .then_with(|| ticker_a.cmp(ticker_b))
    });
    totals
        .into_iter()
        .take(limit)
        .map(|(ticker, _)| ticker)
        .collect()
}

fn flatten_sentiment(sentiment: &TradeSentiment) -> Vec<SentimentRow> {
    let mut rows: Vec<SentimentRow> = sentiment.daily.iter().map(sentiment_day_row).collect();
    rows.push(sentiment_totals_row(&sentiment.totals));
    rows
}

fn sentiment_day_row(day: &TradeSentimentDay) -> SentimentRow {
    SentimentRow {
        date: day.date.clone(),
        bear_trades: day.bear.trades,
        bear_dollars: day.bear.dollars,
        bear_top_tickers: day.bear.top_tickers.join(";"),
        bull_trades: day.bull.trades,
        bull_dollars: day.bull.dollars,
        bull_top_tickers: day.bull.top_tickers.join(";"),
        ratio: day.ratio,
        signal: day.signal,
    }
}

fn sentiment_totals_row(totals: &TradeSentimentTotals) -> SentimentRow {
    SentimentRow {
        date: "total".to_string(),
        bear_trades: totals.bear.trades,
        bear_dollars: totals.bear.dollars,
        bear_top_tickers: totals.bear.top_tickers.join(";"),
        bull_trades: totals.bull.trades,
        bull_dollars: totals.bull.dollars,
        bull_top_tickers: totals.bull.top_tickers.join(";"),
        ratio: totals.ratio,
        signal: totals.signal,
    }
}

#[derive(Debug)]
struct TradePreset {
    name: &'static str,
    group: &'static str,
    base: PresetBase,
    filters: &'static [(&'static str, &'static str)],
}

#[derive(Clone, Copy, Debug)]
enum PresetBase {
    Common,
    Large,
    None,
}

static TRADE_PRESETS: &[TradePreset] = &[
    TradePreset {
        name: "All Trades",
        group: "Common",
        base: PresetBase::Common,
        filters: &[],
    },
    TradePreset {
        name: "Top-10 Rank",
        group: "Common",
        base: PresetBase::Common,
        filters: &[("TradeRank", "10")],
    },
    TradePreset {
        name: "Top-100 Rank",
        group: "Common",
        base: PresetBase::Common,
        filters: &[("MaxDollars", "100000000000"), ("TradeRank", "100")],
    },
    TradePreset {
        name: "Top-100 Rank; Dark Pool Sweeps",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("DarkPools", "1"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOffsetting", "-1"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "0"),
            ("MaxDollars", "100000000000"),
            ("MinVolume", "10000"),
            ("RelativeSize", "0"),
            ("SignaturePrints", "0"),
            ("Sweeps", "1"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-100 Rank; Leveraged ETFs",
        group: "Common",
        base: PresetBase::Common,
        filters: &[
            ("MaxDollars", "1000000000000"),
            ("SectorIndustry", "X B"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-100 Rank; RSI OB; >=5x Avg Size",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "OBD,OBH"),
            ("IncludeOffsetting", "-1"),
            ("IncludePhantom", "-1"),
            ("MaxDollars", "10000000000"),
            ("MinVolume", "10000"),
            ("SignaturePrints", "0"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-100 Rank; RSI OS; >=5x Avg Size",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "OSD,OSH"),
            ("IncludeOffsetting", "-1"),
            ("IncludePhantom", "-1"),
            ("MaxDollars", "10000000000"),
            ("MinVolume", "10000"),
            ("SignaturePrints", "0"),
            ("TradeCount", "3"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-100 Rank; >=20x avg size; DP Only",
        group: "Common",
        base: PresetBase::Common,
        filters: &[
            ("DarkPools", "1"),
            ("RelativeSize", "20"),
            ("SignaturePrints", "0"),
            ("TradeRank", "100"),
        ],
    },
    TradePreset {
        name: "Top-30 Rank; >10x avg size; 99th %",
        group: "Common",
        base: PresetBase::Common,
        filters: &[
            ("RelativeSize", "10"),
            ("SignaturePrints", "0"),
            ("TradeRank", "30"),
            ("VCD", "99.00"),
        ],
    },
    TradePreset {
        name: "Phantom Trades",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("DarkPools", "1"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOffsetting", "0"),
            ("IncludeOpening", "0"),
            ("IncludePremarket", "0"),
            ("IncludeRTH", "0"),
            ("MaxDollars", "100000000000"),
            ("RelativeSize", "0"),
            ("SignaturePrints", "0"),
            ("TradeCount", "3"),
        ],
    },
    TradePreset {
        name: "Offsetting Trades",
        group: "Common",
        base: PresetBase::None,
        filters: &[
            ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
            ("IncludeAH", "0"),
            ("IncludeClosing", "0"),
            ("IncludeOpening", "0"),
            ("IncludePhantom", "0"),
            ("IncludePremarket", "0"),
            ("IncludeRTH", "0"),
            ("MaxDollars", "100000000000"),
            ("RelativeSize", "0"),
            ("SignaturePrints", "0"),
            ("TradeCount", "3"),
        ],
    },
    TradePreset {
        name: "All Disproportionately Large Trades",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[],
    },
    TradePreset {
        name: "Bear Leverage",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "X Bear"), ("VCD", "97.00")],
    },
    TradePreset {
        name: "Biotechnology",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Biotech")],
    },
    TradePreset {
        name: "Bonds",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Bonds")],
    },
    TradePreset {
        name: "Bull Leverage",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "X Bull"), ("VCD", "97.00")],
    },
    TradePreset {
        name: "China",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "China"), ("MaxDollars", "100000000000")],
    },
    TradePreset {
        name: "Communication Services",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Comm Services")],
    },
    TradePreset {
        name: "Consumer Discretionary",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Consumer Disc")],
    },
    TradePreset {
        name: "Consumer Staples",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Consumer Staples")],
    },
    TradePreset {
        name: "Crypto",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Crypto"), ("VCD", "97.00")],
    },
    TradePreset {
        name: "Emerging Markets",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Emerging Markets")],
    },
    TradePreset {
        name: "Energy",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Energy")],
    },
    TradePreset {
        name: "Financials",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Financial")],
    },
    TradePreset {
        name: "Healthcare",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Healthcare")],
    },
    TradePreset {
        name: "Industrials",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Industrials")],
    },
    TradePreset {
        name: "Materials",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Materials")],
    },
    TradePreset {
        name: "Metals and Mining",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Metals and Mining")],
    },
    TradePreset {
        name: "Real Estate",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Real Estate")],
    },
    TradePreset {
        name: "Semiconductors",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Semis")],
    },
    TradePreset {
        name: "Technology",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Technology")],
    },
    TradePreset {
        name: "Utilities",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[("SectorIndustry", "Utilities")],
    },
    TradePreset {
        name: "Commodities",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "AGQ,BOIL,CORN,COPX,CPER,DBC,DJP,GLD,GLDM,IAU,KOLD,PPLT,SCO,SLV,SOYB,UCO,UGL,UNG,URA,USO,UUP,WEAT,ZSL",
            ),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "Electric Vehicles",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "BLNK,F,GM,LI,NIO,NKLA,TSLA,WKHS,QS,LCID,RIVN,TSLQ,TSLL,TSLS,TSLY,TSDD",
            ),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "Megacaps",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            ("Tickers", "AAPL,AMZN,META,GOOG,GOOGL,MSFT,NFLX,NVDA,TSLA"),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "Meme Stocks",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "AMC,BB,CLF,GME,NOK,SAVA,SPCE,TLRY,LOGC,CLOV,SOFI,BKKT,PUBM",
            ),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "Sector ETFs",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "DGRO,EEM,GLD,IBB,ITOT,IVE,IVW,IVV,IWM,IWY,MDY,QQQ,RSP,SLV,SMH,SPYD,SPY,SPYV,SPYG,TLT,USO,XBI,XLE,XLK,XLP,XLI,XLF,XLC,XLY,XLV,XLU",
            ),
            ("VCD", "97.00"),
        ],
    },
    TradePreset {
        name: "SPY/QQQ Surrogates",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            (
                "Tickers",
                "ACWI,DGRO,FBCG,FBCV,IWL,IWB,IVW,IVV,IWF,IWX,IWV,IWY,MGC,MGK,MGV,MTUM,OEF,PSQ,QLD,QID,QQQE,QQQ,QQEW,RSP,SCHG,SCHK,SCHV,SCHX,SDS,SH,SPYM,SPXS,SPXL,SPYD,SPY,SQQQ,SPYV,SPXU,SPYG,SSO,SUSA,TCHP,TQQQ,UDOW,UPRO,VFVA,VOO,VOOG,VOOV,VUG,VV,VTV,XLK,CGGR,JGRO,SPYU",
            ),
            ("MaxDollars", "100000000000"),
            ("RelativeSize", "0"),
        ],
    },
    TradePreset {
        name: "Volatility",
        group: "Disproportionately Large",
        base: PresetBase::Large,
        filters: &[
            ("Tickers", "SVXY,UVXY,VIXY,VXX,SVIX,UVIX"),
            ("VCD", "97.00"),
        ],
    },
];

fn find_trade_preset(name: &str) -> Option<&'static TradePreset> {
    TRADE_PRESETS
        .iter()
        .find(|preset| preset.name.eq_ignore_ascii_case(name))
}

fn apply_preset_filters(filters: &mut Vec<(String, String)>, preset: &TradePreset) {
    let _ = preset.group;
    match preset.base {
        PresetBase::Common => apply_common_preset_filters(filters),
        PresetBase::Large => apply_large_preset_filters(filters),
        PresetBase::None => {}
    }
    for &(key, value) in preset.filters {
        set_filter(filters, key, value.to_string());
    }
}

fn apply_common_preset_filters(filters: &mut Vec<(String, String)>) {
    for (key, value) in [
        ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
        ("IncludeOffsetting", "-1"),
        ("IncludePhantom", "-1"),
        ("MaxDollars", "10000000000"),
        ("MinVolume", "10000"),
        ("RelativeSize", "0"),
        ("TradeCount", "3"),
    ] {
        set_filter(filters, key, value.to_string());
    }
}

fn apply_large_preset_filters(filters: &mut Vec<(String, String)>) {
    for (key, value) in [
        ("Conditions", "IgnoreOBD,IgnoreOBH,IgnoreOSD,IgnoreOSH"),
        ("IncludeOffsetting", "-1"),
        ("IncludePhantom", "-1"),
        ("MaxDollars", "10000000000"),
        ("MinVolume", "10000"),
        ("TradeCount", "3"),
    ] {
        set_filter(filters, key, value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use volumeleaders_client::{Trade, TradeCluster, TradeClusterBomb, TradeLevel};

    use super::*;

    fn trade(value: serde_json::Value) -> Trade {
        serde_json::from_value(value).unwrap()
    }

    fn cluster(value: serde_json::Value) -> TradeCluster {
        serde_json::from_value(value).unwrap()
    }

    fn level(value: serde_json::Value) -> TradeLevel {
        serde_json::from_value(value).unwrap()
    }

    fn cluster_bomb(value: serde_json::Value) -> TradeClusterBomb {
        serde_json::from_value(value).unwrap()
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
        assert_eq!(level["RelativeSize"], 0.0);
        assert!(!level.contains_key("Ticker"));

        let bomb = output["cluster_bombs"][0].as_object().unwrap();
        assert_eq!(bomb["TradeClusterBombRank"], 1);
        assert!(!bomb.contains_key("ExternalFeed"));
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
