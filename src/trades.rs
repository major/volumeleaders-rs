//! Trades endpoint for the `/Trades/GetTrades` DataTables API.

use crate::datatables::{
    DataTablesColumn, DataTablesRequest, define_datatables_request, impl_datatables_client_methods,
};
use crate::models::Trade;

/// Browser endpoint path for institutional trades.
pub(crate) const TRADES_PATH: &str = "/Trades/GetTrades";

define_datatables_request!(
    /// Request parameters for the `/Trades/GetTrades` endpoint.
    ///
    /// Wraps a [`DataTablesRequest`] with pre-configured column definitions
    /// matching the VolumeLeaders trades table.
    TradesRequest,
    trades_columns
);

impl TradesRequest {
    /// Set endpoint filters for the trades table.
    #[must_use]
    pub fn with_trade_filters(mut self, filters: Vec<(String, String)>) -> Self {
        self.0 = self.0.with_extra_values(filters);
        self
    }
}

/// Return the DataTables column definitions for the trades table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the browser-captured values exactly.
#[must_use]
pub fn trades_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("FullTimeString24", "", true, false),
        DataTablesColumn::id("FullTimeString24"),
        DataTablesColumn::id("Ticker"),
        DataTablesColumn::new("Current", "Current", true, false),
        DataTablesColumn::new("Trade", "Trade", true, false),
        DataTablesColumn::id("Sector"),
        DataTablesColumn::id("Industry"),
        DataTablesColumn::searchable("Volume", "Sh"),
        DataTablesColumn::searchable("Dollars", "$$"),
        DataTablesColumn::searchable("DollarsMultiplier", "RS"),
        DataTablesColumn::searchable("CumulativeDistribution", "PCT"),
        DataTablesColumn::searchable("TradeRank", "R"),
        DataTablesColumn::id("RelativeSize"),
        DataTablesColumn::id("LastComparibleTradeDate"),
        DataTablesColumn::new(
            "LastComparibleTradeDate",
            "LastComparibleTradeDate",
            true,
            false,
        ),
    ]
}

impl_datatables_client_methods!(
    get_trades,
    get_trades_limit,
    TradesRequest,
    Trade,
    TRADES_PATH
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_client;

    #[test]
    fn trades_columns_returns_15_columns() {
        let columns = trades_columns();
        assert_eq!(columns.len(), 15);
    }

    #[test]
    fn trades_columns_first_and_last_match_go_source() {
        let columns = trades_columns();

        // First column: time display (not orderable).
        assert_eq!(columns[0].data, "FullTimeString24");
        assert_eq!(columns[0].name, "");
        assert!(columns[0].searchable);
        assert!(!columns[0].orderable);

        // Last column: last trade date (not orderable duplicate).
        assert_eq!(columns[14].data, "LastComparibleTradeDate");
        assert_eq!(columns[14].name, "LastComparibleTradeDate");
        assert!(columns[14].searchable);
        assert!(!columns[14].orderable);
    }

    #[tokio::test]
    async fn get_trades_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trades_get_trades_response.json");
        let mock = crate::test_support::mock_json_post(&mut server, TRADES_PATH, &fixture).await;
        let client = test_client(&server);

        let response = client.get_trades(&TradesRequest::new()).await.unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 465);
        assert_eq!(response.records_filtered, 465);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AXP"));
        assert_eq!(response.data[1].ticker.as_deref(), Some("MRVL"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_trades_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trades_get_trades_response.json");
        crate::test_support::mock_json_post(&mut server, TRADES_PATH, &fixture).await;
        let client = test_client(&server);

        let trades = client
            .get_trades_limit(&TradesRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].ticker.as_deref(), Some("AXP"));
    }
}
