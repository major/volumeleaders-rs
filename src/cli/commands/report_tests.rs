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
fn preset_filters_are_base_defaults_plus_overrides() {
    let top_10 = REPORT_PRESETS
        .iter()
        .find(|p| p.use_name == "top-10-rank")
        .expect("top-10-rank preset must exist");

    assert_eq!(top_10.overrides, &[("TradeRank", "10")]);
    assert!(top_10.omitted_filters.is_empty());
    assert_eq!(top_10.filters().len(), BASE_REPORT_FILTERS.len());
}

#[test]
fn preset_omitted_filters_removes_base_keys() {
    let preset = REPORT_PRESETS
        .iter()
        .find(|p| p.use_name == "disproportionately-large")
        .expect("disproportionately-large preset must exist");

    assert_eq!(preset.omitted_filters, &["TradeCount"]);
    assert!(!preset.filters().iter().any(|&(key, _)| key == "TradeCount"));
}

#[test]
fn top_100_rank_has_trade_rank_100() {
    let preset = REPORT_PRESETS
        .iter()
        .find(|p| p.use_name == "top-100-rank")
        .expect("top-100-rank preset must exist");
    let rank = preset
        .filters()
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
        .filters()
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
        .filters()
        .iter()
        .find(|&&(k, _)| k == "DarkPools")
        .map(|&(_, v)| v);
    let sweeps = preset
        .filters()
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
        .filters()
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
        .filters()
        .iter()
        .find(|&&(k, _)| k == "VCD")
        .map(|&(_, v)| v);
    let rank = preset
        .filters()
        .iter()
        .find(|&&(k, _)| k == "TradeRank")
        .map(|&(_, v)| v);
    let rs = preset
        .filters()
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
        ReportCommand::Top100Rank(flags.clone()).preset().map(|p| p.0),
        Some("top-100-rank")
    );

    // List has no preset.
    assert!(ReportCommand::List.preset().is_none());
}

#[test]
fn validate_report_fields_uses_preset_metadata() {
    validate_report_fields("top-100-rank", Some("Ticker,Dollars")).unwrap();
    validate_report_fields("top-10-rank", Some("DollarsMultiplier")).unwrap();

    let relative_size_err =
        validate_report_fields("top-10-rank", Some("RelativeSize")).unwrap_err();
    assert!(relative_size_err.to_string().contains("RelativeSize"));

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

    assert!(execute_preset(&args).await.is_err());
}
