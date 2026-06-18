//! Volume leaderboard endpoints for institutional, after-hours, and total volume.

use crate::datatables::{
    DataTablesColumn, DataTablesRequest, impl_datatables_client_methods,
    impl_datatables_request_methods,
};
use crate::models::Trade;

/// Browser endpoint path for institutional volume.
pub(crate) const INSTITUTIONAL_VOLUME_PATH: &str = "/InstitutionalVolume/GetInstitutionalVolume";

/// Browser endpoint path for after-hours institutional volume.
pub(crate) const AH_INSTITUTIONAL_VOLUME_PATH: &str =
    "/AHInstitutionalVolume/GetAHInstitutionalVolume";

/// Browser endpoint path for total volume.
pub(crate) const TOTAL_VOLUME_PATH: &str = "/TotalVolume/GetTotalVolume";

/// Request parameters for volume leaderboard endpoints.
///
/// Wraps a [`DataTablesRequest`] with pre-configured column definitions
/// matching one of the three VolumeLeaders volume tables. Use the named
/// constructors to select the correct column set:
///
/// - [`VolumeRequest::institutional`] for institutional volume
/// - [`VolumeRequest::ah_institutional`] for after-hours institutional volume
/// - [`VolumeRequest::total`] for total volume
#[derive(Clone, Debug)]
pub struct VolumeRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(VolumeRequest);

impl VolumeRequest {
    /// Create a request for the institutional volume endpoint.
    #[must_use]
    pub fn institutional() -> Self {
        Self(DataTablesRequest {
            columns: institutional_volume_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Create a request for the after-hours institutional volume endpoint.
    #[must_use]
    pub fn ah_institutional() -> Self {
        Self(DataTablesRequest {
            columns: ah_institutional_volume_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Create a request for the total volume endpoint.
    #[must_use]
    pub fn total() -> Self {
        Self(DataTablesRequest {
            columns: total_volume_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Set the VolumeLeaders date filter.
    #[must_use]
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.0 = self.0.with_extra_value("Date", date);
        self
    }

    /// Set the optional comma-separated ticker filter.
    #[must_use]
    pub fn with_tickers(mut self, tickers: impl Into<String>) -> Self {
        self.0 = self.0.with_extra_value("Tickers", tickers);
        self
    }
}

/// Build the shared volume column layout.
///
/// All three volume endpoints share the same leading columns (Ticker x2,
/// Price, Sector, Industry) and trailing columns (LastComparibleTradeDate x2).
/// Only the three middle metric columns differ. The duplicated Ticker and
/// LastComparibleTradeDate entries match the browser form that VolumeLeaders
/// expects.
fn volume_columns(volume: &str, dollars: &str, rank: &str) -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("Ticker", "Ticker", true, true),
        DataTablesColumn::new("Ticker", "Ticker", true, true),
        DataTablesColumn::new("Price", "Price", true, true),
        DataTablesColumn::new("Sector", "Sector", true, true),
        DataTablesColumn::new("Industry", "Industry", true, true),
        DataTablesColumn::new(volume, volume, true, true),
        DataTablesColumn::new(dollars, dollars, true, true),
        DataTablesColumn::new(rank, rank, true, true),
        DataTablesColumn::new(
            "LastComparibleTradeDate",
            "LastComparibleTradeDate",
            true,
            true,
        ),
        DataTablesColumn::new(
            "LastComparibleTradeDate",
            "LastComparibleTradeDate",
            true,
            true,
        ),
    ]
}

/// Return the DataTables column definitions for the institutional volume table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`InstitutionalVolumeColumns`) exactly.
#[must_use]
pub fn institutional_volume_columns() -> Vec<DataTablesColumn> {
    volume_columns(
        "TotalInstitutionalVolume",
        "TotalInstitutionalDollars",
        "TotalInstitutionalDollarsRank",
    )
}

/// Return the DataTables column definitions for the after-hours institutional
/// volume table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`AHInstitutionalVolumeColumns`) exactly.
#[must_use]
pub fn ah_institutional_volume_columns() -> Vec<DataTablesColumn> {
    volume_columns(
        "AHInstitutionalVolume",
        "AHInstitutionalDollars",
        "AHInstitutionalDollarsRank",
    )
}

/// Return the DataTables column definitions for the total volume table.
///
/// Column order, `Data`/`Name` field values, and `Searchable`/`Orderable`
/// flags match the Go source (`TotalVolumeColumns`) exactly.
#[must_use]
pub fn total_volume_columns() -> Vec<DataTablesColumn> {
    volume_columns("TotalVolume", "TotalDollars", "TotalDollarsRank")
}

impl_datatables_client_methods!(
    get_institutional_volume,
    get_institutional_volume_limit,
    VolumeRequest,
    Trade,
    INSTITUTIONAL_VOLUME_PATH
);
impl_datatables_client_methods!(
    get_ah_institutional_volume,
    get_ah_institutional_volume_limit,
    VolumeRequest,
    Trade,
    AH_INSTITUTIONAL_VOLUME_PATH
);
impl_datatables_client_methods!(
    get_total_volume,
    get_total_volume_limit,
    VolumeRequest,
    Trade,
    TOTAL_VOLUME_PATH
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_client;

    fn volume_fixture() -> String {
        crate::test_support::read_fixture("volume_response.json")
    }

    // -- column definition tests --

    #[test]
    fn institutional_volume_columns_returns_10_columns() {
        assert_eq!(institutional_volume_columns().len(), 10);
    }

    #[test]
    fn ah_institutional_volume_columns_returns_10_columns() {
        assert_eq!(ah_institutional_volume_columns().len(), 10);
    }

    #[test]
    fn total_volume_columns_returns_10_columns() {
        assert_eq!(total_volume_columns().len(), 10);
    }

    #[test]
    fn volume_columns_share_leading_and_trailing_layout() {
        for columns in [
            institutional_volume_columns(),
            ah_institutional_volume_columns(),
            total_volume_columns(),
        ] {
            // Leading: Ticker x2, Price, Sector, Industry
            assert_eq!(columns[0].data, "Ticker");
            assert_eq!(columns[1].data, "Ticker");
            assert_eq!(columns[2].data, "Price");
            assert_eq!(columns[3].data, "Sector");
            assert_eq!(columns[4].data, "Industry");

            // Trailing: LastComparibleTradeDate x2
            assert_eq!(columns[8].data, "LastComparibleTradeDate");
            assert_eq!(columns[9].data, "LastComparibleTradeDate");

            // All columns are searchable and orderable
            for col in &columns {
                assert!(col.searchable);
                assert!(col.orderable);
            }
        }
    }

    #[test]
    fn institutional_volume_columns_middle_fields_match_go_source() {
        let columns = institutional_volume_columns();
        assert_eq!(columns[5].data, "TotalInstitutionalVolume");
        assert_eq!(columns[6].data, "TotalInstitutionalDollars");
        assert_eq!(columns[7].data, "TotalInstitutionalDollarsRank");
    }

    #[test]
    fn ah_institutional_volume_columns_middle_fields_match_go_source() {
        let columns = ah_institutional_volume_columns();
        assert_eq!(columns[5].data, "AHInstitutionalVolume");
        assert_eq!(columns[6].data, "AHInstitutionalDollars");
        assert_eq!(columns[7].data, "AHInstitutionalDollarsRank");
    }

    #[test]
    fn total_volume_columns_middle_fields_match_go_source() {
        let columns = total_volume_columns();
        assert_eq!(columns[5].data, "TotalVolume");
        assert_eq!(columns[6].data, "TotalDollars");
        assert_eq!(columns[7].data, "TotalDollarsRank");
    }

    // -- endpoint tests --

    #[tokio::test]
    async fn get_institutional_volume_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = crate::test_support::mock_json_post(&mut server, INSTITUTIONAL_VOLUME_PATH, &volume_fixture()).await;
        let client = test_client(&server);

        let response = client
            .get_institutional_volume(&VolumeRequest::institutional())
            .await
            .unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 120);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AAPL"));
        assert_eq!(response.data[1].ticker.as_deref(), Some("MSFT"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_institutional_volume_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        crate::test_support::mock_json_post(&mut server, INSTITUTIONAL_VOLUME_PATH, &volume_fixture()).await;
        let client = test_client(&server);

        let trades = client
            .get_institutional_volume_limit(&VolumeRequest::institutional(), 1)
            .await
            .unwrap();

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].ticker.as_deref(), Some("AAPL"));
    }

    #[tokio::test]
    async fn get_ah_institutional_volume_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = crate::test_support::mock_json_post(&mut server, AH_INSTITUTIONAL_VOLUME_PATH, &volume_fixture()).await;
        let client = test_client(&server);

        let response = client
            .get_ah_institutional_volume(&VolumeRequest::ah_institutional())
            .await
            .unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 120);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AAPL"));
        assert_eq!(response.data[1].ticker.as_deref(), Some("MSFT"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_ah_institutional_volume_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        crate::test_support::mock_json_post(&mut server, AH_INSTITUTIONAL_VOLUME_PATH, &volume_fixture()).await;
        let client = test_client(&server);

        let trades = client
            .get_ah_institutional_volume_limit(&VolumeRequest::ah_institutional(), 1)
            .await
            .unwrap();

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].ticker.as_deref(), Some("AAPL"));
    }

    #[tokio::test]
    async fn get_total_volume_returns_fixture_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = crate::test_support::mock_json_post(&mut server, TOTAL_VOLUME_PATH, &volume_fixture()).await;
        let client = test_client(&server);

        let response = client
            .get_total_volume(&VolumeRequest::total())
            .await
            .unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 120);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].ticker.as_deref(), Some("AAPL"));
        assert_eq!(response.data[1].ticker.as_deref(), Some("MSFT"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_total_volume_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        crate::test_support::mock_json_post(&mut server, TOTAL_VOLUME_PATH, &volume_fixture()).await;
        let client = test_client(&server);

        let trades = client
            .get_total_volume_limit(&VolumeRequest::total(), 1)
            .await
            .unwrap();

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].ticker.as_deref(), Some("AAPL"));
    }
}
