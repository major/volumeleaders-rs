use clap::{CommandFactory, Parser};

use crate::cli::Cli;

use super::*;

#[test]
fn cli_alert_command_has_four_subcommands() {
    let command = Cli::command();
    let alert = command.find_subcommand("alert").expect("alert command");
    let names: Vec<_> = alert
        .get_subcommands()
        .map(|cmd| cmd.get_name().to_string())
        .collect();

    assert_eq!(names, vec!["configs", "create", "edit", "delete"]);
}

#[test]
fn edit_requires_key_flag() {
    let result = Cli::try_parse_from(["volumeleaders-agent", "alert", "edit"]);
    assert!(result.is_err(), "edit without --key should fail");

    let result = Cli::try_parse_from(["volumeleaders-agent", "alert", "edit", "--key", "42"]);
    assert!(result.is_ok(), "edit with --key should succeed");
}

#[test]
fn create_accepts_bare_sweep_flag() {
    let cli = Cli::try_parse_from([
        "volumeleaders-agent",
        "alert",
        "create",
        "--name",
        "SweepAlert",
        "--sweep",
    ])
    .expect("bare --sweep flag should parse");

    assert!(matches!(
        cli.command,
        crate::cli::Commands::Alert(AlertArgs {
            command: AlertCommand::Create(CreateArgs {
                config: AlertConfigFlags { sweep: true, .. },
                ..
            }),
        })
    ));
}

#[test]
fn create_accepts_explicit_false_sweep_value() {
    let cli = Cli::try_parse_from([
        "volumeleaders-agent",
        "alert",
        "create",
        "--name",
        "SweepAlert",
        "--sweep",
        "false",
    ])
    .expect("explicit --sweep false value should parse");

    assert!(matches!(
        cli.command,
        crate::cli::Commands::Alert(AlertArgs {
            command: AlertCommand::Create(CreateArgs {
                config: AlertConfigFlags { sweep: false, .. },
                ..
            }),
        })
    ));
}

#[test]
fn documented_big_tech_sweeps_example_parses() {
    let cli = Cli::try_parse_from([
        "volumeleaders-agent",
        "alert",
        "create",
        "--name",
        "BigTechSweeps",
        "--tickers",
        "AAPL,MSFT",
        "--trade-dollars-gte",
        "1000000",
        "--sweep",
    ])
    .expect("documented BigTechSweeps example should parse");

    assert!(matches!(
        cli.command,
        crate::cli::Commands::Alert(AlertArgs {
            command: AlertCommand::Create(CreateArgs {
                name,
                config: AlertConfigFlags {
                    tickers,
                    trade_dollars_gte: 1_000_000,
                    sweep: true,
                    ..
                },
                ..
            }),
        }) if name == "BigTechSweeps" && tickers == "AAPL,MSFT"
    ));
}

#[tokio::test]
async fn create_dry_run_finishes_without_auth() {
    let args = test_create_args(true);

    assert!(execute_create(&args).await.is_ok());
}

#[tokio::test]
async fn edit_dry_run_finishes_without_auth() {
    let args = test_edit_args(true);

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

#[test]
fn resolve_ticker_group_uses_explicit_group() {
    let ticker_group = resolve_ticker_group(Some("MyWatchlist"), "AAPL,MSFT");

    assert_eq!(ticker_group, "MyWatchlist");
}

#[test]
fn resolve_ticker_group_auto_selects_tickers() {
    let ticker_group = resolve_ticker_group(None, "AAPL,MSFT");

    assert_eq!(ticker_group, "SelectedTickers");
}

#[test]
fn resolve_ticker_group_defaults_to_all_tickers() {
    let ticker_group = resolve_ticker_group(None, "");

    assert_eq!(ticker_group, "AllTickers");
}

#[test]
fn build_edit_request_uses_key_and_name() {
    let args = test_edit_args(false);

    let request = build_edit_request(&args);
    let fields = request.fields();

    // AlertConfigKey matches edit key.
    assert_eq!(fields[0], ("AlertConfigKey".into(), "42".into()));
    // Name matches input.
    assert_eq!(fields[1], ("Name".into(), "Edited Alert".into()));
    // Auto-selected SelectedTickers because tickers is non-empty.
    assert_eq!(fields[2], ("TickerGroup".into(), "SelectedTickers".into()));
    // Tickers preserved.
    assert_eq!(fields[3], ("Tickers".into(), "AAPL,MSFT".into()));
}

#[test]
fn build_edit_request_defaults_missing_name_to_empty() {
    let args = EditArgs {
        key: 42,
        dry_run: false,
        name: None,
        config: AlertConfigFlags {
            tickers: "AAPL".to_string(),
            ..test_config_flags()
        },
    };

    let request = build_edit_request(&args);
    let fields = request.fields();

    // AlertConfigKey matches edit key.
    assert_eq!(fields[0], ("AlertConfigKey".into(), "42".into()));
    // Name defaults to empty when omitted.
    assert_eq!(fields[1], ("Name".into(), "".into()));
}

#[test]
fn build_create_request_auto_selects_ticker_group() {
    let args = test_create_args(false);

    let request = build_create_request(&args);
    let fields = request.fields();

    // AlertConfigKey is 0 for create.
    assert_eq!(fields[0], ("AlertConfigKey".into(), "0".into()));
    // Name matches input.
    assert_eq!(fields[1], ("Name".into(), "Test Alert".into()));
    // Auto-selected SelectedTickers because tickers is non-empty.
    assert_eq!(fields[2], ("TickerGroup".into(), "SelectedTickers".into()));
    // Tickers preserved.
    assert_eq!(fields[3], ("Tickers".into(), "AAPL,MSFT".into()));

    // DarkPool is true: dual entries (true then false).
    let dark_pool_entries: Vec<_> = fields.iter().filter(|(k, _)| k == "DarkPool").collect();
    assert_eq!(dark_pool_entries.len(), 2);
    assert_eq!(dark_pool_entries[0].1, "true");
    assert_eq!(dark_pool_entries[1].1, "false");

    // Sweep is false: single false entry.
    let sweep_entries: Vec<_> = fields.iter().filter(|(k, _)| k == "Sweep").collect();
    assert_eq!(sweep_entries.len(), 1);
    assert_eq!(sweep_entries[0].1, "false");

    // OffsettingPrint is true: dual entries.
    let op_entries: Vec<_> = fields
        .iter()
        .filter(|(k, _)| k == "OffsettingPrint")
        .collect();
    assert_eq!(op_entries.len(), 2);
    assert_eq!(op_entries[0].1, "true");
    assert_eq!(op_entries[1].1, "false");

    // PhantomPrint is false: single false entry.
    let pp_entries: Vec<_> = fields.iter().filter(|(k, _)| k == "PhantomPrint").collect();
    assert_eq!(pp_entries.len(), 1);
    assert_eq!(pp_entries[0].1, "false");
}

fn test_config_flags() -> AlertConfigFlags {
    AlertConfigFlags {
        ticker_group: None,
        tickers: "AAPL,MSFT".to_string(),
        trade_rank_lte: 0,
        trade_vcd_gte: 0,
        trade_mult_gte: 0,
        trade_volume_gte: 0,
        trade_dollars_gte: 0,
        trade_conditions: "0".to_string(),
        dark_pool: true,
        sweep: false,
        closing_trade_rank_lte: 0,
        closing_trade_vcd_gte: 0,
        closing_trade_mult_gte: 0,
        closing_trade_volume_gte: 0,
        closing_trade_dollars_gte: 0,
        closing_trade_conditions: "0".to_string(),
        cluster_rank_lte: 0,
        cluster_vcd_gte: 0,
        cluster_mult_gte: 0,
        cluster_volume_gte: 0,
        cluster_dollars_gte: 0,
        total_rank_lte: 0,
        total_volume_gte: 0,
        total_dollars_gte: 0,
        ah_rank_lte: 0,
        ah_volume_gte: 0,
        ah_dollars_gte: 0,
        offsetting_print: true,
        phantom_print: false,
    }
}

fn test_create_args(dry_run: bool) -> CreateArgs {
    CreateArgs {
        dry_run,
        name: "Test Alert".to_string(),
        config: test_config_flags(),
    }
}

fn test_edit_args(dry_run: bool) -> EditArgs {
    EditArgs {
        dry_run,
        key: 42,
        name: Some("Edited Alert".to_string()),
        config: test_config_flags(),
    }
}
