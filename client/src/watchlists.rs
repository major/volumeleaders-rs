//! Watchlist configuration and ticker DataTables endpoints.

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::client::{Client, multipart_form_from_fields, push_bool_field};
use crate::datatables::{
    DataTablesColumn, DataTablesRequest, DataTablesResponse, fetch_limit,
    impl_datatables_request_methods,
};
use crate::error::Result;
use crate::models::{WatchListConfig, WatchListTicker};

/// Browser endpoint path for saving watchlist configurations.
pub(crate) const WATCH_LIST_CONFIG_PATH: &str = "/WatchListConfig";

/// Browser endpoint path for watchlist configuration DataTables rows.
pub(crate) const WATCH_LIST_CONFIGS_GET_WATCH_LISTS_PATH: &str = "/WatchListConfigs/GetWatchLists";

/// Browser endpoint path for watchlist ticker DataTables rows.
pub(crate) const WATCH_LISTS_GET_WATCH_LIST_TICKERS_PATH: &str = "/WatchLists/GetWatchListTickers";

/// Browser endpoint path for deleting watchlist configurations.
pub(crate) const WATCH_LIST_CONFIGS_DELETE_WATCH_LIST_PATH: &str =
    "/WatchListConfigs/DeleteWatchList";

/// Browser endpoint path for adding a ticker to a watchlist from the chart page.
pub(crate) const CHART0_UPDATE_WATCH_LIST_PATH: &str = "/Chart0/UpdateWatchList";

/// Redirect path VolumeLeaders uses after a successful watchlist save.
const WATCH_LIST_CONFIGS_SUCCESS_REDIRECT: &str = "/WatchListConfigs";

/// Request parameters for `/WatchListConfigs/GetWatchLists`.
#[derive(Clone, Debug)]
pub struct WatchListConfigsRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(WatchListConfigsRequest);

impl WatchListConfigsRequest {
    /// Create a watchlist configs request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: watchlist_configs_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        self.0.to_pairs()
    }
}

impl Default for WatchListConfigsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request parameters for `/WatchLists/GetWatchListTickers`.
#[derive(Clone, Debug)]
pub struct WatchListTickersRequest(pub(crate) DataTablesRequest);

impl_datatables_request_methods!(WatchListTickersRequest);

impl WatchListTickersRequest {
    /// Create a watchlist tickers request with default column definitions.
    #[must_use]
    pub fn new() -> Self {
        Self(DataTablesRequest {
            columns: watchlist_tickers_columns(),
            ..DataTablesRequest::default()
        })
    }

    /// Set the watchlist key filter.
    #[must_use]
    pub fn with_watch_list_key(mut self, watch_list_key: i64) -> Self {
        self.0 = self
            .0
            .with_extra_value("WatchListKey", watch_list_key.to_string());
        self
    }

    /// Return raw key-value pairs for form submission.
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        self.0.to_pairs()
    }
}

impl Default for WatchListTickersRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Multipart form payload for creating or editing a watchlist configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SaveWatchListConfigRequest {
    /// Raw browser field names and values accepted by VolumeLeaders.
    fields: Vec<(String, String)>,
}

/// Typed values for creating or editing a watchlist configuration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SaveWatchListConfigFields {
    pub search_template_key: i64,
    pub name: String,
    pub tickers: String,
    pub min_volume: i64,
    pub max_volume: i64,
    pub min_dollars: f64,
    pub max_dollars: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub min_vcd: f64,
    pub sector_industry: String,
    pub security_type_key: i64,
    pub min_relative_size_selected: i64,
    pub max_trade_rank_selected: i64,
    pub normal_prints_selected: bool,
    pub signature_prints_selected: bool,
    pub late_prints_selected: bool,
    pub timely_prints_selected: bool,
    pub dark_pools_selected: bool,
    pub lit_exchanges_selected: bool,
    pub sweeps_selected: bool,
    pub blocks_selected: bool,
    pub premarket_trades_selected: bool,
    pub rth_trades_selected: bool,
    pub ah_trades_selected: bool,
    pub opening_trades_selected: bool,
    pub closing_trades_selected: bool,
    pub phantom_trades_selected: bool,
    pub offsetting_trades_selected: bool,
    pub rsi_overbought_daily_selected: i64,
    pub rsi_overbought_hourly_selected: i64,
    pub rsi_oversold_daily_selected: i64,
    pub rsi_oversold_hourly_selected: i64,
}

impl SaveWatchListConfigRequest {
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

    /// Create a save request from typed watchlist configuration values.
    #[must_use]
    pub fn from_config(config: SaveWatchListConfigFields) -> Self {
        let mut fields = vec![
            (
                "SearchTemplateKey".into(),
                config.search_template_key.to_string(),
            ),
            ("Name".into(), config.name),
            ("Tickers".into(), config.tickers),
            ("MinVolume".into(), config.min_volume.to_string()),
            ("MaxVolume".into(), config.max_volume.to_string()),
            ("MinDollars".into(), config.min_dollars.to_string()),
            ("MaxDollars".into(), config.max_dollars.to_string()),
            ("MinPrice".into(), config.min_price.to_string()),
            ("MaxPrice".into(), config.max_price.to_string()),
            ("MinVCD".into(), config.min_vcd.to_string()),
            ("SectorIndustry".into(), config.sector_industry),
            (
                "SecurityTypeKey".into(),
                config.security_type_key.to_string(),
            ),
            (
                "MinRelativeSizeSelected".into(),
                config.min_relative_size_selected.to_string(),
            ),
            (
                "MaxTradeRankSelected".into(),
                config.max_trade_rank_selected.to_string(),
            ),
        ];

        push_bool_field(
            &mut fields,
            "NormalPrintsSelected",
            config.normal_prints_selected,
        );
        push_bool_field(
            &mut fields,
            "SignaturePrintsSelected",
            config.signature_prints_selected,
        );
        push_bool_field(
            &mut fields,
            "LatePrintsSelected",
            config.late_prints_selected,
        );
        push_bool_field(
            &mut fields,
            "TimelyPrintsSelected",
            config.timely_prints_selected,
        );
        push_bool_field(&mut fields, "DarkPoolsSelected", config.dark_pools_selected);
        push_bool_field(
            &mut fields,
            "LitExchangesSelected",
            config.lit_exchanges_selected,
        );
        push_bool_field(&mut fields, "SweepsSelected", config.sweeps_selected);
        push_bool_field(&mut fields, "BlocksSelected", config.blocks_selected);
        push_bool_field(
            &mut fields,
            "PremarketTradesSelected",
            config.premarket_trades_selected,
        );
        push_bool_field(&mut fields, "RTHTradesSelected", config.rth_trades_selected);
        push_bool_field(&mut fields, "AHTradesSelected", config.ah_trades_selected);
        push_bool_field(
            &mut fields,
            "OpeningTradesSelected",
            config.opening_trades_selected,
        );
        push_bool_field(
            &mut fields,
            "ClosingTradesSelected",
            config.closing_trades_selected,
        );
        push_bool_field(
            &mut fields,
            "PhantomTradesSelected",
            config.phantom_trades_selected,
        );
        push_bool_field(
            &mut fields,
            "OffsettingTradesSelected",
            config.offsetting_trades_selected,
        );

        fields.push((
            "RSIOverboughtDailySelected".into(),
            config.rsi_overbought_daily_selected.to_string(),
        ));
        fields.push((
            "RSIOverboughtHourlySelected".into(),
            config.rsi_overbought_hourly_selected.to_string(),
        ));
        fields.push((
            "RSIOversoldDailySelected".into(),
            config.rsi_oversold_daily_selected.to_string(),
        ));
        fields.push((
            "RSIOversoldHourlySelected".into(),
            config.rsi_oversold_hourly_selected.to_string(),
        ));

        Self { fields }
    }
}



/// Form payload for adding a ticker to an existing watchlist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddTickerToWatchListRequest {
    pub watch_list_key: i64,
    pub ticker: String,
}

/// JSON envelope returned by `/Chart0/UpdateWatchList`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTickerToWatchListResponse {
    pub success: bool,
    pub message: String,
}

/// JSON payload for deleting a watchlist.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteWatchListRequest {
    pub watch_list_key: i64,
}

/// Return the DataTables column definitions for watchlist configurations.
#[must_use]
pub fn watchlist_configs_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("Name", "", true, false),
        DataTablesColumn::new("Name", "Name", true, true),
        DataTablesColumn::new("Tickers", "Tickers", true, false),
        DataTablesColumn::new("Criteria", "Criteria", true, false),
    ]
}

/// Return the DataTables column definitions for watchlist tickers.
#[must_use]
pub fn watchlist_tickers_columns() -> Vec<DataTablesColumn> {
    vec![
        DataTablesColumn::new("Ticker", "Ticker", true, true),
        DataTablesColumn::new("Price", "Price", true, true),
        DataTablesColumn::new("NearestTop10TradeDate", "NearestTop10TradeDate", true, true),
        DataTablesColumn::new(
            "NearestTop10TradeClusterDate",
            "NearestTop10TradeClusterDate",
            true,
            true,
        ),
        DataTablesColumn::new(
            "NearestTop10TradeLevel",
            "NearestTop10TradeLevel",
            true,
            true,
        ),
        DataTablesColumn::new("Ticker", "Charts", true, false),
    ]
}

impl Client {
    /// Post a DataTables request to `/WatchListConfigs/GetWatchLists`.
    #[instrument(skip_all)]
    pub async fn get_watchlist_configs(
        &self,
        request: &WatchListConfigsRequest,
    ) -> Result<DataTablesResponse<WatchListConfig>> {
        let body = self
            .post_form(WATCH_LIST_CONFIGS_GET_WATCH_LISTS_PATH, request.to_pairs())
            .await?;
        Ok(serde_json::from_str(&body)?)
    }

    /// Fetch up to `limit` watchlist configurations by paginating the endpoint.
    #[instrument(skip_all)]
    pub async fn get_watchlist_configs_limit(
        &self,
        request: &WatchListConfigsRequest,
        limit: usize,
    ) -> Result<Vec<WatchListConfig>> {
        fetch_limit(
            self,
            WATCH_LIST_CONFIGS_GET_WATCH_LISTS_PATH,
            request.0.clone(),
            limit,
        )
        .await
    }

    /// Post a DataTables request to `/WatchLists/GetWatchListTickers`.
    #[instrument(skip_all)]
    pub async fn get_watchlist_tickers(
        &self,
        request: &WatchListTickersRequest,
    ) -> Result<DataTablesResponse<WatchListTicker>> {
        let body = self
            .post_form(WATCH_LISTS_GET_WATCH_LIST_TICKERS_PATH, request.to_pairs())
            .await?;
        Ok(serde_json::from_str(&body)?)
    }

    /// Fetch up to `limit` watchlist tickers by paginating the endpoint.
    #[instrument(skip_all)]
    pub async fn get_watchlist_tickers_limit(
        &self,
        request: &WatchListTickersRequest,
        limit: usize,
    ) -> Result<Vec<WatchListTicker>> {
        fetch_limit(
            self,
            WATCH_LISTS_GET_WATCH_LIST_TICKERS_PATH,
            request.0.clone(),
            limit,
        )
        .await
    }

    /// Post a multipart create or edit request to `/WatchListConfig`.
    #[instrument(skip_all)]
    pub async fn save_watchlist_config(&self, request: SaveWatchListConfigRequest) -> Result<()> {
        self.post_multipart_form(
            WATCH_LIST_CONFIG_PATH,
            multipart_form_from_fields(request.fields()),
            WATCH_LIST_CONFIGS_SUCCESS_REDIRECT,
        )
        .await
    }

    /// Post a chart-page form request that adds a ticker to a watchlist.
    #[instrument(skip_all)]
    pub async fn add_ticker_to_watchlist(
        &self,
        request: &AddTickerToWatchListRequest,
    ) -> Result<AddTickerToWatchListResponse> {
        let body = self
            .post_form(
                CHART0_UPDATE_WATCH_LIST_PATH,
                vec![
                    (
                        "WatchListKey".to_string(),
                        request.watch_list_key.to_string(),
                    ),
                    ("Ticker".to_string(), request.ticker.clone()),
                ],
            )
            .await?;
        Ok(serde_json::from_str(&body)?)
    }

    /// Post a JSON delete request to `/WatchListConfigs/DeleteWatchList`.
    #[instrument(skip_all)]
    pub async fn delete_watchlist(&self, request: &DeleteWatchListRequest) -> Result<()> {
        self.post_json(WATCH_LIST_CONFIGS_DELETE_WATCH_LIST_PATH, request)
            .await
            .map(|_| ())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{datatables_body, test_client};

    #[test]
    fn watchlist_configs_columns_match_go_source() {
        let columns = watchlist_configs_columns();

        assert_eq!(columns.len(), 4);
        assert_eq!(columns[0], DataTablesColumn::new("Name", "", true, false));
        assert_eq!(
            columns[1],
            DataTablesColumn::new("Name", "Name", true, true)
        );
        assert_eq!(
            columns[2],
            DataTablesColumn::new("Tickers", "Tickers", true, false)
        );
        assert_eq!(
            columns[3],
            DataTablesColumn::new("Criteria", "Criteria", true, false)
        );
    }

    #[test]
    fn watchlist_tickers_columns_match_go_source() {
        let columns = watchlist_tickers_columns();

        assert_eq!(columns.len(), 6);
        assert_eq!(
            columns[0],
            DataTablesColumn::new("Ticker", "Ticker", true, true)
        );
        assert_eq!(
            columns[4],
            DataTablesColumn::new(
                "NearestTop10TradeLevel",
                "NearestTop10TradeLevel",
                true,
                true
            )
        );
        assert_eq!(
            columns[5],
            DataTablesColumn::new("Ticker", "Charts", true, false)
        );
    }

    #[tokio::test]
    async fn get_watchlist_configs_posts_datatables_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", WATCH_LIST_CONFIGS_GET_WATCH_LISTS_PATH)
            .match_header(
                "content-type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .match_body(mockito::Matcher::Regex(
                r"(?:^|&)columns\[0\]\[data\]=Name(?:&|.*&)columns\[2\]\[data\]=Tickers(?:&|.*&)columns\[3\]\[data\]=Criteria"
                    .to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"draw":1,"recordsTotal":1,"recordsFiltered":1,"data":[{"SearchTemplateKey":6307,"Name":"Testing 3","Tickers":"SPY,AAPL","RSIOverboughtHourly":70,"RSIOverboughtDaily":null,"RSIOverboughtHourlySelected":true}]}"#)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .get_watchlist_configs(&WatchListConfigsRequest::new())
            .await
            .unwrap();

        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].search_template_key, Some(6307));
        assert_eq!(response.data[0].rsi_overbought_hourly, Some(70));
        assert_eq!(response.data[0].rsi_overbought_daily, None);
        assert_eq!(response.data[0].rsi_overbought_hourly_selected, Some(true));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn get_watchlist_tickers_posts_datatables_request() {
        let mut server = mockito::Server::new_async().await;
        let mut request = WatchListTickersRequest::new();
        request
            .0
            .extra_values
            .push(("WatchListKey".to_string(), "6260".to_string()));
        let mock = server
            .mock("POST", WATCH_LISTS_GET_WATCH_LIST_TICKERS_PATH)
            .match_body(mockito::Matcher::Regex(
                r"(?:^|&)columns\[0\]\[data\]=Ticker(?:&|.*&)columns\[4\]\[data\]=NearestTop10TradeLevel(?:&|.*&)WatchListKey=6260(?:&|$)"
                    .to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"draw":1,"recordsTotal":1,"recordsFiltered":1,"data":[{"WatchListKey":0,"SecurityKey":63,"Ticker":"AAPL","Sector":"Technology","NearestTop10TradeDate":"/Date(1766102400000)/","NearestTop10TradeLevel":273.7,"NearestTop10TradeLevelRank":3}]}"#)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client.get_watchlist_tickers(&request).await.unwrap();

        assert_eq!(response.data[0].security_key, Some(63));
        assert_eq!(response.data[0].ticker.as_deref(), Some("AAPL"));
        assert_eq!(response.data[0].nearest_top10_trade_level, Some(273.7));
        assert_eq!(response.data[0].nearest_top10_trade_level_rank, Some(3));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn watchlist_limit_methods_page_through_results() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", WATCH_LIST_CONFIGS_GET_WATCH_LISTS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(datatables_body(vec![
                serde_json::json!({"SearchTemplateKey": 6307}),
            ]))
            .create_async()
            .await;
        server
            .mock("POST", WATCH_LISTS_GET_WATCH_LIST_TICKERS_PATH)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(datatables_body(vec![serde_json::json!({"Ticker": "AAPL"})]))
            .create_async()
            .await;
        let client = test_client(&server);

        let configs = client
            .get_watchlist_configs_limit(&WatchListConfigsRequest::new(), 1)
            .await
            .unwrap();
        let tickers = client
            .get_watchlist_tickers_limit(&WatchListTickersRequest::new(), 1)
            .await
            .unwrap();

        assert_eq!(configs[0].search_template_key, Some(6307));
        assert_eq!(tickers[0].ticker.as_deref(), Some("AAPL"));
    }

    #[tokio::test]
    async fn save_watchlist_config_posts_multipart_form_and_accepts_redirect() {
        let mut server = mockito::Server::new_async().await;
        let save = server
            .mock("POST", WATCH_LIST_CONFIG_PATH)
            .match_header("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .match_body(mockito::Matcher::Regex(
                r#"(?s)name="SearchTemplateKey"\r\n\r\n6307.*name="NormalPrintsSelected"\r\n\r\ntrue.*name="NormalPrintsSelected"\r\n\r\nfalse"#
                    .to_string(),
            ))
            .with_status(302)
            .with_header("location", "/WatchListConfigs?ViewMode=Desktop")
            .create_async()
            .await;
        let follow = server
            .mock("GET", "/WatchListConfigs?ViewMode=Desktop")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("saved")
            .create_async()
            .await;
        let client = test_client(&server);

        client
            .save_watchlist_config(SaveWatchListConfigRequest::new(vec![
                ("SearchTemplateKey".to_string(), "6307".to_string()),
                ("Name".to_string(), "Testing 3".to_string()),
                ("Tickers".to_string(), "SPY,AAPL".to_string()),
                ("NormalPrintsSelected".to_string(), "true".to_string()),
                ("NormalPrintsSelected".to_string(), "false".to_string()),
            ]))
            .await
            .unwrap();

        save.assert_async().await;
        follow.assert_async().await;
    }

    #[tokio::test]
    async fn add_ticker_to_watchlist_posts_captured_form() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", CHART0_UPDATE_WATCH_LIST_PATH)
            .match_header(
                "content-type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .match_body("WatchListKey=6276&Ticker=AMD")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"success":true,"message":"Watch List updated!"}"#)
            .create_async()
            .await;
        let client = test_client(&server);

        let response = client
            .add_ticker_to_watchlist(&AddTickerToWatchListRequest {
                watch_list_key: 6276,
                ticker: "AMD".to_string(),
            })
            .await
            .unwrap();

        assert!(response.success);
        assert_eq!(response.message, "Watch List updated!");
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn delete_watchlist_posts_json_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", WATCH_LIST_CONFIGS_DELETE_WATCH_LIST_PATH)
            .match_header("content-type", "application/json; charset=UTF-8")
            .match_header("x-requested-with", "XMLHttpRequest")
            .match_body(r#"{"WatchListKey":6282}"#)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("null")
            .create_async()
            .await;
        let client = test_client(&server);

        client
            .delete_watchlist(&DeleteWatchListRequest {
                watch_list_key: 6282,
            })
            .await
            .unwrap();

        mock.assert_async().await;
    }
}
