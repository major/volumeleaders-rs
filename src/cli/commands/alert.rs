//! Alert commands: configs, create, edit, delete.

use crate::{
    AlertConfigsRequest, DeleteAlertConfigRequest, SaveAlertConfigFields, SaveAlertConfigRequest,
};
use clap::{Args, Subcommand};
use tracing::instrument;

use crate::cli::AlertArgs;
use crate::cli::commands::scaffold::run_client_command;
use crate::cli::common::auth::make_client;
use crate::cli::dry_run::print_dry_run_plan;
use crate::cli::error::{CliExit, usage_error};
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
        long_about = "Create a new alert configuration with threshold filters.\n\nExamples:\n  volumeleaders-agent alert create --name LargeNVDA --tickers NVDA\n  volumeleaders-agent alert create --name BigTechSweeps --tickers AAPL,MSFT --trade-dollars-gte 1000000 --sweep\n  volumeleaders-agent alert create --name BigTechSweeps --tickers AAPL,MSFT --dry-run"
    )]
    Create(CreateArgs),
    /// Edit an existing alert configuration.
    #[command(
        long_about = "Edit an existing alert configuration by key.\n\nExamples:\n  volumeleaders-agent alert edit --key 123 --name LargeNVDA\n  volumeleaders-agent alert edit --key 123 --tickers NVDA,AAPL --trade-dollars-gte 2000000 --dark-pool\n  volumeleaders-agent alert edit --key 123 --name LargeNVDA --dry-run"
    )]
    Edit(EditArgs),
    /// Delete an alert configuration.
    #[command(
        long_about = "Delete an alert configuration by key. Live deletion requires --yes.\n\nExamples:\n  volumeleaders-agent alert delete --key 123 --dry-run\n  volumeleaders-agent alert delete --key 456 --yes"
    )]
    Delete(DeleteArgs),
}

/// Arguments for `alert configs`.
#[derive(Debug, Args)]
pub struct ConfigsArgs {
    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields alert configs`.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `alert create`.
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Print the alert create request as JSON without sending it.
    #[arg(long)]
    pub dry_run: bool,

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
    /// Print the alert edit request as JSON without sending it.
    #[arg(long)]
    pub dry_run: bool,

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
    /// Print the alert delete request as JSON without sending it.
    #[arg(long)]
    pub dry_run: bool,

    /// Confirm the live delete operation. Not required with --dry-run.
    #[arg(long)]
    pub yes: bool,

    /// Alert configuration key to delete.
    #[arg(long)]
    pub key: i64,
}

/// Handles the alert command group.
#[instrument(skip_all)]
pub async fn handle(args: &AlertArgs) -> Result<(), CliExit> {
    match &args.command {
        AlertCommand::Configs(a) => execute_configs(a).await,
        AlertCommand::Create(a) => execute_create(a).await,
        AlertCommand::Edit(a) => execute_edit(a).await,
        AlertCommand::Delete(a) => execute_delete(a).await,
    }
}

#[instrument(skip_all)]
async fn execute_configs(args: &ConfigsArgs) -> Result<(), CliExit> {
    let client = make_client().await?;
    let request = AlertConfigsRequest::new();
    let configs = client.get_alert_configs_limit(&request, usize::MAX).await?;

    finish_output(print_records(
        &configs,
        &DEFAULT_CONFIGS_FIELDS,
        args.fields.as_deref(),
        args.all_fields,
    ))
}

#[instrument(skip_all)]
async fn execute_create(args: &CreateArgs) -> Result<(), CliExit> {
    let request = build_create_request(args);
    if args.dry_run {
        return print_dry_run_plan("alert create", "create", request.fields());
    }

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
async fn execute_edit(args: &EditArgs) -> Result<(), CliExit> {
    let request = build_edit_request(args);
    let key = args.key;
    if args.dry_run {
        return print_dry_run_plan("alert edit", "edit", request.fields());
    }

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
async fn execute_delete(args: &DeleteArgs) -> Result<(), CliExit> {
    let key = args.key;
    let request = DeleteAlertConfigRequest {
        alert_config_key: key,
    };
    if args.dry_run {
        return print_dry_run_plan("alert delete", "delete", &request);
    }
    if !args.yes {
        return Err(usage_error(
            "alert delete requires --yes to confirm live deletion; use --dry-run to inspect the request",
        ));
    }

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
#[path = "alert_tests.rs"]
mod tests;
