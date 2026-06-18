//! Trade level endpoints for `/Chart/GetTradeLevels`,
//! `/Chart0/GetTradeLevels`, `/TradeLevels/GetTradeLevels`, and
//! `/TradeLevelTouches/GetTradeLevelTouches` DataTables APIs.

use crate::datatables::{
    DataTablesColumn, DataTablesRequest, impl_datatables_client_methods,
    impl_datatables_request_methods,
};
use crate::models::TradeLevel;

/// Browser endpoint path for `/Chart/GetTradeLevels`.
pub(crate) const CHART_GET_TRADE_LEVELS_PATH: &str = "/Chart/GetTradeLevels";

/// Browser endpoint path for `/Chart0/GetTradeLevels`.
pub(crate) const CHART0_GET_TRADE_LEVELS_PATH: &str = "/Chart0/GetTradeLevels";

/// Browser endpoint path for `/TradeLevels/GetTradeLevels`.
pub(crate) const TRADE_LEVELS_GET_TRADE_LEVELS_PATH: &str = "/TradeLevels/GetTradeLevels";

/// Browser endpoint path for `/TradeLevelTouches/GetTradeLevelTouches`.
pub(crate) const TRADE_LEVEL_TOUCHES_PATH: &str = "/TradeLevelTouches/GetTradeLevelTouches";

/// Request parameters for trade level endpoints (`/Chart/GetTradeLevels`,
/// `/Chart0/GetTradeLevels`, `/TradeLevels/GetTradeLevels`).
///
/// Wraps a [`DataTablesRequest`] with pre-configured column definitions
/// matching the VolumeLeaders trade levels table.
#[derive(Clone, Debug)]
pub struct TradeLevelsRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(TradeLevelsRequest);

impl TradeLevelsRequest {
    /// Create a new trade levels request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: trade_levels_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Set chart filters for trade level retrieval.
    #[must_use]
    pub fn with_chart_filters(
        mut self,
        ticker: impl Into<String>,
        start: impl Into<String>,
        end: impl Into<String>,
        levels: usize,
    ) -> Self {
        self.0 = self.0.with_extra_values(vec![
            ("Ticker".to_string(), ticker.into()),
            ("StartDate".to_string(), start.into()),
            ("EndDate".to_string(), end.into()),
            ("Levels".to_string(), levels.to_string()),
        ]);
        self
    }
}

impl Default for TradeLevelsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request parameters for the `/TradeLevelTouches/GetTradeLevelTouches`
/// endpoint.
///
/// Wraps a [`DataTablesRequest`] with pre-configured column definitions
/// matching the VolumeLeaders trade level touches table.
#[derive(Clone, Debug)]
pub struct TradeLevelTouchesRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(TradeLevelTouchesRequest);

impl TradeLevelTouchesRequest {
    /// Create a new trade level touches request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: trade_level_touches_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Set endpoint filters for trade level touches.
    #[must_use]
    pub fn with_level_touch_filters(mut self, filters: Vec<(String, String)>) -> Self {
        self.0 = self.0.with_extra_values(filters);
        self
    }
}

impl Default for TradeLevelTouchesRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Return the DataTables column definitions for the trade levels table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`TradeLevelsColumns`) exactly.
#[must_use]
pub fn trade_levels_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("Price", "Price", true, true),
        DataTablesColumn::new("Dollars", "$$", true, true),
        DataTablesColumn::new("Volume", "Shares", true, true),
        DataTablesColumn::new("Trades", "Trades", true, true),
        DataTablesColumn::new("RelativeSize", "RS", true, true),
        DataTablesColumn::new("CumulativeDistribution", "PCT", true, true),
        DataTablesColumn::new("TradeLevelRank", "Level Rank", true, true),
        DataTablesColumn::new("Dates", "Level Date Range", true, false),
    ]
}

/// Return the DataTables column definitions for the trade level touches table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`TradeLevelTouchesColumns`) exactly.
#[must_use]
pub fn trade_level_touches_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("FullDateTime", "Date/Time", true, true),
        DataTablesColumn::new("Ticker", "Ticker", true, true),
        DataTablesColumn::new("Sector", "Sector", true, true),
        DataTablesColumn::new("Industry", "Industry", true, true),
        DataTablesColumn::new("Dollars", "$$", true, true),
        DataTablesColumn::new("Volume", "Shares", true, true),
        DataTablesColumn::new("Trades", "Trades", true, true),
        DataTablesColumn::new("Price", "Price", true, true),
        DataTablesColumn::new("RelativeSize", "RS", true, true),
        DataTablesColumn::new("CumulativeDistribution", "PCT", true, true),
        DataTablesColumn::new("TradeLevelRank", "Level Rank", true, true),
        DataTablesColumn::new("Dates", "Level Date Range", true, false),
        DataTablesColumn::new("", "", true, false),
    ]
}

impl_datatables_client_methods!(
    get_chart_trade_levels,
    get_chart_trade_levels_limit,
    TradeLevelsRequest,
    TradeLevel,
    CHART_GET_TRADE_LEVELS_PATH
);
impl_datatables_client_methods!(
    get_chart0_trade_levels,
    get_chart0_trade_levels_limit,
    TradeLevelsRequest,
    TradeLevel,
    CHART0_GET_TRADE_LEVELS_PATH
);
impl_datatables_client_methods!(
    get_trade_levels,
    get_trade_levels_limit,
    TradeLevelsRequest,
    TradeLevel,
    TRADE_LEVELS_GET_TRADE_LEVELS_PATH
);
impl_datatables_client_methods!(
    get_trade_level_touches,
    get_trade_level_touches_limit,
    TradeLevelTouchesRequest,
    TradeLevel,
    TRADE_LEVEL_TOUCHES_PATH
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_client;

    // -- column definition tests --

    #[test]
    fn trade_levels_columns_returns_8_columns() {
        let columns = trade_levels_columns();
        assert_eq!(columns.len(), 8);
    }

    #[test]
    fn trade_levels_columns_first_and_last_match_go_source() {
        let columns = trade_levels_columns();

        // First column: Price (orderable).
        assert_eq!(columns[0].data, "Price");
        assert_eq!(columns[0].name, "Price");
        assert!(columns[0].searchable);
        assert!(columns[0].orderable);

        // Last column: Dates (not orderable).
        assert_eq!(columns[7].data, "Dates");
        assert_eq!(columns[7].name, "Level Date Range");
        assert!(columns[7].searchable);
        assert!(!columns[7].orderable);
    }

    #[test]
    fn trade_levels_columns_rank_at_index_6() {
        let columns = trade_levels_columns();
        assert_eq!(columns[6].data, "TradeLevelRank");
        assert_eq!(columns[6].name, "Level Rank");
        assert!(columns[6].orderable);
    }

    #[test]
    fn trade_level_touches_columns_returns_13_columns() {
        let columns = trade_level_touches_columns();
        assert_eq!(columns.len(), 13);
    }

    #[test]
    fn trade_level_touches_columns_first_and_last_match_go_source() {
        let columns = trade_level_touches_columns();

        // First column: FullDateTime (orderable).
        assert_eq!(columns[0].data, "FullDateTime");
        assert_eq!(columns[0].name, "Date/Time");
        assert!(columns[0].searchable);
        assert!(columns[0].orderable);

        // Last column: trailing empty action column (not orderable).
        assert_eq!(columns[12].data, "");
        assert_eq!(columns[12].name, "");
        assert!(columns[12].searchable);
        assert!(!columns[12].orderable);
    }

    #[test]
    fn trade_level_touches_columns_rank_at_index_10() {
        let columns = trade_level_touches_columns();
        assert_eq!(columns[10].data, "TradeLevelRank");
        assert_eq!(columns[10].name, "Level Rank");
        assert!(columns[10].orderable);
    }

    // -- endpoint tests --

    #[tokio::test]
    async fn get_chart_trade_levels_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_levels_response.json");
        let mock = server
            .mock("POST", CHART_GET_TRADE_LEVELS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_chart_trade_levels(&TradeLevelsRequest::new())
            .await
            .unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 25);
        assert_eq!(response.records_filtered, 25);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AMD"));
        assert_eq!(response.data[0].trade_level_rank, Some(6));
        assert_eq!(response.data[1].ticker.as_deref(), Some("AAPL"));
        assert_eq!(response.data[1].trade_level_touches, Some(7));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_chart_trade_levels_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_levels_response.json");
        server
            .mock("POST", CHART_GET_TRADE_LEVELS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let levels = client
            .get_chart_trade_levels_limit(&TradeLevelsRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].ticker.as_deref(), Some("AMD"));
    }

    #[tokio::test]
    async fn get_chart0_trade_levels_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_levels_response.json");
        let mock = server
            .mock("POST", CHART0_GET_TRADE_LEVELS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_chart0_trade_levels(&TradeLevelsRequest::new())
            .await
            .unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AMD"));
        assert_eq!(
            response.data[0].price,
            Some(rust_decimal::Decimal::try_from(185.30).unwrap())
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_chart0_trade_levels_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_levels_response.json");
        server
            .mock("POST", CHART0_GET_TRADE_LEVELS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let levels = client
            .get_chart0_trade_levels_limit(&TradeLevelsRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].ticker.as_deref(), Some("AMD"));
    }

    #[tokio::test]
    async fn get_trade_levels_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_levels_response.json");
        let mock = server
            .mock("POST", TRADE_LEVELS_GET_TRADE_LEVELS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_trade_levels(&TradeLevelsRequest::new())
            .await
            .unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 25);
        assert_eq!(response.records_filtered, 25);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AMD"));
        assert_eq!(response.data[0].trade_level_rank, Some(6));
        assert_eq!(response.data[0].trades, Some(42));
        assert_eq!(response.data[1].ticker.as_deref(), Some("AAPL"));
        assert_eq!(response.data[1].trade_level_rank, Some(3));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_trade_levels_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_levels_response.json");
        server
            .mock("POST", TRADE_LEVELS_GET_TRADE_LEVELS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let levels = client
            .get_trade_levels_limit(&TradeLevelsRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0].ticker.as_deref(), Some("AMD"));
    }

    #[tokio::test]
    async fn get_trade_level_touches_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_level_touches_response.json");
        let mock = server
            .mock("POST", TRADE_LEVEL_TOUCHES_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_trade_level_touches(&TradeLevelTouchesRequest::new())
            .await
            .unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 18);
        assert_eq!(response.records_filtered, 18);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AMD"));
        assert_eq!(response.data[0].trade_level_rank, Some(6));
        assert_eq!(response.data[0].trade_level_touches, Some(1));
        assert_eq!(response.data[1].ticker.as_deref(), Some("NVDA"));
        assert_eq!(response.data[1].trade_level_rank, Some(2));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_trade_level_touches_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_level_touches_response.json");
        server
            .mock("POST", TRADE_LEVEL_TOUCHES_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let touches = client
            .get_trade_level_touches_limit(&TradeLevelTouchesRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(touches.len(), 1);
        assert_eq!(touches[0].ticker.as_deref(), Some("AMD"));
    }
}
