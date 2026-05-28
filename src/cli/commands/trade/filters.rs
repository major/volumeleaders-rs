use crate::{
    DataTablesColumn, TradeClusterBombsRequest, TradeClustersRequest, TradeLevelsRequest,
    TradesRequest,
};

use crate::cli::common::types::TriStateFilter;

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

pub(super) fn default_trade_list_filters() -> Vec<(String, String)> {
    vec![
        pair("MinVolume", HAR_TRADE_MIN_VOLUME.to_string()),
        pair("MaxVolume", DEFAULT_MAX_VOLUME.to_string()),
        pair("MinPrice", "0"),
        pair("MaxPrice", format_float(DEFAULT_MAX_PRICE)),
        pair("MinDollars", "500000"),
        pair("MaxDollars", format_float(HAR_TRADE_MAX_DOLLARS)),
        pair("Conditions", "0"),
        pair("VCD", "0"),
        pair("SecurityTypeKey", "-1"),
        pair("RelativeSize", "0"),
        pair("DarkPools", "-1"),
        pair("Sweeps", "-1"),
        pair("LatePrints", "-1"),
        pair("SignaturePrints", "-1"),
        pair("EvenShared", "-1"),
        pair("TradeRank", "100"),
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

pub(super) fn apply_trade_list_ranges(filters: &mut Vec<(String, String)>, args: &TradeRangeArgs) {
    set_filter(
        filters,
        "MinVolume",
        args.min_volume.unwrap_or(HAR_TRADE_MIN_VOLUME).to_string(),
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
        format_float(args.min_dollars.unwrap_or(500_000.0)),
    );
    set_filter(
        filters,
        "MaxDollars",
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

pub(super) fn apply_trade_filter_args(filters: &mut Vec<(String, String)>, args: &TradeFilterArgs) {
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
        .with_order(3, "desc", "Sh")
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
        .with_order(0, "desc", "Price")
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
        "MinVolume",
        args.ranges.min_volume.unwrap_or(10_000).to_string(),
    );
    filters.push(pair("VCD", args.vcd.unwrap_or(0).to_string()));
    filters.push(pair(
        "SecurityTypeKey",
        args.security_type.unwrap_or(-1).to_string(),
    ));
    filters.push(pair(
        "RelativeSize",
        args.relative_size.unwrap_or(0).to_string(),
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

pub(super) fn cluster_bomb_filters(
    args: &ClusterBombsArgs,
    start: &str,
    end: &str,
) -> Vec<(String, String)> {
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
            args.security_type.unwrap_or(0).to_string(),
        ),
        pair("RelativeSize", args.relative_size.unwrap_or(0).to_string()),
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

pub(super) fn level_touch_filters(
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

pub(super) fn validate_trade_level_count(count: usize) -> bool {
    matches!(count, 5 | 10 | 20 | 50)
}
