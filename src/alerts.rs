//! Alert configuration and alert DataTables endpoints.

use serde::Serialize;
use tracing::instrument;

use crate::client::{Client, FormPairs, config_to_form_pairs, multipart_form_from_fields};
use crate::datatables::{
    DataTablesColumn, DataTablesRequest, define_datatables_request, impl_datatables_client_methods,
};
use crate::error::Result;
use crate::models::{AlertConfig, TradeAlert, TradeClusterAlert};

/// Browser endpoint path for saving alert configurations.
pub(crate) const ALERT_CONFIG_PATH: &str = "/AlertConfig";

/// Browser endpoint path for alert configuration DataTables rows.
pub(crate) const ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH: &str = "/AlertConfigs/GetAlertConfigs";

/// Browser endpoint path for deleting alert configurations.
pub(crate) const ALERT_CONFIGS_DELETE_ALERT_CONFIG_PATH: &str = "/AlertConfigs/DeleteAlertConfig";

/// Browser endpoint path for trade alert DataTables rows.
pub(crate) const TRADE_ALERTS_GET_TRADE_ALERTS_PATH: &str = "/TradeAlerts/GetTradeAlerts";

/// Browser endpoint path for trade cluster alert DataTables rows.
pub(crate) const TRADE_CLUSTER_ALERTS_GET_TRADE_CLUSTER_ALERTS_PATH: &str =
    "/TradeClusterAlerts/GetTradeClusterAlerts";

/// Redirect path VolumeLeaders uses after a successful alert configuration save.
const ALERT_CONFIGS_SUCCESS_REDIRECT: &str = "/AlertConfigs";

define_datatables_request!(
    /// Request parameters for `/AlertConfigs/GetAlertConfigs`.
    AlertConfigsRequest,
    alert_configs_columns
);

define_datatables_request!(
    /// Request parameters for `/TradeAlerts/GetTradeAlerts`.
    TradeAlertsRequest,
    trade_alerts_columns
);

impl TradeAlertsRequest {
    /// Set the alert date filter.
    #[must_use]
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.0 = self.0.with_extra_value("Date", date);
        self
    }
}

define_datatables_request!(
    /// Request parameters for `/TradeClusterAlerts/GetTradeClusterAlerts`.
    TradeClusterAlertsRequest,
    trade_cluster_alerts_columns
);

impl TradeClusterAlertsRequest {
    /// Set the alert date filter.
    #[must_use]
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.0 = self.0.with_extra_value("Date", date);
        self
    }
}

/// Multipart form payload for creating or editing an alert configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SaveAlertConfigRequest {
    /// Raw browser field names and values accepted by VolumeLeaders.
    fields: FormPairs,
}

/// Typed values for creating or editing an alert configuration.
///
/// Field renames match the VolumeLeaders browser form key names. Serde
/// serialization drives the form-pair conversion with ASP.NET checkbox
/// handling for booleans.
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SaveAlertConfigFields {
    pub alert_config_key: i64,
    pub name: String,
    pub ticker_group: String,
    pub tickers: String,
    #[serde(rename = "TradeRankLTE")]
    pub trade_rank_lte: i64,
    #[serde(rename = "TradeVCDGTE")]
    pub trade_vcd_gte: i64,
    #[serde(rename = "TradeMultGTE")]
    pub trade_mult_gte: i64,
    #[serde(rename = "TradeVolumeGTE")]
    pub trade_volume_gte: i64,
    #[serde(rename = "TradeDollarsGTE")]
    pub trade_dollars_gte: i64,
    pub trade_conditions: String,
    pub dark_pool: bool,
    pub sweep: bool,
    #[serde(rename = "ClosingTradeRankLTE")]
    pub closing_trade_rank_lte: i64,
    #[serde(rename = "ClosingTradeVCDGTE")]
    pub closing_trade_vcd_gte: i64,
    #[serde(rename = "ClosingTradeMultGTE")]
    pub closing_trade_mult_gte: i64,
    #[serde(rename = "ClosingTradeVolumeGTE")]
    pub closing_trade_volume_gte: i64,
    #[serde(rename = "ClosingTradeDollarsGTE")]
    pub closing_trade_dollars_gte: i64,
    pub closing_trade_conditions: String,
    #[serde(rename = "TradeClusterRankLTE")]
    pub cluster_rank_lte: i64,
    #[serde(rename = "TradeClusterVCDGTE")]
    pub cluster_vcd_gte: i64,
    #[serde(rename = "TradeClusterMultGTE")]
    pub cluster_mult_gte: i64,
    #[serde(rename = "TradeClusterVolumeGTE")]
    pub cluster_volume_gte: i64,
    #[serde(rename = "TradeClusterDollarsGTE")]
    pub cluster_dollars_gte: i64,
    #[serde(rename = "TotalRankLTE")]
    pub total_rank_lte: i64,
    #[serde(rename = "TotalVolumeGTE")]
    pub total_volume_gte: i64,
    #[serde(rename = "TotalDollarsGTE")]
    pub total_dollars_gte: i64,
    #[serde(rename = "AHRankLTE")]
    pub ah_rank_lte: i64,
    #[serde(rename = "AHVolumeGTE")]
    pub ah_volume_gte: i64,
    #[serde(rename = "AHDollarsGTE")]
    pub ah_dollars_gte: i64,
    pub offsetting_print: bool,
    pub phantom_print: bool,
}

impl SaveAlertConfigRequest {
    /// Create a save request from captured browser form fields in client tests.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn new(fields: FormPairs) -> Self {
        Self { fields }
    }

    /// Return the encoded browser form fields for assertions and submission.
    #[must_use]
    pub fn fields(&self) -> &[(String, String)] {
        &self.fields
    }

    /// Create a save request from typed alert configuration values.
    ///
    /// Field names and boolean handling are driven by the serde `Serialize`
    /// derive on [`SaveAlertConfigFields`], so adding a new field to the
    /// struct is all that is needed — no manual mapping to update.
    #[must_use]
    pub fn from_config(config: SaveAlertConfigFields) -> Self {
        Self {
            fields: config_to_form_pairs(&config),
        }
    }
}

/// JSON payload for deleting an alert configuration.
#[allow(missing_docs)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteAlertConfigRequest {
    /// Primary key of the alert configuration to delete.
    pub alert_config_key: i64,
}

/// Return the DataTables column definitions for alert configurations.
#[must_use]
pub fn alert_configs_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("Name", "", true, false),
        DataTablesColumn::new("Name", "Name", true, true),
        DataTablesColumn::new("Tickers", "Tickers", true, true),
        DataTablesColumn::new("Conditions", "Conditions", true, false),
    ]
}

/// Return the DataTables column definitions for trade alerts.
#[must_use]
pub fn trade_alerts_columns() -> Vec<DataTablesColumn> {
    crate::trades::trades_columns()
}

/// Return the DataTables column definitions for trade cluster alerts.
#[must_use]
pub fn trade_cluster_alerts_columns() -> Vec<DataTablesColumn> {
    crate::clusters::trade_clusters_columns()
}

impl_datatables_client_methods!(
    get_alert_configs,
    get_alert_configs_limit,
    AlertConfigsRequest,
    AlertConfig,
    ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH
);
impl_datatables_client_methods!(
    get_trade_alerts,
    get_trade_alerts_limit,
    TradeAlertsRequest,
    TradeAlert,
    TRADE_ALERTS_GET_TRADE_ALERTS_PATH
);
impl_datatables_client_methods!(
    get_trade_cluster_alerts,
    get_trade_cluster_alerts_limit,
    TradeClusterAlertsRequest,
    TradeClusterAlert,
    TRADE_CLUSTER_ALERTS_GET_TRADE_CLUSTER_ALERTS_PATH
);

impl Client {
    /// Post a multipart create or edit request to `/AlertConfig`.
    #[instrument(skip_all)]
    pub async fn save_alert_config(&self, request: SaveAlertConfigRequest) -> Result<()> {
        self.post_multipart_form(
            ALERT_CONFIG_PATH,
            multipart_form_from_fields(request.fields()),
            ALERT_CONFIGS_SUCCESS_REDIRECT,
        )
        .await
    }

    /// Post a JSON delete request to `/AlertConfigs/DeleteAlertConfig`.
    #[instrument(skip_all)]
    pub async fn delete_alert_config(&self, request: &DeleteAlertConfigRequest) -> Result<()> {
        self.post_json(ALERT_CONFIGS_DELETE_ALERT_CONFIG_PATH, request)
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{datatables_body, test_client};

    #[test]
    fn alert_configs_columns_match_go_source() {
        let columns = alert_configs_columns();

        assert_eq!(columns.len(), 4);
        assert_eq!(columns[0], DataTablesColumn::new("Name", "", true, false));
        assert_eq!(
            columns[1],
            DataTablesColumn::new("Name", "Name", true, true)
        );
        assert_eq!(
            columns[2],
            DataTablesColumn::new("Tickers", "Tickers", true, true)
        );
        assert_eq!(
            columns[3],
            DataTablesColumn::new("Conditions", "Conditions", true, false)
        );
    }

    #[test]
    fn alert_trade_columns_reuse_captured_trade_layouts() {
        assert_eq!(trade_alerts_columns(), crate::trades::trades_columns());
        assert_eq!(
            trade_cluster_alerts_columns(),
            crate::clusters::trade_clusters_columns()
        );
    }

    #[tokio::test]
    async fn get_alert_configs_posts_datatables_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH)
            .match_header(
                "content-type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .match_body(mockito::Matcher::Regex(
                r"(?:^|&)columns\[0\]\[data\]=Name(?:&|.*&)columns\[2\]\[data\]=Tickers(?:&|.*&)columns\[3\]\[data\]=Conditions"
                    .to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"draw":1,"recordsTotal":1,"recordsFiltered":1,"data":[{"AlertConfigKey":42088,"Name":"testing 2","Tickers":"[ALL TICKERS]","TradeConditions":null}]}"#)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_alert_configs(&AlertConfigsRequest::new())
            .await
            .unwrap();

        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].alert_config_key, Some(42088));
        assert_eq!(response.data[0].trade_conditions, None);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_trade_alerts_posts_datatables_request() {
        let mut server = mockito::Server::new_async().await;
        let mut request = TradeAlertsRequest::new();
        request
            .0
            .extra_values
            .push(("Date".to_string(), "2026-05-07".to_string()));
        let mock = server
            .mock("POST", TRADE_ALERTS_GET_TRADE_ALERTS_PATH)
            .match_body(mockito::Matcher::Regex(
                r"(?:^|&)columns\[0\]\[data\]=FullTimeString24(?:&|.*&)columns\[4\]\[data\]=Trade(?:&|.*&)Date=2026-05-07(?:&|$)"
                    .to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"draw":1,"recordsTotal":1,"recordsFiltered":1,"data":[{"Ticker":"AMD","TradeID":123456,"AlertType":"Trade","Sweep":1}]}"#)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client.get_trade_alerts(&request).await.unwrap();

        assert_eq!(response.data[0].trade_id, Some(123456));
        assert_eq!(
            response.data[0].sweep,
            Some(crate::models::FlexBool(Some(true)))
        );
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_trade_cluster_alerts_posts_datatables_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", TRADE_CLUSTER_ALERTS_GET_TRADE_CLUSTER_ALERTS_PATH)
            .match_body(mockito::Matcher::Regex(
                r"(?:^|&)columns\[0\]\[data\]=MinFullTimeString24(?:&|.*&)columns\[12\]\[data\]=TradeClusterRank"
                    .to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"draw":1,"recordsTotal":1,"recordsFiltered":1,"data":[{"Ticker":"AMD","TradeClusterRank":8,"TradeCount":4}]}"#)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_trade_cluster_alerts(&TradeClusterAlertsRequest::new())
            .await
            .unwrap();

        assert_eq!(response.data[0].trade_cluster_rank, Some(8));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn alert_limit_methods_page_through_results() {
        let mut server = mockito::Server::new_async().await;
        crate::test_support::mock_json_post(&mut server, ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH, &datatables_body(vec![
            serde_json::json!({"AlertConfigKey": 1}),
        ])).await;
        crate::test_support::mock_json_post(&mut server, TRADE_ALERTS_GET_TRADE_ALERTS_PATH, &datatables_body(vec![serde_json::json!({"TradeID": 2})])).await;
        crate::test_support::mock_json_post(&mut server, TRADE_CLUSTER_ALERTS_GET_TRADE_CLUSTER_ALERTS_PATH, &datatables_body(vec![
            serde_json::json!({"TradeClusterRank": 3}),
        ])).await;
        let client = test_client(&server);

        let configs = client
            .get_alert_configs_limit(&AlertConfigsRequest::new(), 1)
            .await
            .unwrap();
        let trades = client
            .get_trade_alerts_limit(&TradeAlertsRequest::new(), 1)
            .await
            .unwrap();
        let clusters = client
            .get_trade_cluster_alerts_limit(&TradeClusterAlertsRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(configs[0].alert_config_key, Some(1));
        assert_eq!(trades[0].trade_id, Some(2));
        assert_eq!(clusters[0].trade_cluster_rank, Some(3));
    }

    #[tokio::test]
    async fn save_alert_config_posts_multipart_form_and_accepts_redirect() {
        let mut server = mockito::Server::new_async().await;
        let save = server
            .mock("POST", ALERT_CONFIG_PATH)
            .match_header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .match_body(mockito::Matcher::Regex(
                r#"(?s)name="AlertConfigKey"\r\n\r\n42089.*name="OffsettingPrint"\r\n\r\ntrue.*name="OffsettingPrint"\r\n\r\nfalse"#
                    .to_string(),
            ))
            .with_status(302)
            .with_header("location", "/AlertConfigs?ViewMode=Desktop")
            .create_async()
            .await;
        let follow = server
            .mock("GET", "/AlertConfigs?ViewMode=Desktop")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("saved")
            .create_async()
            .await;
        let client = test_client(&server);

        client
            .save_alert_config(SaveAlertConfigRequest::new(vec![
                ("AlertConfigKey".to_string(), "42089".to_string()),
                ("Name".to_string(), "Testing 2".to_string()),
                ("OffsettingPrint".to_string(), "true".to_string()),
                ("OffsettingPrint".to_string(), "false".to_string()),
            ]))
            .await
            .unwrap();

        save.assert_async().await;
        follow.assert_async().await;
    }

    #[tokio::test]
    async fn delete_alert_config_posts_json_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", ALERT_CONFIGS_DELETE_ALERT_CONFIG_PATH)
            .match_header("content-type", "application/json; charset=UTF-8")
            .match_header("x-requested-with", "XMLHttpRequest")
            .match_body(r#"{"AlertConfigKey":42088}"#)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("42088")
            .create_async()
            .await;
        let client = test_client(&server);

        client
            .delete_alert_config(&DeleteAlertConfigRequest {
                alert_config_key: 42088,
            })
            .await
            .unwrap();

        mock.assert_async().await;
    }
}
