use std::collections::BTreeSet;

use crate::{Trade, TradeCluster, TradeClusterAlert, TradeClusterBomb, TradeLevel};
use serde_json::json;

use crate::cli::field_metadata;
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
        "MinFullTimeString24": "16:00:00",
        "LastComparibleTradeClusterDate": "/Date(1767225600000)/",
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
        "MinFullTimeString24": "15:00:00",
        "Price": 312.25,
        "DollarsMultiplier": 3.5,
        "LastComparibleTradeClusterDate": "/Date(1767225600000)/",
        "MinFullDateTime": "2026-01-02T15:00:00+00:00",
        "MaxFullDateTime": "2026-01-02T15:07:30+00:00",
        "VOLEX": true
    }))
}

fn render_cluster_json(fields: Option<&str>, all_fields: bool) -> serde_json::Value {
    let values = vec![serde_json::to_value(cluster_fixture()).expect("cluster serializes")];
    let mut output = Vec::new();
    write_record_values(
        &mut output,
        &values,
        CLUSTER_HEADERS,
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
            "DollarsMultiplier": 4.2,
            "LastComparibleTradeDate": "/Date(1767225600000)/",
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
            "MinFullTimeString24": "16:00:00",
            "DollarsMultiplier": 3.1,
            "LastComparibleTradeClusterDate": "/Date(1767225600000)/",
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
            "CumulativeDistribution": 99.1,
            "TradeLevelRank": 1,
            "Dates": "2026-01-01,2026-01-02",
            "Name": null
        }))],
        cluster_bombs: vec![cluster_bomb(json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Dollars": 40_000_000.0,
            "Volume": 200_000,
            "TradeCount": 6,
            "TradeClusterBombRank": 1,
            "MinFullTimeString24": "16:00:00",
            "DollarsMultiplier": 2.4,
            "CumulativeDistribution": 98.5,
            "LastComparableTradeClusterBombDate": "/Date(1767225600000)/",
            "ExternalFeed": false
        }))],
    }
}

#[test]
fn trade_shaped_default_headers_are_raw_website_fields() {
    assert_eq!(
        TRADE_HEADERS,
        [
            "FullTimeString24",
            "Volume",
            "Price",
            "Dollars",
            "DollarsMultiplier",
            "TradeRank",
            "LastComparibleTradeDate",
        ]
    );
    assert_eq!(
        CLUSTER_HEADERS,
        [
            "MinFullTimeString24",
            "TradeCount",
            "Price",
            "Dollars",
            "DollarsMultiplier",
            "TradeClusterRank",
            "LastComparibleTradeClusterDate",
        ]
    );
    assert_eq!(
        BOMB_HEADERS,
        [
            "MinFullTimeString24",
            "TradeCount",
            "Volume",
            "Dollars",
            "DollarsMultiplier",
            "CumulativeDistribution",
            "TradeClusterBombRank",
            "LastComparableTradeClusterBombDate",
        ]
    );
    assert_eq!(
        LEVEL_HEADERS,
        [
            "Price",
            "Dollars",
            "Volume",
            "Trades",
            "RelativeSize",
            "CumulativeDistribution",
            "TradeLevelRank",
            "Dates",
        ]
    );
    assert_eq!(
        LEVEL_TOUCHES_HEADERS,
        [
            "Ticker",
            "FullTimeString24",
            "Price",
            "Dollars",
            "Volume",
            "Trades",
            "RelativeSize",
            "TradeLevelRank",
            "Dates",
        ]
    );
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
fn dashboard_output_defaults_to_raw_compact_fields() {
    let args = dashboard_args();
    let output = dashboard_output_value(&dashboard_fixture(), &args).unwrap();

    assert_eq!(output["ticker"], "AAPL");
    assert_eq!(output["date_range"]["start"], "2026-01-01");
    assert_eq!(output["count"], DEFAULT_DASHBOARD_COUNT);

    let trade = output["trades"][0].as_object().unwrap();
    assert_eq!(trade["FullTimeString24"], "16:00:00");
    assert_eq!(trade["Volume"], 50_000);
    assert_eq!(trade["DollarsMultiplier"], 4.2);
    assert_eq!(trade["TradeRank"], 3);
    assert!(!trade.contains_key("Time"));
    assert!(!trade.contains_key("venue"));
    assert!(!trade.contains_key("type"));
    assert!(!trade.contains_key("Ticker"));

    let cluster = output["clusters"][0].as_object().unwrap();
    assert_eq!(cluster["MinFullTimeString24"], "16:00:00");
    assert_eq!(cluster["TradeCount"], 4);
    assert_eq!(cluster["TradeClusterRank"], 2);
    assert_eq!(cluster["DollarsMultiplier"], 3.1);
    assert!(!cluster.contains_key("window"));
    assert!(!cluster.contains_key("Ticker"));

    let level = output["levels"][0].as_object().unwrap();
    assert_eq!(level["Volume"], 150_000);
    assert_eq!(level["RelativeSize"], 0.0);
    assert_eq!(level["TradeLevelRank"], 1);
    assert_eq!(level["Dates"], "2026-01-01,2026-01-02");
    assert!(!level.contains_key("Ticker"));

    let bomb = output["cluster_bombs"][0].as_object().unwrap();
    assert_eq!(bomb["MinFullTimeString24"], "16:00:00");
    assert_eq!(bomb["TradeClusterBombRank"], 1);
    assert_eq!(
        bomb["LastComparableTradeClusterBombDate"],
        "2026-01-01T00:00:00+00:00"
    );
    assert!(!bomb.contains_key("ExternalFeed"));
}

#[test]
fn dashboard_output_marks_empty_sections() {
    let args = dashboard_args();
    let mut dashboard = dashboard_fixture();
    dashboard.trades.clear();

    let output = dashboard_output_value(&dashboard, &args).unwrap();

    assert_eq!(output["trades"], json!([]));
    assert_eq!(output["sections"]["trades"]["count"], 0);
    assert_eq!(output["sections"]["trades"]["empty"], true);
    assert_eq!(output["sections"]["clusters"]["count"], 1);
    assert_eq!(output["sections"]["clusters"]["empty"], false);
    assert_eq!(output["sections"]["levels"]["count"], 1);
    assert_eq!(output["sections"]["levels"]["empty"], false);
    assert_eq!(output["sections"]["cluster_bombs"]["count"], 1);
    assert_eq!(output["sections"]["cluster_bombs"]["empty"], false);
}

#[test]
fn dashboard_output_section_metadata_matches_projected_sections() {
    let mut args = dashboard_args();
    args.fields = Some("trades.Date".to_string());

    let output = dashboard_output_value(&dashboard_fixture(), &args).unwrap();

    assert!(output.get("clusters").is_none());
    assert!(output.get("levels").is_none());
    assert!(output.get("cluster_bombs").is_none());
    assert_eq!(output["sections"]["trades"]["count"], 1);
    assert_eq!(output["sections"]["trades"]["empty"], false);
    assert!(output["sections"].get("clusters").is_none());
    assert!(output["sections"].get("levels").is_none());
    assert!(output["sections"].get("cluster_bombs").is_none());
}

#[test]
fn cluster_output_defaults_to_raw_compact_fields() {
    let output = render_cluster_json(None, false);
    let row = output[0].as_object().unwrap();

    assert_eq!(row["MinFullTimeString24"], "16:00:00");
    assert_eq!(row["TradeCount"], 4);
    assert_eq!(row["Price"], 199.125);
    assert_eq!(row["Dollars"], 20_000_000.126);
    assert_eq!(row["DollarsMultiplier"], 12.345);
    assert_eq!(row["TradeClusterRank"], 2);
    assert_eq!(
        row["LastComparibleTradeClusterDate"],
        "2026-01-01T00:00:00+00:00"
    );
    assert!(!row.contains_key("window"));
    assert!(!row.contains_key("Ticker"));
    assert!(!row.contains_key("SecurityKey"));
}

#[test]
fn cluster_output_accepts_custom_raw_fields() {
    let output = render_cluster_json(
        Some("MinFullTimeString24,TradeCount,LastComparibleTradeClusterDate"),
        false,
    );
    let row = output[0].as_object().unwrap();

    assert_eq!(row.len(), 3);
    assert_eq!(row["MinFullTimeString24"], "16:00:00");
    assert_eq!(row["TradeCount"], 4);
    assert_eq!(
        row["LastComparibleTradeClusterDate"],
        "2026-01-01T00:00:00+00:00"
    );
}

#[test]
fn trade_output_accepts_discovered_metadata_field_absent_from_rows() {
    let records = vec![trade(json!({
        "Ticker": "AAPL",
        "Date": "/Date(1767312000000)/",
        "Price": 200.0,
        "Dollars": 10_000_000.0
    }))];

    print_trade_records(&records, TRADE_HEADERS, Some("Price"), false, "trade list")
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
        TRADE_HEADERS,
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

    assert!(
        output_trade_records(&records, TRADE_HEADERS, Some("Price"), false, "trade list",).is_ok()
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
        BOMB_HEADERS,
        Some("TradeClusterBombRank"),
        false,
        "trade cluster-bombs",
    )
    .expect("cluster-bomb metadata includes cluster-bomb rank");
}

#[test]
fn cluster_output_all_fields_keeps_raw_extra_fields() {
    let output = render_cluster_json(None, true);
    let row = output[0].as_object().unwrap();

    assert_eq!(row["SecurityKey"], 123);
    assert_eq!(row["MinFullDateTime"], "2026-01-02T16:00:00+00:00");
    assert_eq!(row["MaxFullDateTime"], "2026-01-02T16:49:31+00:00");
    assert_eq!(row["EOM"], true);
    assert_eq!(row["OPEX"], false);
    assert!(!row.contains_key("window"));
}

#[test]
fn cluster_alert_output_uses_raw_cluster_headers() {
    let values =
        vec![serde_json::to_value(cluster_alert_fixture()).expect("cluster alert serializes")];
    let mut output = Vec::new();
    write_record_values(&mut output, &values, CLUSTER_HEADERS, None, false, None)
        .expect("cluster alert output renders");
    let output: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let row = output[0].as_object().unwrap();

    assert!(
        CLUSTER_HEADERS
            .iter()
            .all(|header| row.contains_key(*header))
    );
    assert_eq!(row["MinFullTimeString24"], "15:00:00");
    assert_eq!(
        row["LastComparibleTradeClusterDate"],
        "2026-01-01T00:00:00+00:00"
    );
    assert!(!row.contains_key("window"));
    assert!(!row.contains_key("Ticker"));
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
    assert_eq!(trade["Date"], "2026-01-02T00:00:00+00:00");
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
    args.fields = Some("trades.FullTimeString24".to_string());
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
    assert_eq!(trade["FullTimeString24"], serde_json::Value::Null);
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
fn dashboard_output_all_fields_returns_raw_rows() {
    let mut args = dashboard_args();
    args.all_fields = true;

    let output = dashboard_output_value(&dashboard_fixture(), &args).unwrap();
    let trade = output["trades"][0].as_object().unwrap();
    assert_eq!(trade["Ticker"], "AAPL");
    assert_eq!(trade["FullTimeString24"], "16:00:00");
    assert_eq!(trade["Sweep"], true);
    assert_eq!(trade["ClosingTrade"], true);
    assert!(!trade.contains_key("Time"));
    assert!(!trade.contains_key("venue"));
    assert!(!trade.contains_key("type"));

    let cluster = output["clusters"][0].as_object().unwrap();
    assert_eq!(cluster["MinFullDateTime"], "2026-01-02T16:00:00+00:00");
    assert_eq!(cluster["MaxFullDateTime"], "2026-01-02T16:49:31+00:00");
    assert!(!cluster.contains_key("window"));
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

#[test]
fn dashboard_field_metadata_accepts_documented_projection_fields() {
    let fields = field_metadata::field_names("trade dashboard").unwrap();

    for field in fields {
        let mut args = dashboard_args();
        args.fields = Some(field.clone());

        dashboard_output_value(&dashboard_fixture(), &args)
            .unwrap_or_else(|err| panic!("metadata field `{field}` was rejected: {err}"));
    }
}

#[test]
fn dashboard_field_metadata_covers_raw_dashboard_fields() {
    let mut args = dashboard_args();
    args.all_fields = true;
    let output = dashboard_output_value(&dashboard_fixture(), &args).unwrap();
    let documented = field_metadata::field_names("trade dashboard")
        .unwrap()
        .into_iter()
        .collect::<BTreeSet<_>>();

    for section in ["trades", "clusters", "levels", "cluster_bombs"] {
        let rows = output[section].as_array().unwrap();
        for row in rows {
            let row = row.as_object().unwrap();
            for key in row.keys() {
                let qualified = format!("{section}.{key}");
                assert!(
                    documented.contains(&qualified),
                    "dashboard field `{qualified}` is accepted by --fields but missing from fields metadata"
                );
            }
        }
    }
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
    let preset = find_trade_preset("All Disproportionately Large Trades").expect("preset found");
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

fn default_level_touches_page() -> LevelTouchesPageArgs {
    LevelTouchesPageArgs {
        start: 0,
        length: 100,
        order_col: 10,
        order_dir: OrderDirection::Asc,
    }
}

fn default_level_touches_args() -> LevelTouchesArgs {
    LevelTouchesArgs {
        ticker: Some("AAPL".to_string()),
        dates: empty_optional_dates(),
        ranges: empty_trade_ranges(),
        trade_level_rank: 10,
        trade_level_count: DEFAULT_LEVEL_TOUCH_COUNT,
        vcd: None,
        relative_size: None,
        page: default_level_touches_page(),
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
    set_filter(&mut filters, K_START_DATE, start);
    set_filter(&mut filters, K_END_DATE, end);

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

    assert!(has_filter(&filters, "TradeLevelRank", "10"));
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
