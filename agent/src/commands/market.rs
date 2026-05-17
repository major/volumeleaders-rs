//! Market commands: earnings and exhaustion.

use clap::{Args, Subcommand};
use serde_json::Value;
use tracing::instrument;
use volumeleaders_client::{EarningsRequest, ExhaustionScoresRequest};

use crate::cli::MarketArgs;
use crate::common::auth::{handle_api_error, make_client};
use crate::common::dates::resolve_date_range;
use crate::common::types::OutputFormat;
use crate::output::{finish_output, print_json, print_records};

const DEFAULT_EARNINGS_FIELDS: [&str; 6] = [
    "Ticker",
    "EarningsDate",
    "AfterMarketClose",
    "TradeCount",
    "TradeClusterCount",
    "TradeClusterBombCount",
];

/// Market subcommands.
#[derive(Debug, Subcommand)]
pub enum MarketCommand {
    /// Query earnings data.
    Earnings(EarningsArgs),
    /// Query exhaustion scores.
    Exhaustion(ExhaustionArgs),
}

/// Arguments for `market earnings`.
#[derive(Debug, Args)]
pub struct EarningsArgs {
    /// Start date in YYYY-MM-DD format.
    #[arg(long)]
    pub start_date: Option<String>,

    /// End date in YYYY-MM-DD format.
    #[arg(long)]
    pub end_date: Option<String>,

    /// Look back this many days from the end date or today.
    #[arg(long)]
    pub days: Option<u32>,

    /// Output format.
    #[arg(long, value_enum, default_value = "json")]
    pub format: OutputFormat,

    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,
    /// Return every field from the VolumeLeaders API response.
    #[arg(long)]
    pub all_fields: bool,
}

/// Arguments for `market exhaustion`.
#[derive(Debug, Args)]
pub struct ExhaustionArgs {
    /// Date in YYYY-MM-DD format. Leave empty for the current day.
    #[arg(long)]
    pub date: Option<String>,
}

/// Handles the market command group.
#[instrument(skip_all)]
pub async fn handle(args: &MarketArgs, pretty: bool) -> i32 {
    match &args.command {
        MarketCommand::Earnings(args) => execute_earnings(args, pretty).await,
        MarketCommand::Exhaustion(args) => execute_exhaustion(args, pretty).await,
    }
}

#[instrument(skip_all)]
async fn execute_earnings(args: &EarningsArgs, pretty: bool) -> i32 {
    let request = build_earnings_request(args);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let earnings = match client.get_earnings_limit(&request, usize::MAX).await {
        Ok(earnings) => earnings,
        Err(err) => return handle_api_error(err),
    };

    finish_output(print_records(
        &earnings,
        args.format,
        pretty,
        &DEFAULT_EARNINGS_FIELDS,
        args.fields.as_deref(),
        args.all_fields,
    ))
}

#[instrument(skip_all)]
async fn execute_exhaustion(args: &ExhaustionArgs, pretty: bool) -> i32 {
    let request = ExhaustionScoresRequest {
        date: args.date.clone().unwrap_or_default(),
    };
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let scores = match client.get_exhaustion_scores(&request).await {
        Ok(scores) => scores,
        Err(err) => return handle_api_error(err),
    };

    let json = serde_json::to_value(&scores).unwrap_or(Value::Null);
    finish_output(print_json(&json, pretty))
}

fn build_earnings_request(args: &EarningsArgs) -> EarningsRequest {
    let (start, end) = resolve_date_range(
        args.start_date.as_deref(),
        args.end_date.as_deref(),
        args.days,
    );
    EarningsRequest::new().with_date_range(start, end)
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::cli::Cli;
    use crate::common::types::OutputFormat;

    use super::{EarningsArgs, build_earnings_request};

    #[test]
    fn cli_market_command_has_earnings_and_exhaustion_subcommands() {
        let command = Cli::command();
        let market = command.find_subcommand("market").expect("market command");
        let names: Vec<_> = market
            .get_subcommands()
            .map(|command| command.get_name().to_string())
            .collect();

        assert_eq!(names, vec!["earnings", "exhaustion"]);
    }

    #[test]
    fn build_earnings_request_sets_date_filters() {
        let args = EarningsArgs {
            start_date: Some("2025-01-01".to_string()),
            end_date: Some("2025-01-15".to_string()),
            days: None,
            format: OutputFormat::Json,
            fields: None,
            all_fields: false,
        };

        let request = build_earnings_request(&args);

        assert_eq!(
            request.extra_values()[0],
            ("StartDate".to_string(), "2025-01-01".to_string())
        );
        assert_eq!(
            request.extra_values()[1],
            ("EndDate".to_string(), "2025-01-15".to_string())
        );
    }
}
