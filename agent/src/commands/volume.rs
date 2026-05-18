//! Volume commands: institutional, after-hours institutional, and total volume.

use clap::{Args, Subcommand};
use tracing::instrument;
use volumeleaders_client::VolumeRequest;

use crate::cli::VolumeArgs;
use crate::common::auth::{handle_api_error, make_client};
use crate::common::tickers::parse_tickers;
use crate::common::trade_transforms::TradeRecordKind;
use crate::common::types::OrderDirection;
use crate::output::{finish_output, print_transformed_record_values};

const VOLUME_HEADERS: [&str; 20] = [
    "Date",
    "FullDateTime",
    "Ticker",
    "Name",
    "Sector",
    "Industry",
    "Price",
    "Volume",
    "Dollars",
    "DollarsMultiplier",
    "PercentDailyVolume",
    "RelativeSize",
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

    /// Comma-separated field list for output.
    #[arg(long, conflicts_with = "all_fields")]
    pub fields: Option<String>,

    /// Return every field after semantic trade transforms.
    #[arg(long)]
    pub all_fields: bool,
}

/// Volume subcommands.
#[derive(Debug, Subcommand)]
pub enum VolumeCommand {
    /// Query institutional volume.
    Institutional {
        #[command(flatten)]
        args: VolumeOptions,
    },
    /// Query after-hours institutional volume.
    AhInstitutional {
        #[command(flatten)]
        args: VolumeOptions,
    },
    /// Query total volume.
    Total {
        #[command(flatten)]
        args: VolumeOptions,
    },
}

/// Handles the volume command group.
#[instrument(skip_all)]
pub async fn handle(args: &VolumeArgs, pretty: bool) -> i32 {
    match &args.command {
        VolumeCommand::Institutional { args } => execute_institutional(args, pretty).await,
        VolumeCommand::AhInstitutional { args } => execute_ah_institutional(args, pretty).await,
        VolumeCommand::Total { args } => execute_total(args, pretty).await,
    }
}

#[instrument(skip_all)]
async fn execute_institutional(args: &VolumeOptions, pretty: bool) -> i32 {
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

    output_records(&trades, pretty, args.fields.as_deref(), args.all_fields)
}

#[instrument(skip_all)]
async fn execute_ah_institutional(args: &VolumeOptions, pretty: bool) -> i32 {
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

    output_records(&trades, pretty, args.fields.as_deref(), args.all_fields)
}

#[instrument(skip_all)]
async fn execute_total(args: &VolumeOptions, pretty: bool) -> i32 {
    let request = build_request(VolumeRequest::total(), args);
    let client = match make_client().await {
        Ok(client) => client,
        Err(code) => return code,
    };
    let trades = match client.get_total_volume_limit(&request, args.limit).await {
        Ok(trades) => trades,
        Err(err) => return handle_api_error(err),
    };

    output_records(&trades, pretty, args.fields.as_deref(), args.all_fields)
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
    pretty: bool,
    fields: Option<&str>,
    all_fields: bool,
) -> i32 {
    finish_output(print_transformed_record_values(
        records,
        TradeRecordKind::Trade,
        pretty,
        &VOLUME_HEADERS,
        fields,
        all_fields,
    ))
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::cli::Cli;

    use super::*;

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
}
