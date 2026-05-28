//! Alert commands: configs, create, edit, delete.

use crate::{
    AlertConfigsRequest, DeleteAlertConfigRequest, SaveAlertConfigFields, SaveAlertConfigRequest,
};
use clap::{Args, Subcommand};
use tracing::instrument;

use crate::cli::AlertArgs;
use crate::cli::commands::scaffold::run_client_command;
use crate::cli::common::auth::{handle_api_error, make_client};
use crate::cli::output::{finish_output, print_json, print_records};

const DEFAULT_CONFIGS_FIELDS: [&str; 9] = [
    "AlertConfigKey",
    "Name",
    "Tickers",
    "TradeConditions",
    "ClosingTradeConditions",
    "DarkPool",
    "Sweep",
    "OffsettingPrint",
    "PhantomPrint",
];

/// Alert subcommands.
#[derive(Debug, Subcommand)]
pub enum AlertCommand {
    /// List all alert configurations.
    #[command(
        long_about = "List all saved alert configurations.\n\nExamples:\n  volumeleaders-agent alert configs\n  volumeleaders-agent alert configs --fields AlertConfigKey,Name,Tickers"
    )]
    Configs(ConfigsArgs),
    /// Create a new alert configuration.
    #[command(
        long_about = "Create a new alert configuration with threshold filters.\n\nExamples:\n  volumeleaders-agent alert create --name LargeNVDA --tickers NVDA\n  volumeleaders-agent alert create --name BigTechSweeps --tickers AAPL,MSFT --trade-dollars-gte 1000000 --sweep"
    )]
    Create(CreateArgs),
    /// Edit an existing alert configuration.
    #[command(
        long_about = "Edit an existing alert configuration by key.\n\nExamples:\n  volumeleaders-agent alert edit --key 123 --name LargeNVDA\n  volumeleaders-agent alert edit --key 123 --tickers NVDA,AAPL --trade-dollars-gte 2000000 --dark-pool"
    )]
    Edit(EditArgs),
    /// Delete an alert configuration.
    #[command(
        long_about = "Delete an alert configuration by key.\n\nExamples:\n  volumeleaders-agent alert delete --key 123\n  volumeleaders-agent alert delete --key 456"
    )]
    Delete(DeleteArgs),
}

/// Arguments for `alert configs`.
#[derive(Debug, Args)]
pub struct ConfigsArgs {
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `alert create`.
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Alert configuration name.
    #[arg(long)]
    pub name: String,

    /// Ticker group name; defaults to AllTickers and switches to SelectedTickers when tickers are set.
    #[arg(long)]
    pub ticker_group: Option<String>,

    /// Comma-separated ticker symbols for SelectedTickers alerts.
    #[arg(long, default_value = "")]
    pub tickers: String,

    /// Maximum trade rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub trade_rank_lte: i64,

    /// Minimum trade volume-concentration delta score, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub trade_vcd_gte: i64,

    /// Minimum trade dollar multiplier, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub trade_mult_gte: i64,

    /// Minimum trade share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub trade_volume_gte: i64,

    /// Minimum trade dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub trade_dollars_gte: i64,

    /// Trade condition code filter accepted by the alert API; use 0 for no condition filter.
    #[arg(long, default_value = "0")]
    pub trade_conditions: String,

    /// Require dark-pool trades when true; false leaves the alert unrestricted by dark-pool status.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub dark_pool: bool,

    /// Require sweep trades when true; false leaves the alert unrestricted by sweep status.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub sweep: bool,

    /// Maximum closing trade rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_rank_lte: i64,

    /// Minimum closing trade volume-concentration delta score, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_vcd_gte: i64,

    /// Minimum closing trade dollar multiplier, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_mult_gte: i64,

    /// Minimum closing trade share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_volume_gte: i64,

    /// Minimum closing trade dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_dollars_gte: i64,

    /// Closing trade condition code filter accepted by the alert API; use 0 for no condition filter.
    #[arg(long, default_value = "0")]
    pub closing_trade_conditions: String,

    /// Maximum cluster rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_rank_lte: i64,

    /// Minimum cluster volume-concentration delta score, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_vcd_gte: i64,

    /// Minimum cluster dollar multiplier, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_mult_gte: i64,

    /// Minimum cluster share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_volume_gte: i64,

    /// Minimum cluster dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_dollars_gte: i64,

    /// Maximum total rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub total_rank_lte: i64,

    /// Minimum total share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub total_volume_gte: i64,

    /// Minimum total dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub total_dollars_gte: i64,

    /// Maximum after-hours rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub ah_rank_lte: i64,

    /// Minimum after-hours share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub ah_volume_gte: i64,

    /// Minimum after-hours dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub ah_dollars_gte: i64,

    /// Require offsetting prints when true; false leaves the alert unrestricted by offsetting status.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub offsetting_print: bool,

    /// Require phantom prints when true; false leaves the alert unrestricted by phantom status.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub phantom_print: bool,
}

/// Arguments for `alert edit`.
#[derive(Debug, Args)]
pub struct EditArgs {
    /// Alert configuration key to edit.
    #[arg(long)]
    pub key: i64,

    /// Alert configuration name.
    #[arg(long)]
    pub name: Option<String>,

    /// Ticker group name; defaults to AllTickers and switches to SelectedTickers when tickers are set.
    #[arg(long)]
    pub ticker_group: Option<String>,

    /// Comma-separated ticker symbols for SelectedTickers alerts.
    #[arg(long, default_value = "")]
    pub tickers: String,

    /// Maximum trade rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub trade_rank_lte: i64,

    /// Minimum trade volume-concentration delta score, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub trade_vcd_gte: i64,

    /// Minimum trade dollar multiplier, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub trade_mult_gte: i64,

    /// Minimum trade share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub trade_volume_gte: i64,

    /// Minimum trade dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub trade_dollars_gte: i64,

    /// Trade condition code filter accepted by the alert API; use 0 for no condition filter.
    #[arg(long, default_value = "0")]
    pub trade_conditions: String,

    /// Require dark-pool trades when true; false leaves the alert unrestricted by dark-pool status.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub dark_pool: bool,

    /// Require sweep trades when true; false leaves the alert unrestricted by sweep status.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub sweep: bool,

    /// Maximum closing trade rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_rank_lte: i64,

    /// Minimum closing trade volume-concentration delta score, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_vcd_gte: i64,

    /// Minimum closing trade dollar multiplier, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_mult_gte: i64,

    /// Minimum closing trade share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_volume_gte: i64,

    /// Minimum closing trade dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub closing_trade_dollars_gte: i64,

    /// Closing trade condition code filter accepted by the alert API; use 0 for no condition filter.
    #[arg(long, default_value = "0")]
    pub closing_trade_conditions: String,

    /// Maximum cluster rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_rank_lte: i64,

    /// Minimum cluster volume-concentration delta score, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_vcd_gte: i64,

    /// Minimum cluster dollar multiplier, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_mult_gte: i64,

    /// Minimum cluster share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_volume_gte: i64,

    /// Minimum cluster dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub cluster_dollars_gte: i64,

    /// Maximum total rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub total_rank_lte: i64,

    /// Minimum total share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub total_volume_gte: i64,

    /// Minimum total dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub total_dollars_gte: i64,

    /// Maximum after-hours rank to alert on; lower ranks are more significant, and 0 disables this threshold.
    #[arg(long, default_value = "0")]
    pub ah_rank_lte: i64,

    /// Minimum after-hours share volume, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub ah_volume_gte: i64,

    /// Minimum after-hours dollar value, or 0 to disable this threshold.
    #[arg(long, default_value = "0")]
    pub ah_dollars_gte: i64,

    /// Require offsetting prints when true; false leaves the alert unrestricted by offsetting status.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub offsetting_print: bool,

    /// Require phantom prints when true; false leaves the alert unrestricted by phantom status.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "false",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub phantom_print: bool,
}

/// Arguments for `alert delete`.
#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Alert configuration key to delete.
    #[arg(long)]
    pub key: i64,
}

/// Handles the alert command group.
#[instrument(skip_all)]
pub async fn handle(args: &AlertArgs) -> i32 {
    match &args.command {
        AlertCommand::Configs(a) => execute_configs(a).await,
        AlertCommand::Create(a) => execute_create(a).await,
        AlertCommand::Edit(a) => execute_edit(a).await,
        AlertCommand::Delete(a) => execute_delete(a).await,
    }
}

#[instrument(skip_all)]
async fn execute_configs(args: &ConfigsArgs) -> i32 {
    let client = match make_client().await {
        Ok(c) => c,
        Err(code) => return code,
    };
    let request = AlertConfigsRequest::new();
    let configs = match client.get_alert_configs_limit(&request, usize::MAX).await {
        Ok(c) => c,
        Err(err) => return handle_api_error(err),
    };

    finish_output(print_records(
        &configs,
        &DEFAULT_CONFIGS_FIELDS,
        args.fields.as_deref(),
        args.all_fields,
    ))
}

#[instrument(skip_all)]
async fn execute_create(args: &CreateArgs) -> i32 {
    let request = build_create_request(args);
    run_client_command(
        move |client| {
            Box::pin(async move {
                client.save_alert_config(request).await?;
                Ok(serde_json::json!({"success": true, "action": "created", "key": 0}))
            })
        },
        move |result| print_json(&result),
    )
    .await
}

#[instrument(skip_all)]
async fn execute_edit(args: &EditArgs) -> i32 {
    let request = build_edit_request(args);
    let key = args.key;
    run_client_command(
        move |client| {
            Box::pin(async move {
                client.save_alert_config(request).await?;
                Ok(serde_json::json!({"success": true, "action": "updated", "key": key}))
            })
        },
        move |result| print_json(&result),
    )
    .await
}

#[instrument(skip_all)]
async fn execute_delete(args: &DeleteArgs) -> i32 {
    let key = args.key;
    let request = DeleteAlertConfigRequest {
        alert_config_key: key,
    };
    run_client_command(
        move |client| {
            Box::pin(async move {
                client.delete_alert_config(&request).await?;
                Ok(serde_json::json!({"success": true, "action": "deleted", "key": key}))
            })
        },
        move |result| print_json(&result),
    )
    .await
}

fn build_create_request(args: &CreateArgs) -> SaveAlertConfigRequest {
    build_alert_config_request(
        0,
        args.name.clone(),
        args.ticker_group.as_deref(),
        &args.tickers,
        args.trade_rank_lte,
        args.trade_vcd_gte,
        args.trade_mult_gte,
        args.trade_volume_gte,
        args.trade_dollars_gte,
        &args.trade_conditions,
        args.dark_pool,
        args.sweep,
        args.closing_trade_rank_lte,
        args.closing_trade_vcd_gte,
        args.closing_trade_mult_gte,
        args.closing_trade_volume_gte,
        args.closing_trade_dollars_gte,
        &args.closing_trade_conditions,
        args.cluster_rank_lte,
        args.cluster_vcd_gte,
        args.cluster_mult_gte,
        args.cluster_volume_gte,
        args.cluster_dollars_gte,
        args.total_rank_lte,
        args.total_volume_gte,
        args.total_dollars_gte,
        args.ah_rank_lte,
        args.ah_volume_gte,
        args.ah_dollars_gte,
        args.offsetting_print,
        args.phantom_print,
    )
}

fn build_edit_request(args: &EditArgs) -> SaveAlertConfigRequest {
    build_alert_config_request(
        args.key,
        args.name.clone().unwrap_or_default(),
        args.ticker_group.as_deref(),
        &args.tickers,
        args.trade_rank_lte,
        args.trade_vcd_gte,
        args.trade_mult_gte,
        args.trade_volume_gte,
        args.trade_dollars_gte,
        &args.trade_conditions,
        args.dark_pool,
        args.sweep,
        args.closing_trade_rank_lte,
        args.closing_trade_vcd_gte,
        args.closing_trade_mult_gte,
        args.closing_trade_volume_gte,
        args.closing_trade_dollars_gte,
        &args.closing_trade_conditions,
        args.cluster_rank_lte,
        args.cluster_vcd_gte,
        args.cluster_mult_gte,
        args.cluster_volume_gte,
        args.cluster_dollars_gte,
        args.total_rank_lte,
        args.total_volume_gte,
        args.total_dollars_gte,
        args.ah_rank_lte,
        args.ah_volume_gte,
        args.ah_dollars_gte,
        args.offsetting_print,
        args.phantom_print,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_alert_config_request(
    alert_config_key: i64,
    name: String,
    ticker_group: Option<&str>,
    tickers: &str,
    trade_rank_lte: i64,
    trade_vcd_gte: i64,
    trade_mult_gte: i64,
    trade_volume_gte: i64,
    trade_dollars_gte: i64,
    trade_conditions: &str,
    dark_pool: bool,
    sweep: bool,
    closing_trade_rank_lte: i64,
    closing_trade_vcd_gte: i64,
    closing_trade_mult_gte: i64,
    closing_trade_volume_gte: i64,
    closing_trade_dollars_gte: i64,
    closing_trade_conditions: &str,
    cluster_rank_lte: i64,
    cluster_vcd_gte: i64,
    cluster_mult_gte: i64,
    cluster_volume_gte: i64,
    cluster_dollars_gte: i64,
    total_rank_lte: i64,
    total_volume_gte: i64,
    total_dollars_gte: i64,
    ah_rank_lte: i64,
    ah_volume_gte: i64,
    ah_dollars_gte: i64,
    offsetting_print: bool,
    phantom_print: bool,
) -> SaveAlertConfigRequest {
    SaveAlertConfigRequest::from_config(SaveAlertConfigFields {
        alert_config_key,
        name,
        ticker_group: resolve_ticker_group(ticker_group, tickers),
        tickers: tickers.to_string(),
        trade_rank_lte,
        trade_vcd_gte,
        trade_mult_gte,
        trade_volume_gte,
        trade_dollars_gte,
        trade_conditions: trade_conditions.to_string(),
        dark_pool,
        sweep,
        closing_trade_rank_lte,
        closing_trade_vcd_gte,
        closing_trade_mult_gte,
        closing_trade_volume_gte,
        closing_trade_dollars_gte,
        closing_trade_conditions: closing_trade_conditions.to_string(),
        cluster_rank_lte,
        cluster_vcd_gte,
        cluster_mult_gte,
        cluster_volume_gte,
        cluster_dollars_gte,
        total_rank_lte,
        total_volume_gte,
        total_dollars_gte,
        ah_rank_lte,
        ah_volume_gte,
        ah_dollars_gte,
        offsetting_print,
        phantom_print,
    })
}

/// Resolve the ticker group: use the explicit value if provided, auto-select
/// "SelectedTickers" when tickers are set, otherwise default to "AllTickers".
fn resolve_ticker_group(explicit: Option<&str>, tickers: &str) -> String {
    match explicit {
        Some(group) => group.to_string(),
        None if !tickers.is_empty() => "SelectedTickers".to_string(),
        None => "AllTickers".to_string(),
    }
}

#[cfg(test)]
mod tests {
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
                command: AlertCommand::Create(CreateArgs { sweep: true, .. }),
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
                command: AlertCommand::Create(CreateArgs { sweep: false, .. }),
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
                    tickers,
                    trade_dollars_gte: 1_000_000,
                    sweep: true,
                    ..
                }),
            }) if name == "BigTechSweeps" && tickers == "AAPL,MSFT"
        ));
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
        let args = EditArgs {
            key: 42,
            name: Some("Edited Alert".to_string()),
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
        };

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
            name: None,
            ticker_group: None,
            tickers: "AAPL".to_string(),
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
        let args = CreateArgs {
            name: "Test Alert".to_string(),
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
        };

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
}
