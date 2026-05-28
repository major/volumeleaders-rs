//! Watchlist commands: configs, tickers, create, edit, delete, add-ticker.

use crate::{
    AddTickerToWatchListRequest, DeleteWatchListRequest, SaveWatchListConfigFields,
    SaveWatchListConfigRequest, WatchListConfigsRequest, WatchListTickersRequest,
};
use clap::{Args, Subcommand};
use serde_json::Value;
use tracing::instrument;

use crate::cli::WatchlistArgs;
use crate::cli::commands::scaffold::run_client_command;
use crate::cli::common::auth::{handle_api_error, make_client};
use crate::cli::dry_run::print_dry_run_plan;
use crate::cli::error::usage_error;
use crate::cli::output::{finish_output, print_json, print_records};

const DEFAULT_CONFIGS_FIELDS: [&str; 4] = ["SearchTemplateKey", "Name", "Tickers", "Criteria"];
const DEFAULT_TICKERS_FIELDS: [&str; 5] = [
    "Ticker",
    "Price",
    "NearestTop10TradeDate",
    "NearestTop10TradeClusterDate",
    "NearestTop10TradeLevel",
];

/// Watchlist subcommands.
#[derive(Debug, Subcommand)]
pub enum WatchlistCommand {
    /// List all watchlist configurations.
    #[command(
        long_about = "List all saved watchlist configurations.\n\nExamples:\n  volumeleaders-agent watchlist configs\n  volumeleaders-agent watchlist configs --fields SearchTemplateKey,Name,Tickers"
    )]
    Configs(ConfigsArgs),
    /// List tickers in a watchlist.
    #[command(
        long_about = "List tickers in one watchlist or across all watchlists.\n\nExamples:\n  volumeleaders-agent watchlist tickers\n  volumeleaders-agent watchlist tickers --watchlist-key 123 --fields Ticker,Price"
    )]
    Tickers(TickersArgs),
    /// Create a new watchlist configuration.
    #[command(
        long_about = "Create a new watchlist configuration with optional filters.\n\nExamples:\n  volumeleaders-agent watchlist create --name BigTech --tickers AAPL,NVDA\n  volumeleaders-agent watchlist create --name LiquidLargeCaps --min-volume 1000000 --min-dollars 5000000\n  volumeleaders-agent watchlist create --name BigTech --tickers AAPL,NVDA --dry-run"
    )]
    Create(CreateArgs),
    /// Edit an existing watchlist configuration.
    #[command(
        long_about = "Edit an existing watchlist configuration by key.\n\nExamples:\n  volumeleaders-agent watchlist edit --key 123 --name BigTech\n  volumeleaders-agent watchlist edit --key 123 --tickers AAPL,NVDA,MSFT --min-volume 1000000\n  volumeleaders-agent watchlist edit --key 123 --name BigTech --dry-run"
    )]
    Edit(EditArgs),
    /// Delete a watchlist configuration.
    #[command(
        long_about = "Delete a watchlist configuration by key. Live deletion requires --yes.\n\nExamples:\n  volumeleaders-agent watchlist delete --key 123 --dry-run\n  volumeleaders-agent watchlist delete --key 456 --yes"
    )]
    Delete(DeleteArgs),
    /// Add a ticker to an existing watchlist.
    #[command(
        long_about = "Add a ticker to an existing watchlist.\n\nExamples:\n  volumeleaders-agent watchlist add-ticker --watchlist-key 123 --ticker NVDA\n  volumeleaders-agent watchlist add-ticker --watchlist-key 123 --ticker AAPL\n  volumeleaders-agent watchlist add-ticker --watchlist-key 123 --ticker NVDA --dry-run"
    )]
    AddTicker(AddTickerArgs),
}

/// Arguments for `watchlist configs`.
#[derive(Debug, Args)]
pub struct ConfigsArgs {
    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `watchlist tickers`.
#[derive(Debug, Args)]
pub struct TickersArgs {
    /// Watchlist key (-1 for all watchlists).
    #[arg(long, default_value = "-1")]
    pub watchlist_key: i64,

    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `watchlist create`.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Print the watchlist create request as JSON without sending it.
    #[arg(long)]
    pub dry_run: bool,

    /// Watchlist name.
    #[arg(long)]
    pub name: String,

    #[command(flatten)]
    pub config: WatchlistConfigFlags,
}

/// Arguments for `watchlist edit`.
#[allow(missing_docs)]
#[derive(Debug, Args)]
pub struct EditArgs {
    /// Print the watchlist edit request as JSON without sending it.
    #[arg(long)]
    pub dry_run: bool,

    /// Watchlist key to edit.
    #[arg(long)]
    pub key: i64,

    /// Watchlist name (optional for edit).
    #[arg(long)]
    pub name: Option<String>,

    #[command(flatten)]
    pub config: WatchlistConfigFlags,
}

/// Arguments for `watchlist delete`.
#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Print the watchlist delete request as JSON without sending it.
    #[arg(long)]
    pub dry_run: bool,

    /// Confirm the live delete operation. Not required with --dry-run.
    #[arg(long)]
    pub yes: bool,

    /// Watchlist key to delete.
    #[arg(long)]
    pub key: i64,
}

/// Arguments for `watchlist add-ticker`.
#[derive(Debug, Args)]
pub struct AddTickerArgs {
    /// Print the add-ticker request as JSON without sending it.
    #[arg(long)]
    pub dry_run: bool,

    /// Watchlist key to add the ticker to.
    #[arg(long)]
    pub watchlist_key: i64,

    /// Ticker symbol to add.
    #[arg(long)]
    pub ticker: String,
}

/// Shared configuration flags for create and edit.
#[derive(Debug, Args)]
pub struct WatchlistConfigFlags {
    /// Comma-separated ticker symbols to include; leave empty for the API default universe.
    #[arg(long, default_value = "")]
    pub tickers: String,

    /// Minimum share volume threshold for trades in the watchlist.
    #[arg(long, default_value = "0")]
    pub min_volume: i64,

    /// Maximum share volume threshold for trades in the watchlist.
    #[arg(long, default_value = "2000000000")]
    pub max_volume: i64,

    /// Minimum trade dollar value threshold.
    #[arg(long, default_value = "0.0")]
    pub min_dollars: f64,

    /// Maximum trade dollar value threshold.
    #[arg(long, default_value = "30000000000.0")]
    pub max_dollars: f64,

    /// Minimum trade price threshold.
    #[arg(long, default_value = "0.0")]
    pub min_price: f64,

    /// Maximum trade price threshold.
    #[arg(long, default_value = "100000.0")]
    pub max_price: f64,

    /// Minimum volume-concentration delta score threshold.
    #[arg(long, default_value = "0.0")]
    pub min_vcd: f64,

    /// Sector or industry text filter; leave empty to include all sectors.
    #[arg(long, default_value = "")]
    pub sector_industry: String,

    /// Security type code: -1 all, 1 stocks, 26 ETFs, 4 REITs.
    #[arg(long, default_value = "-1")]
    pub security_type: i64,

    /// Minimum relative-size bucket, such as 0, 5, 10, 25, 50, or 100.
    #[arg(long, default_value = "0")]
    pub min_relative_size: i64,

    /// Maximum trade-rank bucket; lower ranks are more significant and -1 disables the filter.
    #[arg(long, default_value = "-1")]
    pub max_trade_rank: i64,

    /// Include normal prints; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub normal_prints: bool,

    /// Include signature prints; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub signature_prints: bool,

    /// Include late prints; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub late_prints: bool,

    /// Include timely prints; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub timely_prints: bool,

    /// Include dark-pool trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub dark_pools: bool,

    /// Include lit-exchange trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub lit_exchanges: bool,

    /// Include sweep trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub sweeps: bool,

    /// Include block trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub blocks: bool,

    /// Include premarket trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub premarket_trades: bool,

    /// Include regular trading-hours trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub rth_trades: bool,

    /// Include after-hours trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub ah_trades: bool,

    /// Include opening trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub opening_trades: bool,

    /// Include closing trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub closing_trades: bool,

    /// Include phantom trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub phantom_trades: bool,

    /// Include offsetting trades; pass false to exclude them, omitted uses the default true.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        value_parser = clap::value_parser!(bool),
        default_value = "true",
        default_missing_value = "true",
        num_args = 0..=1
    )]
    pub offsetting_trades: bool,

    /// Daily RSI overbought filter: -1 ignore, 0 require no, 1 require yes.
    #[arg(long, default_value = "-1")]
    pub rsi_overbought_daily: i64,

    /// Hourly RSI overbought filter: -1 ignore, 0 require no, 1 require yes.
    #[arg(long, default_value = "-1")]
    pub rsi_overbought_hourly: i64,

    /// Daily RSI oversold filter: -1 ignore, 0 require no, 1 require yes.
    #[arg(long, default_value = "-1")]
    pub rsi_oversold_daily: i64,

    /// Hourly RSI oversold filter: -1 ignore, 0 require no, 1 require yes.
    #[arg(long, default_value = "-1")]
    pub rsi_oversold_hourly: i64,
}

/// Handles the watchlist command group.
#[instrument(skip_all)]
pub async fn handle(args: &WatchlistArgs) -> i32 {
    match &args.command {
        WatchlistCommand::Configs(a) => execute_configs(a).await,
        WatchlistCommand::Tickers(a) => execute_tickers(a).await,
        WatchlistCommand::Create(a) => execute_create(a).await,
        WatchlistCommand::Edit(a) => execute_edit(a).await,
        WatchlistCommand::Delete(a) => execute_delete(a).await,
        WatchlistCommand::AddTicker(a) => execute_add_ticker(a).await,
    }
}

#[instrument(skip_all)]
async fn execute_configs(args: &ConfigsArgs) -> i32 {
    let client = match make_client().await {
        Ok(c) => c,
        Err(code) => return code,
    };
    let request = WatchListConfigsRequest::new();
    let configs = match client
        .get_watchlist_configs_limit(&request, usize::MAX)
        .await
    {
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
async fn execute_tickers(args: &TickersArgs) -> i32 {
    let client = match make_client().await {
        Ok(c) => c,
        Err(code) => return code,
    };
    let request = WatchListTickersRequest::new().with_watch_list_key(args.watchlist_key);

    let tickers = match client
        .get_watchlist_tickers_limit(&request, usize::MAX)
        .await
    {
        Ok(t) => t,
        Err(err) => return handle_api_error(err),
    };

    finish_output(print_records(
        &tickers,
        &DEFAULT_TICKERS_FIELDS,
        args.fields.as_deref(),
        args.all_fields,
    ))
}

#[instrument(skip_all)]
async fn execute_create(args: &CreateArgs) -> i32 {
    let request = build_config_request(0, &args.name, &args.config);
    if args.dry_run {
        return print_dry_run_plan("watchlist create", "create", request.fields());
    }

    run_client_command(
        move |client| {
            Box::pin(async move {
                client.save_watchlist_config(request).await?;
                Ok(serde_json::json!({"success": true, "action": "created"}))
            })
        },
        move |result| print_json(&result),
    )
    .await
}

#[instrument(skip_all)]
async fn execute_edit(args: &EditArgs) -> i32 {
    let key = args.key;
    let name = args.name.as_deref().unwrap_or("");
    let request = build_config_request(key, name, &args.config);
    if args.dry_run {
        return print_dry_run_plan("watchlist edit", "edit", request.fields());
    }

    run_client_command(
        move |client| {
            Box::pin(async move {
                client.save_watchlist_config(request).await?;
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
    let request = DeleteWatchListRequest {
        watch_list_key: key,
    };
    if args.dry_run {
        return print_dry_run_plan("watchlist delete", "delete", &request);
    }
    if !args.yes {
        return usage_error(
            "watchlist delete requires --yes to confirm live deletion; use --dry-run to inspect the request",
        );
    }

    run_client_command(
        move |client| {
            Box::pin(async move {
                client.delete_watchlist(&request).await?;
                Ok(serde_json::json!({"success": true, "action": "deleted", "key": key}))
            })
        },
        move |result| print_json(&result),
    )
    .await
}

#[instrument(skip_all)]
async fn execute_add_ticker(args: &AddTickerArgs) -> i32 {
    let request = AddTickerToWatchListRequest {
        watch_list_key: args.watchlist_key,
        ticker: args.ticker.clone(),
    };
    if args.dry_run {
        return print_dry_run_plan("watchlist add-ticker", "add-ticker", &request);
    }

    run_client_command(
        move |client| Box::pin(async move { client.add_ticker_to_watchlist(&request).await }),
        move |response| {
            let json = serde_json::to_value(&response).unwrap_or(Value::Null);
            print_json(&json)
        },
    )
    .await
}

/// Build a typed watchlist config create or edit request.
fn build_config_request(
    key: i64,
    name: &str,
    cfg: &WatchlistConfigFlags,
) -> SaveWatchListConfigRequest {
    SaveWatchListConfigRequest::from_config(SaveWatchListConfigFields {
        search_template_key: key,
        name: name.to_string(),
        tickers: cfg.tickers.clone(),
        min_volume: cfg.min_volume,
        max_volume: cfg.max_volume,
        min_dollars: cfg.min_dollars,
        max_dollars: cfg.max_dollars,
        min_price: cfg.min_price,
        max_price: cfg.max_price,
        min_vcd: cfg.min_vcd,
        sector_industry: cfg.sector_industry.clone(),
        security_type_key: cfg.security_type,
        min_relative_size_selected: cfg.min_relative_size,
        max_trade_rank_selected: cfg.max_trade_rank,
        normal_prints_selected: cfg.normal_prints,
        signature_prints_selected: cfg.signature_prints,
        late_prints_selected: cfg.late_prints,
        timely_prints_selected: cfg.timely_prints,
        dark_pools_selected: cfg.dark_pools,
        lit_exchanges_selected: cfg.lit_exchanges,
        sweeps_selected: cfg.sweeps,
        blocks_selected: cfg.blocks,
        premarket_trades_selected: cfg.premarket_trades,
        rth_trades_selected: cfg.rth_trades,
        ah_trades_selected: cfg.ah_trades,
        opening_trades_selected: cfg.opening_trades,
        closing_trades_selected: cfg.closing_trades,
        phantom_trades_selected: cfg.phantom_trades,
        offsetting_trades_selected: cfg.offsetting_trades,
        rsi_overbought_daily_selected: cfg.rsi_overbought_daily,
        rsi_overbought_hourly_selected: cfg.rsi_overbought_hourly,
        rsi_oversold_daily_selected: cfg.rsi_oversold_daily,
        rsi_oversold_hourly_selected: cfg.rsi_oversold_hourly,
    })
}

#[cfg(test)]
mod tests {
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

        assert_eq!(execute_create(&args).await, 0);
    }

    #[tokio::test]
    async fn edit_dry_run_finishes_without_auth() {
        let args = EditArgs {
            dry_run: true,
            key: 42,
            name: Some("DryRunWatchlist".to_string()),
            config: test_config_flags(),
        };

        assert_eq!(execute_edit(&args).await, 0);
    }

    #[tokio::test]
    async fn delete_dry_run_finishes_without_confirmation_or_auth() {
        let args = DeleteArgs {
            dry_run: true,
            yes: false,
            key: 42,
        };

        assert_eq!(execute_delete(&args).await, 0);
    }

    #[tokio::test]
    async fn delete_without_confirmation_fails_before_auth() {
        let args = DeleteArgs {
            dry_run: false,
            yes: false,
            key: 42,
        };

        assert_eq!(
            execute_delete(&args).await,
            crate::cli::error::EXIT_USAGE_ERROR
        );
    }

    #[tokio::test]
    async fn add_ticker_dry_run_finishes_without_auth() {
        let args = AddTickerArgs {
            dry_run: true,
            watchlist_key: 42,
            ticker: "NVDA".to_string(),
        };

        assert_eq!(execute_add_ticker(&args).await, 0);
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

        let result =
            Cli::try_parse_from(["volumeleaders-agent", "watchlist", "edit", "--key", "42"]);
        assert!(result.is_ok(), "edit with --key should succeed");
    }

    #[test]
    fn delete_requires_key_flag() {
        let result = Cli::try_parse_from(["volumeleaders-agent", "watchlist", "delete"]);
        assert!(result.is_err(), "delete without --key should fail");

        let result =
            Cli::try_parse_from(["volumeleaders-agent", "watchlist", "delete", "--key", "99"]);
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
}
