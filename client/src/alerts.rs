//! Alert configuration and alert DataTables endpoints.

use serde::Serialize;
use tracing::instrument;

use crate::client::{Client, multipart_form_from_fields, push_bool_field};
use crate::datatables::{
    DataTablesColumn, DataTablesRequest, DataTablesResponse, fetch_limit,
    impl_datatables_request_methods,
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

/// Request parameters for `/AlertConfigs/GetAlertConfigs`.
#[derive(Clone, Debug)]
pub struct AlertConfigsRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(AlertConfigsRequest);

impl AlertConfigsRequest {
    /// Create an alert configs request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: alert_configs_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        self.0.to_pairs()
    }
}

impl Default for AlertConfigsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request parameters for `/TradeAlerts/GetTradeAlerts`.
#[derive(Clone, Debug)]
pub struct TradeAlertsRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(TradeAlertsRequest);

impl TradeAlertsRequest {
    /// Create a trade alerts request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: trade_alerts_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Set the alert date filter.
    #[must_use]
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.0 = self.0.with_extra_value("Date", date);
        self
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        self.0.to_pairs()
    }
}

impl Default for TradeAlertsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request parameters for `/TradeClusterAlerts/GetTradeClusterAlerts`.
#[derive(Clone, Debug)]
pub struct TradeClusterAlertsRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(TradeClusterAlertsRequest);

impl TradeClusterAlertsRequest {
    /// Create a trade cluster alerts request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: trade_cluster_alerts_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Set the alert date filter.
    #[must_use]
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.0 = self.0.with_extra_value("Date", date);
        self
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        self.0.to_pairs()
    }
}

impl Default for TradeClusterAlertsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Multipart form payload for creating or editing an alert configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SaveAlertConfigRequest {
    /// Raw browser field names and values accepted by VolumeLeaders.
    fields: Vec<(String, String)>,
}

/// Typed values for creating or editing an alert configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SaveAlertConfigFields {
    pub alert_config_key: i64,
    pub name: String,
    pub ticker_group: String,
    pub tickers: String,
    pub trade_rank_lte: i64,
    pub trade_vcd_gte: i64,
    pub trade_mult_gte: i64,
    pub trade_volume_gte: i64,
    pub trade_dollars_gte: i64,
    pub trade_conditions: String,
    pub dark_pool: bool,
    pub sweep: bool,
    pub closing_trade_rank_lte: i64,
    pub closing_trade_vcd_gte: i64,
    pub closing_trade_mult_gte: i64,
    pub closing_trade_volume_gte: i64,
    pub closing_trade_dollars_gte: i64,
    pub closing_trade_conditions: String,
    pub cluster_rank_lte: i64,
    pub cluster_vcd_gte: i64,
    pub cluster_mult_gte: i64,
    pub cluster_volume_gte: i64,
    pub cluster_dollars_gte: i64,
    pub total_rank_lte: i64,
    pub total_volume_gte: i64,
    pub total_dollars_gte: i64,
    pub ah_rank_lte: i64,
    pub ah_volume_gte: i64,
    pub ah_dollars_gte: i64,
    pub offsetting_print: bool,
    pub phantom_print: bool,
}

impl SaveAlertConfigRequest {
    /// Create a save request from captured browser form fields in client tests.
    #[must_use]
    #[cfg(test)]
    pub(crate) fn new(fields: Vec<(String, String)>) -> Self {
        Self { fields }
    }

    /// Return the encoded browser form fields for assertions and submission.
    #[must_use]
    pub fn fields(&self) -> &[(String, String)] {
        &self.fields
    }

    /// Create a save request from typed alert configuration values.
    #[must_use]
    pub fn from_config(config: SaveAlertConfigFields) -> Self {
        let mut fields = vec![
            ("AlertConfigKey".into(), config.alert_config_key.to_string()),
            ("Name".into(), config.name),
            ("TickerGroup".into(), config.ticker_group),
            ("Tickers".into(), config.tickers),
            ("TradeRankLTE".into(), config.trade_rank_lte.to_string()),
            ("TradeVCDGTE".into(), config.trade_vcd_gte.to_string()),
            ("TradeMultGTE".into(), config.trade_mult_gte.to_string()),
            ("TradeVolumeGTE".into(), config.trade_volume_gte.to_string()),
            (
                "TradeDollarsGTE".into(),
                config.trade_dollars_gte.to_string(),
            ),
            ("TradeConditions".into(), config.trade_conditions),
            (
                "ClosingTradeRankLTE".into(),
                config.closing_trade_rank_lte.to_string(),
            ),
            (
                "ClosingTradeVCDGTE".into(),
                config.closing_trade_vcd_gte.to_string(),
            ),
            (
                "ClosingTradeMultGTE".into(),
                config.closing_trade_mult_gte.to_string(),
            ),
            (
                "ClosingTradeVolumeGTE".into(),
                config.closing_trade_volume_gte.to_string(),
            ),
            (
                "ClosingTradeDollarsGTE".into(),
                config.closing_trade_dollars_gte.to_string(),
            ),
            (
                "ClosingTradeConditions".into(),
                config.closing_trade_conditions,
            ),
            (
                "TradeClusterRankLTE".into(),
                config.cluster_rank_lte.to_string(),
            ),
            (
                "TradeClusterVCDGTE".into(),
                config.cluster_vcd_gte.to_string(),
            ),
            (
                "TradeClusterMultGTE".into(),
                config.cluster_mult_gte.to_string(),
            ),
            (
                "TradeClusterVolumeGTE".into(),
                config.cluster_volume_gte.to_string(),
            ),
            (
                "TradeClusterDollarsGTE".into(),
                config.cluster_dollars_gte.to_string(),
            ),
            ("TotalRankLTE".into(), config.total_rank_lte.to_string()),
            ("TotalVolumeGTE".into(), config.total_volume_gte.to_string()),
            (
                "TotalDollarsGTE".into(),
                config.total_dollars_gte.to_string(),
            ),
            ("AHRankLTE".into(), config.ah_rank_lte.to_string()),
            ("AHVolumeGTE".into(), config.ah_volume_gte.to_string()),
            ("AHDollarsGTE".into(), config.ah_dollars_gte.to_string()),
        ];
        push_bool_field(&mut fields, "DarkPool", config.dark_pool);
        push_bool_field(&mut fields, "Sweep", config.sweep);
        push_bool_field(&mut fields, "OffsettingPrint", config.offsetting_print);
        push_bool_field(&mut fields, "PhantomPrint", config.phantom_print);
        Self { fields }
    }
}



/// JSON payload for deleting an alert configuration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteAlertConfigRequest {
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

impl Client {
    /// Post a DataTables request to `/AlertConfigs/GetAlertConfigs`.
    #[instrument(skip_all)]
    pub async fn get_alert_configs(
        &self,
        request: &AlertConfigsRequest,
    ) -> Result<DataTablesResponse<AlertConfig>> {
        self.post_datatables(ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH, request.to_pairs())
            .await
    }

    /// Fetch up to `limit` alert configurations by paginating the endpoint.
    #[instrument(skip_all)]
    pub async fn get_alert_configs_limit(
        &self,
        request: &AlertConfigsRequest,
        limit: usize,
    ) -> Result<Vec<AlertConfig>> {
        fetch_limit(
            self,
            ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH,
            request.0.clone(),
            limit,
        )
        .await
    }

    /// Post a DataTables request to `/TradeAlerts/GetTradeAlerts`.
    #[instrument(skip_all)]
    pub async fn get_trade_alerts(
        &self,
        request: &TradeAlertsRequest,
    ) -> Result<DataTablesResponse<TradeAlert>> {
        self.post_datatables(TRADE_ALERTS_GET_TRADE_ALERTS_PATH, request.to_pairs())
            .await
    }

    /// Fetch up to `limit` trade alerts by paginating the endpoint.
    #[instrument(skip_all)]
    pub async fn get_trade_alerts_limit(
        &self,
        request: &TradeAlertsRequest,
        limit: usize,
    ) -> Result<Vec<TradeAlert>> {
        fetch_limit(
            self,
            TRADE_ALERTS_GET_TRADE_ALERTS_PATH,
            request.0.clone(),
            limit,
        )
        .await
    }

    /// Post a DataTables request to `/TradeClusterAlerts/GetTradeClusterAlerts`.
    #[instrument(skip_all)]
    pub async fn get_trade_cluster_alerts(
        &self,
        request: &TradeClusterAlertsRequest,
    ) -> Result<DataTablesResponse<TradeClusterAlert>> {
        self.post_datatables(
                TRADE_CLUSTER_ALERTS_GET_TRADE_CLUSTER_ALERTS_PATH,
                request.to_pairs(),
            )
            .await
    }

    /// Fetch up to `limit` trade cluster alerts by paginating the endpoint.
    #[instrument(skip_all)]
    pub async fn get_trade_cluster_alerts_limit(
        &self,
        request: &TradeClusterAlertsRequest,
        limit: usize,
    ) -> Result<Vec<TradeClusterAlert>> {
        fetch_limit(
            self,
            TRADE_CLUSTER_ALERTS_GET_TRADE_CLUSTER_ALERTS_PATH,
            request.0.clone(),
            limit,
        )
        .await
    }

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
        server
            .mock("POST", ALERT_CONFIGS_GET_ALERT_CONFIGS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(datatables_body(vec![
                serde_json::json!({"AlertConfigKey": 1}),
            ]))
            .create_async()
            .await;
        server
            .mock("POST", TRADE_ALERTS_GET_TRADE_ALERTS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(datatables_body(vec![serde_json::json!({"TradeID": 2})]))
            .create_async()
            .await;
        server
            .mock("POST", TRADE_CLUSTER_ALERTS_GET_TRADE_CLUSTER_ALERTS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(datatables_body(vec![
                serde_json::json!({"TradeClusterRank": 3}),
            ]))
            .create_async()
            .await;
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
