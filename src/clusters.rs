//! Trade cluster endpoints for `/TradeClusters/GetTradeClusters` and
//! `/TradeClusterBombs/GetTradeClusterBombs` DataTables APIs.

use crate::datatables::{
    DataTablesColumn, DataTablesRequest, define_datatables_request, impl_datatables_client_methods,
};
use crate::models::{TradeCluster, TradeClusterBomb};

/// Browser endpoint path for trade clusters.
pub(crate) const TRADE_CLUSTERS_PATH: &str = "/TradeClusters/GetTradeClusters";

/// Browser endpoint path for trade cluster bombs.
pub(crate) const TRADE_CLUSTER_BOMBS_PATH: &str = "/TradeClusterBombs/GetTradeClusterBombs";

define_datatables_request!(
    /// Request parameters for the `/TradeClusters/GetTradeClusters` endpoint.
    ///
    /// Wraps a [`DataTablesRequest`] with pre-configured column definitions
    /// matching the VolumeLeaders trade clusters table.
    TradeClustersRequest,
    trade_clusters_columns
);

impl TradeClustersRequest {
    /// Set endpoint filters for the trade clusters table.
    #[must_use]
    pub fn with_cluster_filters(mut self, filters: Vec<(String, String)>) -> Self {
        self.0 = self.0.with_extra_values(filters);
        self
    }
}

define_datatables_request!(
    /// Request parameters for the `/TradeClusterBombs/GetTradeClusterBombs` endpoint.
    ///
    /// Wraps a [`DataTablesRequest`] with pre-configured column definitions
    /// matching the VolumeLeaders trade cluster bombs table.
    TradeClusterBombsRequest,
    trade_cluster_bombs_columns
);

impl TradeClusterBombsRequest {
    /// Set endpoint filters for the trade cluster bombs table.
    #[must_use]
    pub fn with_cluster_bomb_filters(mut self, filters: Vec<(String, String)>) -> Self {
        self.0 = self.0.with_extra_values(filters);
        self
    }
}

/// Return the DataTables column definitions for the trade clusters table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the browser request payload.
#[must_use]
pub fn trade_clusters_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("MinFullTimeString24", "", true, false),
        DataTablesColumn::id("MinFullTimeString24"),
        DataTablesColumn::id("Ticker"),
        DataTablesColumn::searchable("TradeCount", "Trades"),
        DataTablesColumn::new("Current", "Current", true, false),
        DataTablesColumn::new("Cluster", "Cluster", true, false),
        DataTablesColumn::id("Sector"),
        DataTablesColumn::id("Industry"),
        DataTablesColumn::searchable("Volume", "Sh"),
        DataTablesColumn::searchable("Dollars", "$$"),
        DataTablesColumn::searchable("DollarsMultiplier", "RS"),
        DataTablesColumn::searchable("CumulativeDistribution", "PCT"),
        DataTablesColumn::searchable("TradeClusterRank", "Rank"),
        DataTablesColumn::searchable("LastComparibleTradeClusterDate", "Last Date"),
        DataTablesColumn::new("LastComparibleTradeClusterDate", "Last Date", true, false),
    ]
}

/// Return the DataTables column definitions for the trade cluster bombs table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the browser request payload.
#[must_use]
pub fn trade_cluster_bombs_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("MinFullTimeString24", "", true, false),
        DataTablesColumn::new("MinFullTimeString24", "MinFullTimeString24", true, false),
        DataTablesColumn::id("Ticker"),
        DataTablesColumn::searchable("TradeCount", "Trades"),
        DataTablesColumn::id("Sector"),
        DataTablesColumn::id("Industry"),
        DataTablesColumn::searchable("Volume", "Sh"),
        DataTablesColumn::searchable("Dollars", "$$"),
        DataTablesColumn::searchable("DollarsMultiplier", "RS"),
        DataTablesColumn::searchable("CumulativeDistribution", "PCT"),
        DataTablesColumn::searchable("TradeClusterBombRank", "Rank"),
        DataTablesColumn::searchable("LastComparableTradeClusterBombDate", "Last Date"),
        DataTablesColumn::new("LastComparableTradeClusterBombDate", "Charts", true, false),
    ]
}

impl_datatables_client_methods!(
    get_trade_clusters,
    get_trade_clusters_limit,
    TradeClustersRequest,
    TradeCluster,
    TRADE_CLUSTERS_PATH
);
impl_datatables_client_methods!(
    get_trade_cluster_bombs,
    get_trade_cluster_bombs_limit,
    TradeClusterBombsRequest,
    TradeClusterBomb,
    TRADE_CLUSTER_BOMBS_PATH
);

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
    fn trade_clusters_columns_first_and_last_match_browser() {
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
    fn trade_cluster_bombs_columns_returns_13_columns() {
        let columns = trade_cluster_bombs_columns();
        assert_eq!(columns.len(), 13);
    }

    #[test]
    fn trade_cluster_bombs_columns_first_and_last_match_browser() {
        let columns = trade_cluster_bombs_columns();

        // First column: time display (not orderable).
        assert_eq!(columns[0].data, "MinFullTimeString24");
        assert_eq!(columns[0].name, "");
        assert!(columns[0].searchable);
        assert!(!columns[0].orderable);

        // Last column: chart links (not orderable duplicate).
        assert_eq!(columns[12].data, "LastComparableTradeClusterBombDate");
        assert_eq!(columns[12].name, "Charts");
        assert!(columns[12].searchable);
        assert!(!columns[12].orderable);
    }

    #[test]
    fn trade_cluster_bombs_columns_rank_at_index_9() {
        let columns = trade_cluster_bombs_columns();
        assert_eq!(columns[10].data, "TradeClusterBombRank");
        assert_eq!(columns[10].name, "Rank");
        assert!(columns[10].orderable);
    }

    // -- endpoint tests --

    #[tokio::test]
    async fn get_trade_clusters_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let fixture = crate::test_support::read_fixture("trade_clusters_response.json");
        let mock =
            crate::test_support::mock_json_post(&mut server, TRADE_CLUSTERS_PATH, &fixture).await;
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
        crate::test_support::mock_json_post(&mut server, TRADE_CLUSTERS_PATH, &fixture).await;
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
        let mock =
            crate::test_support::mock_json_post(&mut server, TRADE_CLUSTER_BOMBS_PATH, &fixture)
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
        crate::test_support::mock_json_post(&mut server, TRADE_CLUSTER_BOMBS_PATH, &fixture).await;
        let client = test_client(&server);

        let bombs = client
            .get_trade_cluster_bombs_limit(&TradeClusterBombsRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(bombs.len(), 1);
        assert_eq!(bombs[0].ticker.as_deref(), Some("AMD"));
    }
}
