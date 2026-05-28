//! Volume commands: institutional, after-hours institutional, and total volume.

use crate::VolumeRequest;
use clap::{Args, Subcommand};
use tracing::instrument;

use crate::cli::VolumeArgs;
use crate::cli::common::auth::{handle_api_error, make_client};
use crate::cli::common::tickers::parse_tickers;
use crate::cli::common::trade_transforms::TradeRecordKind;
use crate::cli::common::types::OrderDirection;
use crate::cli::field_metadata;
use crate::cli::output::{finish_output, print_transformed_record_values_with_allowed_fields};

const VOLUME_HEADERS: [&str; 16] = [
    "Date",
    "FullDateTime",
    "Ticker",
    "Sector",
    "Industry",
    "Price",
    "Dollars",
    "DollarsMultiplier",
    "CumulativeDistribution",
    "TradeRank",
    "type",
    "venue",
    "LatePrint",
    "SignaturePrint",
    "PhantomPrint",
    "events",
];

/// Shared volume command flags.
#[derive(Debug, Args)]
pub struct VolumeOptions {
    /// Trading date in YYYY-MM-DD format.
    #[arg(long)]
    pub date: String,

    /// Comma-separated ticker symbols.
    #[arg(long)]
    pub tickers: Option<String>,

    /// Maximum number of rows to return.
    #[arg(long, default_value_t = 100)]
    pub limit: usize,

    /// Sort direction for the volume leaderboard.
    #[arg(long = "order-dir", value_enum, default_value = "asc")]
    pub order_dir: OrderDirection,

    /// Exact, case-sensitive output fields to keep, comma-separated; discover with `fields volume institutional`, `fields volume ah-institutional`, or `fields volume total`.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,

    /// Return every field after semantic trade transforms.
    #[arg(long)]
    pub all_fields: bool,
}

/// Volume subcommands.
#[allow(missing_docs)]
#[derive(Debug, Subcommand)]
pub enum VolumeCommand {
    /// Query institutional volume.
    #[command(
        long_about = "Query the institutional volume leaderboard for a trading date.\n\nExamples:\n  volumeleaders-agent volume institutional --date 2026-05-27\n  volumeleaders-agent volume institutional --date 2026-05-27 --tickers AAPL,NVDA --limit 50"
    )]
    Institutional {
        #[command(flatten)]
        args: VolumeOptions,
    },
    /// Query after-hours institutional volume.
    #[command(
        long_about = "Query the after-hours institutional volume leaderboard for a trading date.\n\nExamples:\n  volumeleaders-agent volume ah-institutional --date 2026-05-27\n  volumeleaders-agent volume ah-institutional --date 2026-05-27 --tickers AAPL,NVDA --limit 50"
    )]
    AhInstitutional {
        #[command(flatten)]
        args: VolumeOptions,
    },
    /// Query total volume.
    #[command(
        long_about = "Query the total volume leaderboard for a trading date.\n\nExamples:\n  volumeleaders-agent volume total --date 2026-05-27\n  volumeleaders-agent volume total --date 2026-05-27 --tickers SPY,QQQ --order-dir desc --limit 50"
    )]
    Total {
        #[command(flatten)]
        args: VolumeOptions,
    },
}

/// Handles the volume command group.
#[instrument(skip_all)]
pub async fn handle(args: &VolumeArgs) -> i32 {
    match &args.command {
        VolumeCommand::Institutional { args } => execute_institutional(args).await,
        VolumeCommand::AhInstitutional { args } => execute_ah_institutional(args).await,
        VolumeCommand::Total { args } => execute_total(args).await,
    }
}

#[instrument(skip_all)]
async fn execute_institutional(args: &VolumeOptions) -> i32 {
    let request = build_request(VolumeRequest::institutional(), args);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let trades = match client
        .get_institutional_volume_limit(&request, args.limit)
        .await
    {
        Ok(trades) => trades,
        Err(err) => return handle_api_error(err),
    };

    output_records(&trades, args.fields.as_deref(), args.all_fields)
}

#[instrument(skip_all)]
async fn execute_ah_institutional(args: &VolumeOptions) -> i32 {
    let request = build_request(VolumeRequest::ah_institutional(), args);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let trades = match client
        .get_ah_institutional_volume_limit(&request, args.limit)
        .await
    {
        Ok(trades) => trades,
        Err(err) => return handle_api_error(err),
    };

    output_records(&trades, args.fields.as_deref(), args.all_fields)
}

#[instrument(skip_all)]
async fn execute_total(args: &VolumeOptions) -> i32 {
    let request = build_request(VolumeRequest::total(), args);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let trades = match client.get_total_volume_limit(&request, args.limit).await {
        Ok(trades) => trades,
        Err(err) => return handle_api_error(err),
    };

    output_records(&trades, args.fields.as_deref(), args.all_fields)
}

fn build_request(mut request: VolumeRequest, args: &VolumeOptions) -> VolumeRequest {
    request = request.with_date(args.date.clone());

    if let Some(tickers) = args.tickers.as_deref() {
        let parsed = parse_tickers(tickers);
        if !parsed.is_empty() {
            request = request.with_tickers(parsed.join(","));
        }
    }

    request.with_order(1, args.order_dir.as_str(), "")
}

fn output_records<T: serde::Serialize>(
    records: &[T],
    fields: Option<&str>,
    all_fields: bool,
) -> i32 {
    let allowed_fields = field_metadata::field_names("volume institutional");
    finish_output(print_transformed_record_values_with_allowed_fields(
        records,
        TradeRecordKind::Trade,
        &VOLUME_HEADERS,
        fields,
        all_fields,
        allowed_fields.as_deref(),
    ))
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::Trade;
    use crate::cli::Cli;

    use super::*;

    fn trade(value: serde_json::Value) -> Trade {
        serde_json::from_value(value).unwrap()
    }

    fn sample_args() -> VolumeOptions {
        VolumeOptions {
            date: "2025-01-15".to_string(),
            tickers: Some("aapl,msft".to_string()),
            limit: 25,
            order_dir: OrderDirection::Asc,
            fields: None,
            all_fields: false,
        }
    }

    #[test]
    fn build_request_sets_date_tickers_and_order() {
        let request = build_request(VolumeRequest::institutional(), &sample_args());

        assert_eq!(
            request.extra_values()[0],
            ("Date".to_string(), "2025-01-15".to_string())
        );
        assert_eq!(
            request.extra_values()[1],
            ("Tickers".to_string(), "AAPL,MSFT".to_string())
        );
        assert!(request.encode().contains("order[0][dir]=asc"));
    }

    #[test]
    fn build_request_skips_empty_ticker_filters() {
        let mut args = sample_args();
        args.tickers = Some(" , , ".to_string());

        let request = build_request(VolumeRequest::total(), &args);

        assert_eq!(
            request.extra_values(),
            vec![("Date".to_string(), "2025-01-15".to_string())]
        );
    }

    #[test]
    fn cli_volume_command_has_three_subcommands() {
        let command = Cli::command();
        let volume = command.find_subcommand("volume").expect("volume command");
        let names: Vec<_> = volume
            .get_subcommands()
            .map(|command| command.get_name().to_string())
            .collect();

        assert_eq!(names, vec!["institutional", "ah-institutional", "total"]);
    }

    #[test]
    fn output_records_accepts_discovered_metadata_field_absent_from_rows() {
        let records = vec![trade(serde_json::json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Price": 200.0,
            "Dollars": 10_000_000.0
        }))];

        assert_eq!(output_records(&records, Some("events"), false), 0);
    }

    #[test]
    fn output_records_rejects_field_missing_from_metadata() {
        let records = vec![trade(serde_json::json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Price": 200.0,
            "Dollars": 10_000_000.0
        }))];

        assert_eq!(
            output_records(&records, Some("NotAField"), false),
            crate::cli::error::EXIT_USAGE_ERROR
        );
    }
}
