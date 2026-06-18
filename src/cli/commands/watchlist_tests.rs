use clap::{CommandFactory, Parser};

use crate::cli::Cli;

use super::*;

#[test]
fn cli_watchlist_command_has_six_subcommands() {
    let command = Cli::command();
    let watchlist = command
        .find_subcommand("watchlist")
        .expect("watchlist command");
    let names: Vec<_> = watchlist
        .get_subcommands()
        .map(|cmd| cmd.get_name().to_string())
        .collect();

    assert_eq!(
        names,
        vec![
            "configs",
            "tickers",
            "create",
            "edit",
            "delete",
            "add-ticker"
        ]
    );
}

#[test]
fn build_config_request_creates_correct_form_with_dual_entry_bools() {
    let cfg = WatchlistConfigFlags {
        tickers: "SPY,AAPL".to_string(),
        min_volume: 100,
        max_volume: 2_000_000_000,
        min_dollars: 0.0,
        max_dollars: 30_000_000_000.0,
        min_price: 0.0,
        max_price: 100_000.0,
        min_vcd: 0.0,
        sector_industry: String::new(),
        security_type: -1,
        min_relative_size: 0,
        max_trade_rank: -1,
        normal_prints: true,
        signature_prints: false,
        late_prints: true,
        timely_prints: true,
        dark_pools: true,
        lit_exchanges: true,
        sweeps: true,
        blocks: true,
        premarket_trades: true,
        rth_trades: true,
        ah_trades: true,
        opening_trades: true,
        closing_trades: true,
        phantom_trades: true,
        offsetting_trades: true,
        rsi_overbought_daily: -1,
        rsi_overbought_hourly: -1,
        rsi_oversold_daily: -1,
        rsi_oversold_hourly: -1,
    };

    let request = build_config_request(0, "Test WL", &cfg);
    let fields = request.fields();

    // Key is 0 for create.
    assert_eq!(fields[0], ("SearchTemplateKey".into(), "0".into()));
    assert_eq!(fields[1], ("Name".into(), "Test WL".into()));
    assert_eq!(fields[2], ("Tickers".into(), "SPY,AAPL".into()));
    assert_eq!(fields[3], ("MinVolume".into(), "100".into()));

    // NormalPrintsSelected is true: dual entries (true then false).
    let normal_entries: Vec<_> = fields
        .iter()
        .filter(|(k, _)| k == "NormalPrintsSelected")
        .collect();
    assert_eq!(normal_entries.len(), 2);
    assert_eq!(normal_entries[0].1, "true");
    assert_eq!(normal_entries[1].1, "false");

    // SignaturePrintsSelected is false: single false entry.
    let sig_entries: Vec<_> = fields
        .iter()
        .filter(|(k, _)| k == "SignaturePrintsSelected")
        .collect();
    assert_eq!(sig_entries.len(), 1);
    assert_eq!(sig_entries[0].1, "false");

    // RSI fields are simple string values, not dual-entry.
    let rsi_entries: Vec<_> = fields
        .iter()
        .filter(|(k, _)| k == "RSIOverboughtDailySelected")
        .collect();
    assert_eq!(rsi_entries.len(), 1);
    assert_eq!(rsi_entries[0].1, "-1");
}

#[test]
fn create_can_disable_default_true_watchlist_boolean() {
    let cli = Cli::try_parse_from([
        "volumeleaders-agent",
        "watchlist",
        "create",
        "--name",
        "NoNormalPrints",
        "--normal-prints",
        "false",
    ])
    .expect("--normal-prints false should parse");

    assert!(matches!(
        cli.command,
        crate::cli::Commands::Watchlist(WatchlistArgs {
            command: WatchlistCommand::Create(CreateArgs {
                name,
                config: WatchlistConfigFlags {
                    normal_prints: false,
                    ..
                },
                ..
            }),
        }) if name == "NoNormalPrints"
    ));
}

#[test]
fn create_keeps_bare_default_true_watchlist_boolean_enabled() {
    let cli = Cli::try_parse_from([
        "volumeleaders-agent",
        "watchlist",
        "create",
        "--name",
        "NormalPrints",
        "--normal-prints",
    ])
    .expect("bare --normal-prints should still parse as true");

    assert!(matches!(
        cli.command,
        crate::cli::Commands::Watchlist(WatchlistArgs {
            command: WatchlistCommand::Create(CreateArgs {
                name,
                config: WatchlistConfigFlags {
                    normal_prints: true,
                    ..
                },
                ..
            }),
        }) if name == "NormalPrints"
    ));
}

#[tokio::test]
async fn create_dry_run_finishes_without_auth() {
    let args = CreateArgs {
        dry_run: true,
        name: "DryRunWatchlist".to_string(),
        config: test_config_flags(),
    };

    assert!(execute_create(&args).await.is_ok());
}

#[tokio::test]
async fn edit_dry_run_finishes_without_auth() {
    let args = EditArgs {
        dry_run: true,
        key: 42,
        name: Some("DryRunWatchlist".to_string()),
        config: test_config_flags(),
    };

    assert!(execute_edit(&args).await.is_ok());
}

#[tokio::test]
async fn delete_dry_run_finishes_without_confirmation_or_auth() {
    let args = DeleteArgs {
        dry_run: true,
        yes: false,
        key: 42,
    };

    assert!(execute_delete(&args).await.is_ok());
}

#[tokio::test]
async fn delete_without_confirmation_fails_before_auth() {
    let args = DeleteArgs {
        dry_run: false,
        yes: false,
        key: 42,
    };

    assert!(execute_delete(&args).await.is_err());
}

#[tokio::test]
async fn add_ticker_dry_run_finishes_without_auth() {
    let args = AddTickerArgs {
        dry_run: true,
        watchlist_key: 42,
        ticker: "NVDA".to_string(),
    };

    assert!(execute_add_ticker(&args).await.is_ok());
}

#[test]
fn build_config_request_edits_config_with_nonzero_key_and_name() {
    let cfg = WatchlistConfigFlags {
        tickers: "MSFT,NVDA".to_string(),
        min_volume: 500,
        max_volume: 1_500_000,
        min_dollars: 10_000.5,
        max_dollars: 25_000_000.25,
        min_price: 12.5,
        max_price: 450.75,
        min_vcd: 2.5,
        sector_industry: "Technology".to_string(),
        security_type: 1,
        min_relative_size: 25,
        max_trade_rank: 10,
        normal_prints: false,
        signature_prints: true,
        late_prints: false,
        timely_prints: true,
        dark_pools: false,
        lit_exchanges: true,
        sweeps: false,
        blocks: true,
        premarket_trades: false,
        rth_trades: true,
        ah_trades: false,
        opening_trades: true,
        closing_trades: false,
        phantom_trades: true,
        offsetting_trades: false,
        rsi_overbought_daily: 1,
        rsi_overbought_hourly: 0,
        rsi_oversold_daily: 1,
        rsi_oversold_hourly: 0,
    };

    let request = build_config_request(42, "Edited WL", &cfg);
    let fields = request.fields();

    // Key is non-zero for edit.
    assert_eq!(fields[0], ("SearchTemplateKey".into(), "42".into()));
    assert_eq!(fields[1], ("Name".into(), "Edited WL".into()));
    assert_eq!(fields[2], ("Tickers".into(), "MSFT,NVDA".into()));
    assert_eq!(fields[3], ("MinVolume".into(), "500".into()));
    assert_eq!(fields[4], ("MaxVolume".into(), "1500000".into()));
    assert_eq!(fields[5], ("MinDollars".into(), "10000.5".into()));
    assert_eq!(fields[6], ("MaxDollars".into(), "25000000.25".into()));
    assert_eq!(fields[7], ("MinPrice".into(), "12.5".into()));
    assert_eq!(fields[8], ("MaxPrice".into(), "450.75".into()));
    assert_eq!(fields[9], ("MinVCD".into(), "2.5".into()));
    assert_eq!(fields[10], ("SectorIndustry".into(), "Technology".into()));
    assert_eq!(fields[11], ("SecurityTypeKey".into(), "1".into()));
    assert_eq!(fields[12], ("MinRelativeSizeSelected".into(), "25".into()));
    assert_eq!(fields[13], ("MaxTradeRankSelected".into(), "10".into()));
}

#[test]
fn build_config_request_edit_flips_dual_entry_bool_paths() {
    let cfg = WatchlistConfigFlags {
        tickers: "QQQ".to_string(),
        min_volume: 0,
        max_volume: 2_000_000_000,
        min_dollars: 0.0,
        max_dollars: 30_000_000_000.0,
        min_price: 0.0,
        max_price: 100_000.0,
        min_vcd: 0.0,
        sector_industry: String::new(),
        security_type: -1,
        min_relative_size: 0,
        max_trade_rank: -1,
        normal_prints: false,
        signature_prints: true,
        late_prints: false,
        timely_prints: true,
        dark_pools: false,
        lit_exchanges: true,
        sweeps: false,
        blocks: true,
        premarket_trades: false,
        rth_trades: true,
        ah_trades: false,
        opening_trades: true,
        closing_trades: false,
        phantom_trades: true,
        offsetting_trades: false,
        rsi_overbought_daily: 0,
        rsi_overbought_hourly: 1,
        rsi_oversold_daily: 0,
        rsi_oversold_hourly: 1,
    };

    let request = build_config_request(42, "Edited WL", &cfg);
    let fields = request.fields();

    // Key and name stay on the edit path while bool fields flip paths.
    assert_eq!(fields[0], ("SearchTemplateKey".into(), "42".into()));
    assert_eq!(fields[1], ("Name".into(), "Edited WL".into()));

    // NormalPrintsSelected is false: single false entry.
    let normal_entries: Vec<_> = fields
        .iter()
        .filter(|(k, _)| k == "NormalPrintsSelected")
        .collect();
    assert_eq!(normal_entries.len(), 1);
    assert_eq!(normal_entries[0].1, "false");

    // SignaturePrintsSelected is true: dual entries (true then false).
    let sig_entries: Vec<_> = fields
        .iter()
        .filter(|(k, _)| k == "SignaturePrintsSelected")
        .collect();
    assert_eq!(sig_entries.len(), 2);
    assert_eq!(sig_entries[0].1, "true");
    assert_eq!(sig_entries[1].1, "false");

    // RSI fields remain simple string values on edit.
    let rsi_entries: Vec<_> = fields
        .iter()
        .filter(|(k, _)| k == "RSIOversoldHourlySelected")
        .collect();
    assert_eq!(rsi_entries.len(), 1);
    assert_eq!(rsi_entries[0].1, "1");
}

#[test]
fn tickers_request_sets_watchlist_key_in_extra_values() {
    let request = WatchListTickersRequest::new().with_watch_list_key(6260);

    let wl_pair = request
        .extra_values()
        .iter()
        .find(|(k, _)| k == "WatchListKey")
        .expect("WatchListKey should be in extra_values");
    assert_eq!(wl_pair.1, "6260");
}

#[test]
fn edit_requires_key_flag() {
    let result = Cli::try_parse_from(["volumeleaders-agent", "watchlist", "edit"]);
    assert!(result.is_err(), "edit without --key should fail");

    let result = Cli::try_parse_from(["volumeleaders-agent", "watchlist", "edit", "--key", "42"]);
    assert!(result.is_ok(), "edit with --key should succeed");
}

#[test]
fn delete_requires_key_flag() {
    let result = Cli::try_parse_from(["volumeleaders-agent", "watchlist", "delete"]);
    assert!(result.is_err(), "delete without --key should fail");

    let result = Cli::try_parse_from(["volumeleaders-agent", "watchlist", "delete", "--key", "99"]);
    assert!(result.is_ok(), "delete with --key should succeed");
}

#[test]
fn add_ticker_requires_both_flags() {
    let result = Cli::try_parse_from(["volumeleaders-agent", "watchlist", "add-ticker"]);
    assert!(result.is_err(), "add-ticker without flags should fail");

    let result = Cli::try_parse_from([
        "volumeleaders-agent",
        "watchlist",
        "add-ticker",
        "--watchlist-key",
        "6260",
        "--ticker",
        "AAPL",
    ]);
    assert!(result.is_ok(), "add-ticker with both flags should succeed");
}

fn test_config_flags() -> WatchlistConfigFlags {
    WatchlistConfigFlags {
        tickers: "SPY,AAPL".to_string(),
        min_volume: 100,
        max_volume: 2_000_000_000,
        min_dollars: 0.0,
        max_dollars: 30_000_000_000.0,
        min_price: 0.0,
        max_price: 100_000.0,
        min_vcd: 0.0,
        sector_industry: String::new(),
        security_type: -1,
        min_relative_size: 0,
        max_trade_rank: -1,
        normal_prints: true,
        signature_prints: false,
        late_prints: true,
        timely_prints: true,
        dark_pools: true,
        lit_exchanges: true,
        sweeps: true,
        blocks: true,
        premarket_trades: true,
        rth_trades: true,
        ah_trades: true,
        opening_trades: true,
        closing_trades: true,
        phantom_trades: true,
        offsetting_trades: true,
        rsi_overbought_daily: -1,
        rsi_overbought_hourly: -1,
        rsi_oversold_daily: -1,
        rsi_oversold_hourly: -1,
    }
}
