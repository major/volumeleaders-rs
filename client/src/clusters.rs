//! Trade cluster endpoints for `/TradeClusters/GetTradeClusters` and
//! `/TradeClusterBombs/GetTradeClusterBombs` DataTables APIs.

use tracing::instrument;

use crate::client::Client;
use crate::datatables::{
    DataTablesColumn, DataTablesRequest, DataTablesResponse, fetch_limit,
    impl_datatables_request_methods,
};
use crate::error::Result;
use crate::models::{TradeCluster, TradeClusterBomb};

/// Browser endpoint path for trade clusters.
pub(crate) const TRADE_CLUSTERS_PATH: &str = "/TradeClusters/GetTradeClusters";

/// Browser endpoint path for trade cluster bombs.
pub(crate) const TRADE_CLUSTER_BOMBS_PATH: &str = "/TradeClusterBombs/GetTradeClusterBombs";

/// Request parameters for the `/TradeClusters/GetTradeClusters` endpoint.
///
/// Wraps a [`DataTablesRequest`] with pre-configured column definitions
/// matching the VolumeLeaders trade clusters table.
#[derive(Clone, Debug)]
pub struct TradeClustersRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(TradeClustersRequest);

impl TradeClustersRequest {
    /// Create a new trade clusters request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: trade_clusters_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Set endpoint filters for the trade clusters table.
    #[must_use]
    pub fn with_cluster_filters(mut self, filters: Vec<(String, String)>) -> Self {
        self.0 = self.0.with_extra_values(filters);
        self
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        self.0.to_pairs()
    }
}

impl Default for TradeClustersRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request parameters for the `/TradeClusterBombs/GetTradeClusterBombs` endpoint.
///
/// Wraps a [`DataTablesRequest`] with pre-configured column definitions
/// matching the VolumeLeaders trade cluster bombs table.
#[derive(Clone, Debug)]
pub struct TradeClusterBombsRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(TradeClusterBombsRequest);

impl TradeClusterBombsRequest {
    /// Create a new trade cluster bombs request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: trade_cluster_bombs_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Set endpoint filters for the trade cluster bombs table.
    #[must_use]
    pub fn with_cluster_bomb_filters(mut self, filters: Vec<(String, String)>) -> Self {
        self.0 = self.0.with_extra_values(filters);
        self
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        self.0.to_pairs()
    }
}

impl Default for TradeClusterBombsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Return the DataTables column definitions for the trade clusters table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`TradeClustersColumns`) exactly.
#[must_use]
pub fn trade_clusters_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("MinFullTimeString24", "", true, false),
        DataTablesColumn::new("MinFullTimeString24", "MinFullTimeString24", true, true),
        DataTablesColumn::new("Ticker", "Ticker", true, true),
        DataTablesColumn::new("TradeCount", "Trades", true, true),
        DataTablesColumn::new("Current", "Current", true, false),
        DataTablesColumn::new("Cluster", "Cluster", true, false),
        DataTablesColumn::new("Sector", "Sector", true, true),
        DataTablesColumn::new("Industry", "Industry", true, true),
        DataTablesColumn::new("Volume", "Sh", true, true),
        DataTablesColumn::new("Dollars", "$$", true, true),
        DataTablesColumn::new("DollarsMultiplier", "RS", true, true),
        DataTablesColumn::new("CumulativeDistribution", "PCT", true, true),
        DataTablesColumn::new("TradeClusterRank", "Rank", true, true),
        DataTablesColumn::new("LastComparibleTradeClusterDate", "Last Date", true, true),
        DataTablesColumn::new("LastComparibleTradeClusterDate", "Last Date", true, false),
    ]
}

/// Return the DataTables column definitions for the trade cluster bombs table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`TradeClusterBombsColumns`) exactly.
#[must_use]
pub fn trade_cluster_bombs_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("MinFullTimeString24", "MinFullTimeString24", true, true),
        DataTablesColumn::new("Ticker", "Ticker", true, true),
        DataTablesColumn::new("TradeCount", "Trades", true, true),
        DataTablesColumn::new("Sector", "Sector", true, true),
        DataTablesColumn::new("Industry", "Industry", true, true),
        DataTablesColumn::new("Volume", "Sh", true, true),
        DataTablesColumn::new("Dollars", "$$", true, true),
        DataTablesColumn::new("DollarsMultiplier", "RS", true, true),
        DataTablesColumn::new("CumulativeDistribution", "PCT", true, true),
        DataTablesColumn::new("TradeClusterBombRank", "Rank", true, true),
        DataTablesColumn::new(
            "LastComparableTradeClusterBombDate",
            "Last Date",
            true,
            true,
        ),
        DataTablesColumn::new(
            "LastComparableTradeClusterBombDate",
            "Last Date",
            true,
            false,
        ),
    ]
}

impl Client {
    /// Post a DataTables request to `/TradeClusters/GetTradeClusters` and
    /// return the typed response envelope.
    #[instrument(skip_all)]
    pub async fn get_trade_clusters(
        &self,
        request: &TradeClustersRequest,
    ) -> Result<DataTablesResponse<TradeCluster>> {
        self.post_datatables(TRADE_CLUSTERS_PATH, request.to_pairs())
            .await
    }

    /// Fetch up to `limit` trade clusters by paginating
    /// `/TradeClusters/GetTradeClusters`.
    #[instrument(skip_all)]
    pub async fn get_trade_clusters_limit(
        &self,
        request: &TradeClustersRequest,
        limit: usize,
    ) -> Result<Vec<TradeCluster>> {
        fetch_limit(self, TRADE_CLUSTERS_PATH, request.0.clone(), limit).await
    }

    /// Post a DataTables request to
    /// `/TradeClusterBombs/GetTradeClusterBombs` and return the typed
    /// response envelope.
    #[instrument(skip_all)]
    pub async fn get_trade_cluster_bombs(
        &self,
        request: &TradeClusterBombsRequest,
    ) -> Result<DataTablesResponse<TradeClusterBomb>> {
        self.post_datatables(TRADE_CLUSTER_BOMBS_PATH, request.to_pairs())
            .await
    }

    /// Fetch up to `limit` trade cluster bombs by paginating
    /// `/TradeClusterBombs/GetTradeClusterBombs`.
    #[instrument(skip_all)]
    pub async fn get_trade_cluster_bombs_limit(
        &self,
        request: &TradeClusterBombsRequest,
        limit: usize,
    ) -> Result<Vec<TradeClusterBomb>> {
        fetch_limit(self, TRADE_CLUSTER_BOMBS_PATH, request.0.clone(), limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_client;

    // -- column definition tests --

    #[test]
    fn trade_clusters_columns_returns_15_columns() {
        let columns = trade_clusters_columns();
        assert_eq!(columns.len(), 15);
    }

    #[test]
    fn trade_clusters_columns_first_and_last_match_go_source() {
        let columns = trade_clusters_columns();

        // First column: time display (not orderable).
        assert_eq!(columns[0].data, "MinFullTimeString24");
        assert_eq!(columns[0].name, "");
        assert!(columns[0].searchable);
        assert!(!columns[0].orderable);

        // Last column: last cluster date (not orderable duplicate).
        assert_eq!(columns[14].data, "LastComparibleTradeClusterDate");
        assert_eq!(columns[14].name, "Last Date");
        assert!(columns[14].searchable);
        assert!(!columns[14].orderable);
    }

    #[test]
    fn trade_clusters_columns_rank_at_index_12() {
        let columns = trade_clusters_columns();
        assert_eq!(columns[12].data, "TradeClusterRank");
        assert_eq!(columns[12].name, "Rank");
        assert!(columns[12].orderable);
    }

    #[test]
    fn trade_cluster_bombs_columns_returns_12_columns() {
        let columns = trade_cluster_bombs_columns();
        assert_eq!(columns.len(), 12);
    }

    #[test]
    fn trade_cluster_bombs_columns_first_and_last_match_go_source() {
        let columns = trade_cluster_bombs_columns();

        // First column: time (orderable).
        assert_eq!(columns[0].data, "MinFullTimeString24");
        assert_eq!(columns[0].name, "MinFullTimeString24");
        assert!(columns[0].searchable);
        assert!(columns[0].orderable);

        // Last column: last bomb date (not orderable duplicate).
        assert_eq!(columns[11].data, "LastComparableTradeClusterBombDate");
        assert_eq!(columns[11].name, "Last Date");
        assert!(columns[11].searchable);
        assert!(!columns[11].orderable);
    }

    #[test]
    fn trade_cluster_bombs_columns_rank_at_index_9() {
        let columns = trade_cluster_bombs_columns();
        assert_eq!(columns[9].data, "TradeClusterBombRank");
        assert_eq!(columns[9].name, "Rank");
        assert!(columns[9].orderable);
    }

    // -- endpoint tests --

    #[tokio::test]
    async fn get_trade_clusters_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_clusters_response.json");
        let mock = server
            .mock("POST", TRADE_CLUSTERS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_trade_clusters(&TradeClustersRequest::new())
            .await
            .unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 85);
        assert_eq!(response.records_filtered, 85);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AAPL"));
        assert_eq!(response.data[0].trade_cluster_rank, Some(7));
        assert_eq!(response.data[1].ticker.as_deref(), Some("AMD"));
        assert_eq!(response.data[1].trade_count, Some(4));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_trade_clusters_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_clusters_response.json");
        server
            .mock("POST", TRADE_CLUSTERS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let clusters = client
            .get_trade_clusters_limit(&TradeClustersRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].ticker.as_deref(), Some("AAPL"));
    }

    #[tokio::test]
    async fn get_trade_cluster_bombs_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_cluster_bombs_response.json");
        let mock = server
            .mock("POST", TRADE_CLUSTER_BOMBS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_trade_cluster_bombs(&TradeClusterBombsRequest::new())
            .await
            .unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 42);
        assert_eq!(response.records_filtered, 42);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AMD"));
        assert_eq!(response.data[0].trade_cluster_bomb_rank, Some(4));
        assert_eq!(response.data[1].ticker.as_deref(), Some("NVDA"));
        assert_eq!(response.data[1].trade_count, Some(8));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_trade_cluster_bombs_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_cluster_bombs_response.json");
        server
            .mock("POST", TRADE_CLUSTER_BOMBS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&fixture)
            .create_async()
            .await;
        let client = test_client(&server);

        let bombs = client
            .get_trade_cluster_bombs_limit(&TradeClusterBombsRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(bombs.len(), 1);
        assert_eq!(bombs[0].ticker.as_deref(), Some("AMD"));
    }
}
