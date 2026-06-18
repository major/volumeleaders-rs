//! Market commands: earnings and exhaustion.

use crate::cli::error::CliExit;
use crate::{EarningsRequest, ExhaustionScoresRequest};
use clap::{Args, Subcommand};
use tracing::instrument;

use crate::cli::MarketArgs;
use crate::cli::commands::scaffold::run_client_command;
use crate::cli::common::auth::make_client;
use crate::cli::common::dates::resolve_date_range;
use crate::cli::output::{finish_output, print_json, print_records};

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
    #[command(
        long_about = "Query earnings dates and related trade counts.\n\nExamples:\n  volumeleaders-agent market earnings\n  volumeleaders-agent market earnings --start-date 2026-05-01 --end-date 2026-05-27 --fields Ticker,EarningsDate,TradeCount"
    )]
    Earnings(EarningsArgs),
    /// Query exhaustion scores.
    #[command(
        long_about = "Query market exhaustion scores for a date.\n\nExamples:\n  volumeleaders-agent market exhaustion\n  volumeleaders-agent market exhaustion --date 2026-05-27"
    )]
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

    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields market earnings`.
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
pub async fn handle(args: &MarketArgs) -> Result<(), CliExit> {
    match &args.command {
        MarketCommand::Earnings(args) => execute_earnings(args).await,
        MarketCommand::Exhaustion(args) => execute_exhaustion(args).await,
    }
}

#[instrument(skip_all)]
async fn execute_earnings(args: &EarningsArgs) -> Result<(), CliExit> {
    let request = build_earnings_request(args);
    let client = make_client().await?;
    let earnings = client.get_earnings_limit(&request, usize::MAX).await?;

    finish_output(print_records(
        &earnings,
        &DEFAULT_EARNINGS_FIELDS,
        args.fields.as_deref(),
        args.all_fields,
    ))
}

#[instrument(skip_all)]
async fn execute_exhaustion(args: &ExhaustionArgs) -> Result<(), CliExit> {
    let request = ExhaustionScoresRequest {
        date: args.date.clone().unwrap_or_default(),
    };
    run_client_command(
        move |client| Box::pin(async move { client.get_exhaustion_scores(&request).await }),
        move |scores| {
            let json = serde_json::to_value(&scores).unwrap_or(serde_json::Value::Null);
            print_json(&json)
        },
    )
    .await
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
