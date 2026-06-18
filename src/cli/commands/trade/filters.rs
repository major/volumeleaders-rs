use crate::datatables::SortDir;
use crate::{
    DataTablesColumn, TradeClusterBombsRequest, TradeClustersRequest, TradeLevelsRequest,
    TradesRequest,
};

use crate::cli::common::types::TriStateFilter;

// Filter key constants — compile-checked names for the VolumeLeaders API.
// ponytail: string keys kept as &str constants rather than a typed enum since
// API filter values are already stringly-typed; the win is catching typos
// at compile time.
pub(super) const K_CONDITIONS: &str = "Conditions";
pub(super) const K_DARK_POOLS: &str = "DarkPools";
pub(super) const K_END_DATE: &str = "EndDate";
pub(super) const K_EVEN_SHARED: &str = "EvenShared";
pub(super) const K_INCLUDE_AH: &str = "IncludeAH";
pub(super) const K_INCLUDE_CLOSING: &str = "IncludeClosing";
pub(super) const K_INCLUDE_OFFSETTING: &str = "IncludeOffsetting";
pub(super) const K_INCLUDE_OPENING: &str = "IncludeOpening";
pub(super) const K_INCLUDE_PHANTOM: &str = "IncludePhantom";
pub(super) const K_INCLUDE_PREMARKET: &str = "IncludePremarket";
pub(super) const K_INCLUDE_RTH: &str = "IncludeRTH";
pub(super) const K_LATE_PRINTS: &str = "LatePrints";
pub(super) const K_LEVELS: &str = "Levels";
pub(super) const K_MARKET_CAP: &str = "MarketCap";
pub(super) const K_MAX_DOLLARS: &str = "MaxDollars";
pub(super) const K_MAX_PRICE: &str = "MaxPrice";
pub(super) const K_MAX_VOLUME: &str = "MaxVolume";
pub(super) const K_MIN_DOLLARS: &str = "MinDollars";
pub(super) const K_MIN_PRICE: &str = "MinPrice";
pub(super) const K_MIN_VOLUME: &str = "MinVolume";
pub(super) const K_RELATIVE_SIZE: &str = "RelativeSize";
pub(super) const K_SECTOR_INDUSTRY: &str = "SectorIndustry";
pub(super) const K_SECURITY_TYPE_KEY: &str = "SecurityTypeKey";
pub(super) const K_SIGNATURE_PRINTS: &str = "SignaturePrints";
pub(super) const K_SORT: &str = "Sort";
pub(super) const K_START_DATE: &str = "StartDate";
pub(super) const K_SWEEPS: &str = "Sweeps";
pub(super) const K_TICKERS: &str = "Tickers";
pub(super) const K_TRADE_CLUSTER_BOMB_RANK: &str = "TradeClusterBombRank";
pub(super) const K_TRADE_CLUSTER_RANK: &str = "TradeClusterRank";
pub(super) const K_TRADE_LEVEL_RANK: &str = "TradeLevelRank";
pub(super) const K_TRADE_RANK: &str = "TradeRank";
pub(super) const K_TRADE_RANK_SNAPSHOT: &str = "TradeRankSnapshot";
pub(super) const K_VCD: &str = "VCD";

use super::{
    ClusterBombsArgs, ClustersArgs, DEFAULT_MAX_DOLLARS, DEFAULT_MAX_PRICE, DEFAULT_MAX_VOLUME,
    DashboardArgs, DashboardFilterArgs, HAR_TRADE_MAX_DOLLARS, HAR_TRADE_MIN_VOLUME,
    LevelTouchesArgs, TradeFilterArgs, TradeRangeArgs, parse_ticker_args,
};

pub(super) fn parse_tri_state_filter(value: &str) -> Result<TriStateFilter, String> {
    match value.to_ascii_lowercase().as_str() {
        "all" | "-1" => Ok(TriStateFilter::All),
        "only" | "enabled" | "1" => Ok(TriStateFilter::Enabled),
        "disabled" | "0" => Ok(TriStateFilter::Disabled),
        _ => Err("expected all, only, disabled, -1, 1, or 0".to_string()),
    }
}

pub(super) fn default_trade_filters(min_dollars: f64, vcd: i32) -> Vec<(String, String)> {
    vec![
        pair(K_MIN_VOLUME, "0"),
        pair(K_MAX_VOLUME, DEFAULT_MAX_VOLUME.to_string()),
        pair(K_MIN_PRICE, "0"),
        pair(K_MAX_PRICE, format_float(DEFAULT_MAX_PRICE)),
        pair(K_MIN_DOLLARS, format_float(min_dollars)),
        pair(K_MAX_DOLLARS, format_float(DEFAULT_MAX_DOLLARS)),
        pair(K_CONDITIONS, "-1"),
        pair(K_VCD, vcd.to_string()),
        pair(K_SECURITY_TYPE_KEY, "-1"),
        pair(K_RELATIVE_SIZE, "5"),
        pair(K_DARK_POOLS, "-1"),
        pair(K_SWEEPS, "-1"),
        pair(K_LATE_PRINTS, "-1"),
        pair(K_SIGNATURE_PRINTS, "-1"),
        pair(K_EVEN_SHARED, "-1"),
        pair(K_TRADE_RANK, "-1"),
        pair(K_TRADE_RANK_SNAPSHOT, "-1"),
        pair(K_MARKET_CAP, "0"),
        pair(K_INCLUDE_PREMARKET, "1"),
        pair(K_INCLUDE_RTH, "1"),
        pair(K_INCLUDE_AH, "1"),
        pair(K_INCLUDE_OPENING, "1"),
        pair(K_INCLUDE_CLOSING, "1"),
        pair(K_INCLUDE_PHANTOM, "1"),
        pair(K_INCLUDE_OFFSETTING, "1"),
    ]
}

pub(super) fn default_trade_list_filters() -> Vec<(String, String)> {
    vec![
        pair(K_MIN_VOLUME, HAR_TRADE_MIN_VOLUME.to_string()),
        pair(K_MAX_VOLUME, DEFAULT_MAX_VOLUME.to_string()),
        pair(K_MIN_PRICE, "0"),
        pair(K_MAX_PRICE, format_float(DEFAULT_MAX_PRICE)),
        pair(K_MIN_DOLLARS, "500000"),
        pair(K_MAX_DOLLARS, format_float(HAR_TRADE_MAX_DOLLARS)),
        pair(K_CONDITIONS, "0"),
        pair(K_VCD, "0"),
        pair(K_SECURITY_TYPE_KEY, "-1"),
        pair(K_RELATIVE_SIZE, "0"),
        pair(K_DARK_POOLS, "-1"),
        pair(K_SWEEPS, "-1"),
        pair(K_LATE_PRINTS, "-1"),
        pair(K_SIGNATURE_PRINTS, "-1"),
        pair(K_EVEN_SHARED, "-1"),
        pair(K_TRADE_RANK, "100"),
        pair(K_TRADE_RANK_SNAPSHOT, "-1"),
        pair(K_MARKET_CAP, "0"),
        pair(K_INCLUDE_PREMARKET, "1"),
        pair(K_INCLUDE_RTH, "1"),
        pair(K_INCLUDE_AH, "1"),
        pair(K_INCLUDE_OPENING, "1"),
        pair(K_INCLUDE_CLOSING, "1"),
        pair(K_INCLUDE_PHANTOM, "1"),
        pair(K_INCLUDE_OFFSETTING, "1"),
    ]
}

pub(super) fn apply_trade_list_ranges(filters: &mut Vec<(String, String)>, args: &TradeRangeArgs) {
    set_filter(
        filters,
        K_MIN_VOLUME,
        args.min_volume.unwrap_or(HAR_TRADE_MIN_VOLUME).to_string(),
    );
    set_filter(
        filters,
        K_MAX_VOLUME,
        args.max_volume.unwrap_or(DEFAULT_MAX_VOLUME).to_string(),
    );
    set_filter(
        filters,
        K_MIN_PRICE,
        format_float(args.min_price.unwrap_or(0.0)),
    );
    set_filter(
        filters,
        K_MAX_PRICE,
        format_float(args.max_price.unwrap_or(DEFAULT_MAX_PRICE)),
    );
    set_filter(
        filters,
        K_MIN_DOLLARS,
        format_float(args.min_dollars.unwrap_or(500_000.0)),
    );
    set_filter(
        filters,
        K_MAX_DOLLARS,
        format_float(args.max_dollars.unwrap_or(HAR_TRADE_MAX_DOLLARS)),
    );
}

pub(super) fn apply_trade_ranges(
    filters: &mut Vec<(String, String)>,
    args: &TradeRangeArgs,
    default_min: f64,
) {
    set_filter(
        filters,
        K_MIN_VOLUME,
        args.min_volume.unwrap_or(0).to_string(),
    );
    set_filter(
        filters,
        K_MAX_VOLUME,
        args.max_volume.unwrap_or(DEFAULT_MAX_VOLUME).to_string(),
    );
    set_filter(
        filters,
        K_MIN_PRICE,
        format_float(args.min_price.unwrap_or(0.0)),
    );
    set_filter(
        filters,
        K_MAX_PRICE,
        format_float(args.max_price.unwrap_or(DEFAULT_MAX_PRICE)),
    );
    set_filter(
        filters,
        K_MIN_DOLLARS,
        format_float(args.min_dollars.unwrap_or(default_min)),
    );
    set_filter(
        filters,
        K_MAX_DOLLARS,
        format_float(args.max_dollars.unwrap_or(DEFAULT_MAX_DOLLARS)),
    );
}

pub(super) fn apply_trade_filter_args(filters: &mut Vec<(String, String)>, args: &TradeFilterArgs) {
    if let Some(value) = &args.conditions {
        set_filter(filters, K_CONDITIONS, value.clone());
    }
    if let Some(value) = args.vcd {
        set_filter(filters, K_VCD, value.to_string());
    }
    if let Some(value) = args.security_type {
        set_filter(filters, K_SECURITY_TYPE_KEY, value.to_string());
    }
    if let Some(value) = args.relative_size {
        set_filter(filters, K_RELATIVE_SIZE, value.to_string());
    }
    apply_tri_state(filters, K_DARK_POOLS, args.dark_pools);
    apply_tri_state(filters, K_SWEEPS, args.sweeps);
    apply_tri_state(filters, K_LATE_PRINTS, args.late_prints);
    apply_tri_state(filters, K_SIGNATURE_PRINTS, args.sig_prints);
    apply_tri_state(filters, K_EVEN_SHARED, args.even_shared);
    if let Some(value) = args.trade_rank {
        set_filter(filters, K_TRADE_RANK, value.to_string());
    }
    if let Some(value) = args.rank_snapshot {
        set_filter(filters, K_TRADE_RANK_SNAPSHOT, value.to_string());
    }
    if let Some(value) = args.market_cap {
        set_filter(filters, K_MARKET_CAP, value.to_string());
    }
    apply_tri_state(filters, K_INCLUDE_PREMARKET, args.premarket);
    apply_tri_state(filters, K_INCLUDE_RTH, args.rth);
    apply_tri_state(filters, K_INCLUDE_AH, args.ah);
    apply_tri_state(filters, K_INCLUDE_OPENING, args.opening);
    apply_tri_state(filters, K_INCLUDE_CLOSING, args.closing);
    apply_tri_state(filters, K_INCLUDE_PHANTOM, args.phantom);
    apply_tri_state(filters, K_INCLUDE_OFFSETTING, args.offsetting);
    if let Some(value) = &args.sector {
        set_filter(filters, K_SECTOR_INDUSTRY, value.clone());
    }
}

fn apply_dashboard_filter_args(filters: &mut Vec<(String, String)>, args: &DashboardFilterArgs) {
    if let Some(value) = &args.conditions {
        set_filter(filters, K_CONDITIONS, value.clone());
    }
    if let Some(value) = args.vcd {
        set_filter(filters, K_VCD, value.to_string());
    }
    if let Some(value) = args.relative_size {
        set_filter(filters, K_RELATIVE_SIZE, value.to_string());
    }
    apply_tri_state(filters, K_DARK_POOLS, args.dark_pools);
    apply_tri_state(filters, K_SWEEPS, args.sweeps);
    apply_tri_state(filters, K_LATE_PRINTS, args.late_prints);
    apply_tri_state(filters, K_SIGNATURE_PRINTS, args.sig_prints);
    if let Some(value) = args.trade_rank {
        set_filter(filters, K_TRADE_RANK, value.to_string());
    }
    apply_tri_state(filters, K_INCLUDE_PREMARKET, args.premarket);
    apply_tri_state(filters, K_INCLUDE_RTH, args.rth);
    apply_tri_state(filters, K_INCLUDE_AH, args.ah);
    apply_tri_state(filters, K_INCLUDE_OPENING, args.opening);
    apply_tri_state(filters, K_INCLUDE_CLOSING, args.closing);
    apply_tri_state(filters, K_INCLUDE_PHANTOM, args.phantom);
    apply_tri_state(filters, K_INCLUDE_OFFSETTING, args.offsetting);
    if let Some(value) = &args.sector {
        set_filter(filters, K_SECTOR_INDUSTRY, value.clone());
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

pub(super) fn set_ticker_filters(
    filters: &mut Vec<(String, String)>,
    tickers: &[String],
    key: &'static str,
) {
    filters.retain(|(existing_key, _)| existing_key != key);
    for ticker in tickers {
        filters.push((key.to_string(), ticker.clone()));
    }
}

pub(super) fn set_filter(filters: &mut Vec<(String, String)>, key: &'static str, value: String) {
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

pub(super) fn dashboard_trades_request(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> TradesRequest {
    let mut filters = default_trade_filters(args.ranges.min_dollars.unwrap_or(0.0), 0);
    apply_trade_ranges(&mut filters, &args.ranges, 0.0);
    apply_dashboard_filter_args(&mut filters, &args.filters);
    set_filter(&mut filters, K_TICKERS, ticker.to_string());
    set_filter(&mut filters, K_START_DATE, start.to_string());
    set_filter(&mut filters, K_END_DATE, end.to_string());
    set_filter(&mut filters, K_SORT, "Dollars".to_string());
    remove_filters(
        &mut filters,
        &[
            K_SECURITY_TYPE_KEY,
            K_EVEN_SHARED,
            K_TRADE_RANK_SNAPSHOT,
            K_MARKET_CAP,
        ],
    );
    TradesRequest::new()
        .with_columns(trade_chart_columns())
        .with_length(args.count as i32)
        .with_search("", false)
        .with_order(0, SortDir::Desc, "FullTimeString24")
        .with_trade_filters(filters)
}

pub(super) fn dashboard_clusters_request(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> TradeClustersRequest {
    TradeClustersRequest::new()
        .with_columns(trade_cluster_chart_columns())
        .with_length(args.count as i32)
        .with_search("", false)
        .with_order(3, SortDir::Desc, "Sh")
        .with_cluster_filters(dashboard_cluster_filters(args, ticker, start, end))
}

pub(super) fn dashboard_levels_request(
    ticker: &str,
    start: &str,
    end: &str,
    count: usize,
) -> TradeLevelsRequest {
    let levels = super::nearest_level_count(count);
    TradeLevelsRequest::new()
        .with_columns(trade_level_chart_columns())
        .with_length(count as i32)
        .with_search("", false)
        .with_order(0, SortDir::Desc, "Price")
        .with_chart_filters(ticker, start, end, levels)
}

pub(super) fn dashboard_bombs_request(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> TradeClusterBombsRequest {
    TradeClusterBombsRequest::new()
        .with_columns(trade_cluster_bomb_chart_columns())
        .with_length(args.count as i32)
        .with_search("", false)
        .with_order(2, SortDir::Desc, "Sh")
        .with_cluster_bomb_filters(dashboard_bomb_filters(args, ticker, start, end))
}

fn dashboard_cluster_filters(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> Vec<(String, String)> {
    vec![
        pair(K_TICKERS, ticker.to_string()),
        pair(K_START_DATE, start.to_string()),
        pair(K_END_DATE, end.to_string()),
        pair(
            K_MIN_VOLUME,
            args.ranges.min_volume.unwrap_or(0).to_string(),
        ),
        pair(
            K_MAX_VOLUME,
            args.ranges
                .max_volume
                .unwrap_or(DEFAULT_MAX_VOLUME)
                .to_string(),
        ),
        pair(
            K_MIN_PRICE,
            format_float(args.ranges.min_price.unwrap_or(0.0)),
        ),
        pair(
            K_MAX_PRICE,
            format_float(args.ranges.max_price.unwrap_or(DEFAULT_MAX_PRICE)),
        ),
        pair(
            K_MIN_DOLLARS,
            format_float(args.ranges.min_dollars.unwrap_or(500_000.0)),
        ),
        pair(
            K_MAX_DOLLARS,
            format_float(args.ranges.max_dollars.unwrap_or(DEFAULT_MAX_DOLLARS)),
        ),
        pair(K_VCD, args.filters.vcd.unwrap_or(0).to_string()),
        pair(
            K_RELATIVE_SIZE,
            args.filters.relative_size.unwrap_or(0).to_string(),
        ),
        pair(K_TRADE_CLUSTER_RANK, "-1"),
        pair(K_SORT, "Dollars"),
    ]
}

fn dashboard_bomb_filters(
    args: &DashboardArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> Vec<(String, String)> {
    let mut filters = dashboard_cluster_filters(args, ticker, start, end);
    remove_filters(
        &mut filters,
        &[K_MIN_PRICE, K_MAX_PRICE, K_TRADE_CLUSTER_RANK],
    );
    filters.push(pair(K_TRADE_CLUSTER_BOMB_RANK, "-1"));
    filters
}

fn remove_filters(filters: &mut Vec<(String, String)>, names: &[&str]) {
    filters.retain(|(key, _)| !names.contains(&key.as_str()));
}

fn trade_chart_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::id("FullTimeString24"),
        DataTablesColumn::searchable("Volume", "Sh"),
        DataTablesColumn::id("Price"),
        DataTablesColumn::searchable("Dollars", "$$"),
        DataTablesColumn::searchable("DollarsMultiplier", "RS"),
        DataTablesColumn::searchable(K_TRADE_RANK, "R"),
        DataTablesColumn::searchable("LastComparibleTradeDate", "Last Comp"),
    ]
}

fn trade_cluster_chart_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::id("MinFullTimeString24"),
        DataTablesColumn::id("Price"),
        DataTablesColumn::id("TradeCount"),
        DataTablesColumn::searchable("Volume", "Sh"),
        DataTablesColumn::searchable("Dollars", "$$"),
        DataTablesColumn::searchable("DollarsMultiplier", "RS"),
        DataTablesColumn::searchable(K_TRADE_CLUSTER_RANK, "R"),
        DataTablesColumn::searchable("LastComparibleTradeClusterDate", "Last Comp"),
    ]
}

fn trade_level_chart_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::id("Price"),
        DataTablesColumn::searchable("Dollars", "$$"),
        DataTablesColumn::searchable("Volume", "Sh"),
        DataTablesColumn::id("Trades"),
        DataTablesColumn::searchable(K_RELATIVE_SIZE, "RS"),
        DataTablesColumn::searchable("CumulativeDistribution", "PCT"),
        DataTablesColumn::searchable(K_TRADE_LEVEL_RANK, "Rank"),
        DataTablesColumn::id("Dates"),
    ]
}

fn trade_cluster_bomb_chart_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::id("MinFullTimeString24"),
        DataTablesColumn::id("TradeCount"),
        DataTablesColumn::searchable("Volume", "Sh"),
        DataTablesColumn::searchable("Dollars", "$$"),
        DataTablesColumn::searchable("DollarsMultiplier", "RS"),
        DataTablesColumn::searchable("CumulativeDistribution", "PCT"),
        DataTablesColumn::searchable(K_TRADE_CLUSTER_BOMB_RANK, "R"),
        DataTablesColumn::searchable("LastComparableTradeClusterBombDate", "Last Comp"),
    ]
}

pub(super) fn cluster_filters(
    args: &ClustersArgs,
    start: &str,
    end: &str,
) -> Vec<(String, String)> {
    let tickers = parse_ticker_args(&args.tickers).join(",");
    let mut filters = range_base_filters(
        &tickers,
        start,
        end,
        &args.ranges,
        args.ranges.min_dollars.unwrap_or(500_000.0),
    );
    set_filter(
        &mut filters,
        K_MIN_VOLUME,
        args.ranges.min_volume.unwrap_or(10_000).to_string(),
    );
    filters.push(pair(K_VCD, args.vcd.unwrap_or(0).to_string()));
    filters.push(pair(
        K_SECURITY_TYPE_KEY,
        args.security_type.unwrap_or(-1).to_string(),
    ));
    filters.push(pair(
        K_RELATIVE_SIZE,
        args.relative_size.unwrap_or(0).to_string(),
    ));
    filters.push(pair(
        K_TRADE_CLUSTER_RANK,
        args.trade_cluster_rank.to_string(),
    ));
    if let Some(sector) = &args.sector {
        filters.push(pair(K_SECTOR_INDUSTRY, sector.clone()));
    }
    filters
}

pub(super) fn cluster_bomb_filters(
    args: &ClusterBombsArgs,
    start: &str,
    end: &str,
) -> Vec<(String, String)> {
    let tickers = parse_ticker_args(&args.tickers).join(",");
    let mut filters = vec![
        pair(K_TICKERS, tickers),
        pair(K_START_DATE, start.to_string()),
        pair(K_END_DATE, end.to_string()),
        pair(
            K_MIN_VOLUME,
            args.ranges.min_volume.unwrap_or(0).to_string(),
        ),
        pair(
            K_MAX_VOLUME,
            args.ranges
                .max_volume
                .unwrap_or(DEFAULT_MAX_VOLUME)
                .to_string(),
        ),
        pair(
            K_MIN_DOLLARS,
            format_float(args.ranges.min_dollars.unwrap_or(0.0)),
        ),
        pair(
            K_MAX_DOLLARS,
            format_float(args.ranges.max_dollars.unwrap_or(DEFAULT_MAX_DOLLARS)),
        ),
        pair(K_VCD, args.vcd.unwrap_or(0).to_string()),
        pair(
            K_SECURITY_TYPE_KEY,
            args.security_type.unwrap_or(0).to_string(),
        ),
        pair(K_RELATIVE_SIZE, args.relative_size.unwrap_or(0).to_string()),
        pair(
            K_TRADE_CLUSTER_BOMB_RANK,
            args.trade_cluster_bomb_rank.to_string(),
        ),
    ];
    if let Some(sector) = &args.sector {
        filters.push(pair(K_SECTOR_INDUSTRY, sector.clone()));
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
        pair(K_TICKERS, tickers.to_string()),
        pair(K_START_DATE, start.to_string()),
        pair(K_END_DATE, end.to_string()),
        pair(K_MIN_VOLUME, ranges.min_volume.unwrap_or(0).to_string()),
        pair(
            K_MAX_VOLUME,
            ranges.max_volume.unwrap_or(DEFAULT_MAX_VOLUME).to_string(),
        ),
        pair(K_MIN_PRICE, format_float(ranges.min_price.unwrap_or(0.0))),
        pair(
            K_MAX_PRICE,
            format_float(ranges.max_price.unwrap_or(DEFAULT_MAX_PRICE)),
        ),
        pair(
            K_MIN_DOLLARS,
            format_float(ranges.min_dollars.unwrap_or(default_min_dollars)),
        ),
        pair(
            K_MAX_DOLLARS,
            format_float(ranges.max_dollars.unwrap_or(DEFAULT_MAX_DOLLARS)),
        ),
    ]
}

pub(super) fn level_touch_filters(
    args: &LevelTouchesArgs,
    ticker: &str,
    start: &str,
    end: &str,
) -> Vec<(String, String)> {
    let mut filters = range_base_filters(ticker, start, end, &args.ranges, 500_000.0);
    filters.push(pair(K_VCD, args.vcd.unwrap_or(0).to_string()));
    filters.push(pair(
        K_RELATIVE_SIZE,
        args.relative_size.unwrap_or(5).to_string(),
    ));
    filters.push(pair(K_TRADE_LEVEL_RANK, args.trade_level_rank.to_string()));
    filters.push(pair(K_LEVELS, args.trade_level_count.to_string()));
    filters
}

pub(super) fn validate_trade_level_count(count: usize) -> bool {
    matches!(count, 5 | 10 | 20 | 50)
}
