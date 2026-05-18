//! Earnings endpoint for `/Earnings/GetEarnings` DataTables API.

use tracing::instrument;

use crate::client::Client;
use crate::datatables::{
    DataTablesColumn, DataTablesRequest, DataTablesResponse, impl_datatables_request_methods,
};
use crate::error::Result;
use crate::models::Earning;

/// Browser endpoint path for `/Earnings/GetEarnings`.
pub(crate) const EARNINGS_GET_EARNINGS_PATH: &str = "/Earnings/GetEarnings";

/// Request parameters for the `/Earnings/GetEarnings` endpoint.
///
/// Wraps a [`DataTablesRequest`] with pre-configured column definitions
/// matching the VolumeLeaders earnings table.
#[derive(Clone, Debug)]
pub struct EarningsRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(EarningsRequest);

impl EarningsRequest {
    /// Create a new earnings request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: earnings_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Set the date range filters used by the earnings endpoint.
    #[must_use]
    pub fn with_date_range(mut self, start: impl Into<String>, end: impl Into<String>) -> Self {
        self.0 = self
            .0
            .with_extra_value("StartDate", start)
            .with_extra_value("EndDate", end);
        self
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        self.0.to_pairs()
    }
}

impl Default for EarningsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Return the DataTables column definitions for the earnings table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`EarningsColumns`) exactly.
#[must_use]
pub fn earnings_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("Date", "Earnings Date", true, true),
        DataTablesColumn::new("Ticker", "Ticker", true, true),
        DataTablesColumn::new("Current", "Current", true, false),
        DataTablesColumn::new("Sector", "Sector", true, true),
        DataTablesColumn::new("Industry", "Industry", true, true),
        DataTablesColumn::new("TradeCount", "Recent Top-100 Trades", true, true),
        DataTablesColumn::new("TradeClusterCount", "Recent Top-100 Clusters", true, true),
        DataTablesColumn::new("TradeClusterBombCount", "Recent Top-100 Bombs", true, true),
        DataTablesColumn::new("Ticker", "Charts", true, false),
    ]
}

impl Client {
    /// Post a DataTables request to `/Earnings/GetEarnings` and return the
    /// typed response envelope.
    #[instrument(skip_all)]
    pub async fn get_earnings(
        &self,
        request: &EarningsRequest,
    ) -> Result<DataTablesResponse<Earning>> {
        self.post_datatables(EARNINGS_GET_EARNINGS_PATH, request.to_pairs())
            .await
    }

    /// Fetch up to `limit` earnings by paginating `/Earnings/GetEarnings`.
    /// A zero limit fetches all available records.
    #[instrument(skip_all)]
    pub async fn get_earnings_limit(
        &self,
        request: &EarningsRequest,
        limit: usize,
    ) -> Result<Vec<Earning>> {
        self.fetch_limit(EARNINGS_GET_EARNINGS_PATH, request.0.clone(), limit)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_client;

    // -- column definition tests --

    #[test]
    fn earnings_columns_returns_9_columns() {
        let columns = earnings_columns();
        assert_eq!(columns.len(), 9);
    }

    #[test]
    fn earnings_columns_first_and_last_match_go_source() {
        let columns = earnings_columns();

        // First column: Date (orderable).
        assert_eq!(columns[0].data, "Date");
        assert_eq!(columns[0].name, "Earnings Date");
        assert!(columns[0].searchable);
        assert!(columns[0].orderable);

        // Last column: Ticker/Charts (not orderable).
        assert_eq!(columns[8].data, "Ticker");
        assert_eq!(columns[8].name, "Charts");
        assert!(columns[8].searchable);
        assert!(!columns[8].orderable);
    }

    #[test]
    fn earnings_columns_trade_count_at_index_5() {
        let columns = earnings_columns();
        assert_eq!(columns[5].data, "TradeCount");
        assert_eq!(columns[5].name, "Recent Top-100 Trades");
        assert!(columns[5].orderable);
    }

    #[test]
    fn earnings_columns_current_not_orderable() {
        let columns = earnings_columns();
        assert_eq!(columns[2].data, "Current");
        assert_eq!(columns[2].name, "Current");
        assert!(!columns[2].orderable);
    }

    // -- endpoint tests --

    #[tokio::test]
    async fn get_earnings_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("earnings_response.json");
        let mock = server
            .mock("POST", EARNINGS_GET_EARNINGS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client.get_earnings(&EarningsRequest::new()).await.unwrap();

        assert_eq!(response.draw, 6);
        assert_eq!(response.records_total, 2);
        assert_eq!(response.records_filtered, 2);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AMD"));
        assert_eq!(
            response.data[0].name.as_deref(),
            Some("Advanced Micro Devices")
        );
        assert_eq!(response.data[0].current, Some(220.25));
        assert_eq!(response.data[0].trade_count, Some(9));
        assert_eq!(response.data[0].after_market_close, Some(true));
        assert_eq!(response.data[0].sector.as_deref(), Some("Technology"));
        assert_eq!(response.data[1].ticker.as_deref(), Some("NVDA"));
        assert!(response.data[1].sector.is_none());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_earnings_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("earnings_response.json");
        server
            .mock("POST", EARNINGS_GET_EARNINGS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let earnings = client
            .get_earnings_limit(&EarningsRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(earnings.len(), 1);
        assert_eq!(earnings[0].ticker.as_deref(), Some("AMD"));
    }
}
