//! Executive summary endpoints for `/ExecutiveSummary/GetExhaustionScores`,
//! `/ExecutiveSummary/GetWelcomeTrades`,
//! `/ExecutiveSummary/GetWelcomeTradeClusters`, and
//! `/Trades/GetAllSnapshots` APIs.

use std::collections::HashMap;

use serde::Serialize;
use tracing::instrument;

use crate::client::{Client, FormPairs};
use crate::datatables::{
    DataTablesColumn, DataTablesRequest, impl_datatables_client_methods,
    impl_datatables_request_methods,
};
use crate::error::Result;
use crate::models::{ExhaustionScore, Trade, TradeCluster};

/// Browser endpoint path for `/ExecutiveSummary/GetExhaustionScores`.
pub(crate) const EXECUTIVE_SUMMARY_GET_EXHAUSTION_SCORES_PATH: &str =
    "/ExecutiveSummary/GetExhaustionScores";

/// Browser endpoint path for `/ExecutiveSummary/GetWelcomeTrades`.
pub(crate) const EXECUTIVE_SUMMARY_GET_WELCOME_TRADES_PATH: &str =
    "/ExecutiveSummary/GetWelcomeTrades";

/// Browser endpoint path for `/ExecutiveSummary/GetWelcomeTradeClusters`.
pub(crate) const EXECUTIVE_SUMMARY_GET_WELCOME_TRADE_CLUSTERS_PATH: &str =
    "/ExecutiveSummary/GetWelcomeTradeClusters";

/// Browser endpoint path for `/Trades/GetAllSnapshots`.
pub(crate) const TRADES_GET_ALL_SNAPSHOTS_PATH: &str = "/Trades/GetAllSnapshots";

/// JSON request payload for `/ExecutiveSummary/GetExhaustionScores`.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExhaustionScoresRequest {
    /// Date string for the exhaustion scores query (e.g. `"2026-05-01"`).
    pub date: String,
}

/// Request parameters for `/ExecutiveSummary/GetWelcomeTrades`.
///
/// Wraps a [`DataTablesRequest`] with pre-configured column definitions
/// matching the VolumeLeaders welcome trades table.
#[derive(Clone, Debug)]
pub struct WelcomeTradesRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(WelcomeTradesRequest);

impl WelcomeTradesRequest {
    /// Create a welcome trades request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: welcome_trades_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> FormPairs {
        self.0.to_pairs()
    }
}

impl Default for WelcomeTradesRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request parameters for `/ExecutiveSummary/GetWelcomeTradeClusters`.
///
/// Wraps a [`DataTablesRequest`] with pre-configured column definitions
/// matching the VolumeLeaders welcome trade clusters table.
#[derive(Clone, Debug)]
pub struct WelcomeTradeClustersRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(WelcomeTradeClustersRequest);

impl WelcomeTradeClustersRequest {
    /// Create a welcome trade clusters request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: welcome_trade_clusters_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> FormPairs {
        self.0.to_pairs()
    }
}

impl Default for WelcomeTradeClustersRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Return the DataTables column definitions for the welcome trades table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`WelcomeTradesColumns`) exactly.
#[must_use]
pub fn welcome_trades_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("Ticker", "Ticker", true, true),
        DataTablesColumn::new("TradeRank", "R", true, true),
        DataTablesColumn::new("DollarsMultiplier", "RS", true, true),
        DataTablesColumn::new("CumulativeDistribution", "PCT", true, true),
        DataTablesColumn::new("LastComparibleTradeDate", "Charts", true, false),
    ]
}

/// Return the DataTables column definitions for the welcome trade clusters
/// table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`WelcomeTradeClustersColumns`) exactly.
#[must_use]
pub fn welcome_trade_clusters_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("Ticker", "Ticker", true, true),
        DataTablesColumn::new("TradeClusterRank", "R", true, true),
        DataTablesColumn::new("DollarsMultiplier", "RS", true, true),
        DataTablesColumn::new("CumulativeDistribution", "PCT", true, true),
        DataTablesColumn::new("LastComparibleTradeClusterDate", "Charts", true, false),
    ]
}

/// Parse the semicolon-delimited ticker snapshot string returned by
/// `/Trades/GetAllSnapshots` into a ticker-to-price map.
///
/// Each item has the form `TICKER:PRICE` separated by semicolons.
/// Empty items (from trailing semicolons) are silently skipped.
///
/// # Errors
///
/// Returns an error if an item is missing the `:` separator or the price
/// portion cannot be parsed as `f64`.
pub fn parse_snapshots(raw: &str) -> Result<HashMap<String, f64>> {
    let mut snapshots = HashMap::new();
    for item in raw.split(';') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let (ticker, price_str) = item.split_once(':').ok_or_else(|| {
            crate::error::ClientError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("parse snapshot {item:?}: missing separator"),
            ))
        })?;
        let ticker = ticker.trim();
        if ticker.is_empty() {
            return Err(crate::error::ClientError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("parse snapshot {item:?}: missing ticker"),
            )));
        }
        let price: f64 = price_str.trim().parse().map_err(|e| {
            crate::error::ClientError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("parse snapshot price for {ticker:?}: {e}"),
            ))
        })?;
        snapshots.insert(ticker.to_string(), price);
    }
    Ok(snapshots)
}

impl_datatables_client_methods!(
    get_welcome_trades,
    get_welcome_trades_limit,
    WelcomeTradesRequest,
    Trade,
    EXECUTIVE_SUMMARY_GET_WELCOME_TRADES_PATH
);
impl_datatables_client_methods!(
    get_welcome_trade_clusters,
    get_welcome_trade_clusters_limit,
    WelcomeTradeClustersRequest,
    TradeCluster,
    EXECUTIVE_SUMMARY_GET_WELCOME_TRADE_CLUSTERS_PATH
);

impl Client {
    /// Post a JSON request to `/ExecutiveSummary/GetExhaustionScores` and
    /// return the exhaustion score data.
    #[instrument(skip_all)]
    pub async fn get_exhaustion_scores(
        &self,
        request: &ExhaustionScoresRequest,
    ) -> Result<ExhaustionScore> {
        let body = self
            .post_json(EXECUTIVE_SUMMARY_GET_EXHAUSTION_SCORES_PATH, request)
            .await?;
        Ok(serde_json::from_str(&body)?)
    }

    /// Post a JSON null request to `/Trades/GetAllSnapshots` and return
    /// ticker snapshot prices keyed by ticker symbol.
    #[instrument(skip_all)]
    pub async fn get_all_snapshots(&self) -> Result<HashMap<String, f64>> {
        let raw = self.get_all_snapshots_string().await?;
        parse_snapshots(&raw)
    }

    /// Post a JSON null request to `/Trades/GetAllSnapshots` and return the
    /// raw semicolon-delimited ticker snapshot string.
    #[instrument(skip_all)]
    pub async fn get_all_snapshots_string(&self) -> Result<String> {
        let body = self.post_json(TRADES_GET_ALL_SNAPSHOTS_PATH, &()).await?;
        Ok(serde_json::from_str(&body)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_client;

    // -- column definition tests --

    #[test]
    fn welcome_trades_columns_returns_5_columns() {
        let columns = welcome_trades_columns();
        assert_eq!(columns.len(), 5);
    }

    #[test]
    fn welcome_trades_columns_match_go_source() {
        let columns = welcome_trades_columns();

        assert_eq!(columns[0].data, "Ticker");
        assert_eq!(columns[0].name, "Ticker");
        assert!(columns[0].orderable);

        assert_eq!(columns[1].data, "TradeRank");
        assert_eq!(columns[1].name, "R");
        assert!(columns[1].orderable);

        assert_eq!(columns[4].data, "LastComparibleTradeDate");
        assert_eq!(columns[4].name, "Charts");
        assert!(!columns[4].orderable);
    }

    #[test]
    fn welcome_trade_clusters_columns_returns_5_columns() {
        let columns = welcome_trade_clusters_columns();
        assert_eq!(columns.len(), 5);
    }

    #[test]
    fn welcome_trade_clusters_columns_match_go_source() {
        let columns = welcome_trade_clusters_columns();

        assert_eq!(columns[0].data, "Ticker");
        assert_eq!(columns[0].name, "Ticker");
        assert!(columns[0].orderable);

        assert_eq!(columns[1].data, "TradeClusterRank");
        assert_eq!(columns[1].name, "R");
        assert!(columns[1].orderable);

        assert_eq!(columns[4].data, "LastComparibleTradeClusterDate");
        assert_eq!(columns[4].name, "Charts");
        assert!(!columns[4].orderable);
    }

    // -- parse_snapshots tests --

    #[test]
    fn parse_snapshots_parses_ticker_prices() {
        let raw = "A:114.52;AA:62.67;";
        let snapshots = parse_snapshots(raw).unwrap();
        assert!((snapshots["A"] - 114.52).abs() < 0.0001);
        assert!((snapshots["AA"] - 62.67).abs() < 0.0001);
    }

    #[test]
    fn parse_snapshots_skips_empty_items() {
        let raw = "SPY:450.00;;QQQ:380.50;";
        let snapshots = parse_snapshots(raw).unwrap();
        assert_eq!(snapshots.len(), 2);
        assert!((snapshots["SPY"] - 450.0).abs() < 0.0001);
    }

    #[test]
    fn parse_snapshots_reports_missing_separator() {
        let err = parse_snapshots("A:114.52;broken").unwrap_err();
        assert!(err.to_string().contains("missing separator"));
    }

    #[test]
    fn parse_snapshots_reports_missing_ticker() {
        let err = parse_snapshots(":123.45").unwrap_err();
        assert!(err.to_string().contains("missing ticker"));
    }

    #[test]
    fn parse_snapshots_reports_invalid_price() {
        let err = parse_snapshots("AMD:notanumber").unwrap_err();
        assert!(err.to_string().contains("parse snapshot price"));
    }

    // -- endpoint tests --

    #[tokio::test]
    async fn get_exhaustion_scores_returns_parsed_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", EXECUTIVE_SUMMARY_GET_EXHAUSTION_SCORES_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "DateKey": 20260501,
                    "ExhaustionScoreRank": 4,
                    "ExhaustionScoreRank30Day": 8,
                    "ExhaustionScoreRank90Day": 11,
                    "ExhaustionScoreRank365Day": 22
                }"#,
            )
            .create_async()
            .await;
        let client = test_client(&server);

        let score = client
            .get_exhaustion_scores(&ExhaustionScoresRequest {
                date: "2026-05-01".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(score.date_key, Some(20_260_501));
        assert_eq!(score.exhaustion_score_rank, Some(4));
        assert_eq!(score.exhaustion_score_rank_30_day, Some(8));
        assert_eq!(score.exhaustion_score_rank_90_day, Some(11));
        assert_eq!(score.exhaustion_score_rank_365_day, Some(22));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_welcome_trades_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("welcome_trades_response.json");
        let mock = server
            .mock("POST", EXECUTIVE_SUMMARY_GET_WELCOME_TRADES_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_welcome_trades(&WelcomeTradesRequest::new())
            .await
            .unwrap();

        assert_eq!(response.draw, 7);
        assert_eq!(response.records_total, 2);
        assert_eq!(response.records_filtered, 2);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AMD"));
        assert_eq!(response.data[0].trade_rank, Some(11));
        assert_eq!(response.data[1].ticker.as_deref(), Some("NVDA"));
        assert_eq!(response.data[1].trade_rank, Some(5));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_welcome_trades_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("welcome_trades_response.json");
        server
            .mock("POST", EXECUTIVE_SUMMARY_GET_WELCOME_TRADES_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let trades = client
            .get_welcome_trades_limit(&WelcomeTradesRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].ticker.as_deref(), Some("AMD"));
    }

    #[tokio::test]
    async fn get_welcome_trade_clusters_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("welcome_trade_clusters_response.json");
        let mock = server
            .mock("POST", EXECUTIVE_SUMMARY_GET_WELCOME_TRADE_CLUSTERS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_welcome_trade_clusters(&WelcomeTradeClustersRequest::new())
            .await
            .unwrap();

        assert_eq!(response.draw, 8);
        assert_eq!(response.records_total, 2);
        assert_eq!(response.records_filtered, 2);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AMD"));
        assert_eq!(response.data[0].trade_cluster_rank, Some(7));
        assert_eq!(response.data[1].ticker.as_deref(), Some("MSFT"));
        assert_eq!(response.data[1].trade_cluster_rank, Some(3));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_welcome_trade_clusters_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("welcome_trade_clusters_response.json");
        server
            .mock("POST", EXECUTIVE_SUMMARY_GET_WELCOME_TRADE_CLUSTERS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let clusters = client
            .get_welcome_trade_clusters_limit(&WelcomeTradeClustersRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].ticker.as_deref(), Some("AMD"));
    }

    #[tokio::test]
    async fn get_all_snapshots_parses_ticker_prices() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", TRADES_GET_ALL_SNAPSHOTS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#""A:114.52;AA:62.67;""#)
            .create_async()
            .await;
        let client = test_client(&server);

        let snapshots = client.get_all_snapshots().await.unwrap();

        assert!((snapshots["A"] - 114.52).abs() < 0.0001);
        assert!((snapshots["AA"] - 62.67).abs() < 0.0001);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_all_snapshots_string_returns_raw_string() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", TRADES_GET_ALL_SNAPSHOTS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#""A:114.52;AA:62.67;""#)
            .create_async()
            .await;
        let client = test_client(&server);

        let raw = client.get_all_snapshots_string().await.unwrap();

        assert_eq!(raw, "A:114.52;AA:62.67;");
        mock.assert_async().await;
    }
}
