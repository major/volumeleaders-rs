//! DataTables request and response types.
//!
//! VolumeLeaders endpoints use ASP.NET MVC's bracketed form field names for
//! server-side DataTables requests. The encoder below keeps those keys literal
//! instead of delegating to a generic form serializer, because generic URL form
//! encoders percent-encode the brackets and produce a body the server rejects.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::client::Client;
use crate::error::Result;

const DEFAULT_DATATABLES_LENGTH: i32 = 25;
const DEFAULT_ORDER_COLUMN: i32 = 1;
const DEFAULT_ORDER_DIR: &str = "desc";

/// Page size used by the DataTables paginator.
pub const PAGE_SIZE: usize = 100;

/// One DataTables form column definition.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DataTablesColumn {
    /// Data field name sent in `columns[N][data]`.
    pub data: String,
    /// Server-side display/sort name sent in `columns[N][name]`.
    pub name: String,
    /// Whether the column participates in search.
    pub searchable: bool,
    /// Whether the column can be ordered.
    pub orderable: bool,
}

impl DataTablesColumn {
    /// Build a DataTables column definition.
    #[must_use]
    pub fn new(
        data: impl Into<String>,
        name: impl Into<String>,
        searchable: bool,
        orderable: bool,
    ) -> Self {
        Self {
            data: data.into(),
            name: name.into(),
            searchable,
            orderable,
        }
    }
}

/// One DataTables sort instruction.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DataTablesOrder {
    /// Zero-based column index to sort on.
    pub column: i32,
    /// Sort direction, usually `asc` or `desc`.
    pub dir: String,
    /// Optional named sort key sent as `order[N][name]` when non-empty.
    pub name: String,
}

impl DataTablesOrder {
    /// Build a DataTables order definition.
    #[must_use]
    pub fn new(column: i32, dir: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            column,
            dir: dir.into(),
            name: name.into(),
        }
    }
}

/// Server-side DataTables form request.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DataTablesRequest {
    /// Request draw counter. Defaults to `1` when zero.
    pub(crate) draw: i32,
    /// Zero-based starting row.
    pub(crate) start: i32,
    /// Requested page length. Defaults to `25` when zero.
    pub(crate) length: i32,
    /// DataTables column definitions.
    pub(crate) columns: Vec<DataTablesColumn>,
    /// DataTables sort definitions. Defaults to column `1`, direction `desc`.
    pub(crate) order: Vec<DataTablesOrder>,
    /// Global search value included only when `include_search` is true.
    pub(crate) search_value: String,
    /// Global search regex flag included only when `include_search` is true.
    pub(crate) search_regex: bool,
    /// Whether to include global `search[...]` form fields.
    pub(crate) include_search: bool,
    /// Extra endpoint-specific form values appended after DataTables fields.
    pub(crate) extra_values: Vec<(String, String)>,
}

impl DataTablesRequest {
    /// Replace the column definitions.
    #[must_use]
    pub fn with_columns(mut self, columns: Vec<DataTablesColumn>) -> Self {
        self.columns = columns;
        self
    }

    /// Set the zero-based starting row.
    #[must_use]
    pub fn with_start(mut self, start: i32) -> Self {
        self.start = start;
        self
    }

    /// Set the requested page length.
    #[must_use]
    pub fn with_length(mut self, length: i32) -> Self {
        self.length = length;
        self
    }

    /// Replace the sort list with a single order definition.
    #[must_use]
    pub fn with_order(self, column: i32, dir: impl Into<String>, name: impl Into<String>) -> Self {
        self.with_orders(vec![DataTablesOrder::new(column, dir, name)])
    }

    /// Replace the sort list.
    #[must_use]
    pub fn with_orders(mut self, order: Vec<DataTablesOrder>) -> Self {
        self.order = order;
        self
    }

    /// Enable global search fields.
    #[must_use]
    pub fn with_search(mut self, value: impl Into<String>, regex: bool) -> Self {
        self.include_search = true;
        self.search_value = value.into();
        self.search_regex = regex;
        self
    }

    /// Add one endpoint-specific form value.
    #[must_use]
    pub(crate) fn with_extra_value(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.extra_values.push((key.into(), value.into()));
        self
    }

    /// Replace endpoint-specific form values.
    #[must_use]
    pub(crate) fn with_extra_values(mut self, values: Vec<(String, String)>) -> Self {
        self.extra_values = values;
        self
    }

    /// Return endpoint-specific form values for assertions and diagnostics.
    #[must_use]
    pub fn extra_values(&self) -> &[(String, String)] {
        &self.extra_values
    }

    /// Encode this request as an ASP.NET-compatible form body.
    #[must_use]
    pub fn encode(&self) -> String {
        raw_pairs(self)
            .into_iter()
            .map(|(k, v)| format!("{k}={}", encode_value(&v)))
            .collect::<Vec<_>>()
            .join("&")
    }

    /// Return raw key-value pairs for use with [`Client::post_form`].
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        raw_pairs(self)
    }
}

macro_rules! impl_datatables_request_methods {
    ($request_type:ty) => {
        impl $request_type {
            /// Replace the DataTables column definitions.
            #[must_use]
            pub fn with_columns(
                mut self,
                columns: Vec<$crate::datatables::DataTablesColumn>,
            ) -> Self {
                self.0 = self.0.with_columns(columns);
                self
            }

            /// Set the zero-based starting row.
            #[must_use]
            pub fn with_start(mut self, start: i32) -> Self {
                self.0 = self.0.with_start(start);
                self
            }

            /// Set the requested page length.
            #[must_use]
            pub fn with_length(mut self, length: i32) -> Self {
                self.0 = self.0.with_length(length);
                self
            }

            /// Replace the sort list with a single order definition.
            #[must_use]
            pub fn with_order(
                mut self,
                column: i32,
                dir: impl Into<String>,
                name: impl Into<String>,
            ) -> Self {
                self.0 = self.0.with_order(column, dir, name);
                self
            }

            /// Enable global search fields.
            #[must_use]
            pub fn with_search(mut self, value: impl Into<String>, regex: bool) -> Self {
                self.0 = self.0.with_search(value, regex);
                self
            }

            /// Return endpoint-specific form values for assertions and diagnostics.
            #[must_use]
            pub fn extra_values(&self) -> &[(String, String)] {
                self.0.extra_values()
            }

            /// Encode this request as an ASP.NET-compatible form body.
            #[must_use]
            pub fn encode(&self) -> String {
                self.0.encode()
            }
        }
    };
}

pub(crate) use impl_datatables_request_methods;

/// Typed server-side DataTables JSON envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct DataTablesResponse<T> {
    /// Request draw counter echoed by the server.
    #[serde(default = "default_i32")]
    pub draw: i32,
    /// Total rows before filtering.
    #[serde(default = "default_i32")]
    pub records_total: i32,
    /// Total rows after filtering.
    #[serde(default = "default_i32")]
    pub records_filtered: i32,
    /// Page of row data.
    #[serde(default = "Vec::new", deserialize_with = "deserialize_rows")]
    pub data: Vec<T>,
    /// Optional DataTables error message.
    #[serde(default)]
    pub error: Option<String>,
}

fn raw_pairs(req: &DataTablesRequest) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let draw = if req.draw == 0 { 1 } else { req.draw };
    let length = if req.length == 0 {
        DEFAULT_DATATABLES_LENGTH
    } else {
        req.length
    };

    pairs.push(("draw".into(), draw.to_string()));
    pairs.push(("start".into(), req.start.to_string()));
    pairs.push(("length".into(), length.to_string()));

    for (index, column) in req.columns.iter().enumerate() {
        let prefix = format!("columns[{index}]");
        pairs.push((format!("{prefix}[data]"), column.data.clone()));
        pairs.push((format!("{prefix}[name]"), column.name.clone()));
        pairs.push((
            format!("{prefix}[searchable]"),
            bool_value(column.searchable).to_string(),
        ));
        pairs.push((
            format!("{prefix}[orderable]"),
            bool_value(column.orderable).to_string(),
        ));
        pairs.push((format!("{prefix}[search][value]"), String::new()));
        pairs.push((format!("{prefix}[search][regex]"), "false".to_string()));
    }

    if req.order.is_empty() {
        push_order_raw(&mut pairs, 0, DEFAULT_ORDER_COLUMN, DEFAULT_ORDER_DIR, "");
    } else {
        for (index, order) in req.order.iter().enumerate() {
            let dir = if order.dir.is_empty() {
                DEFAULT_ORDER_DIR
            } else {
                order.dir.as_str()
            };
            push_order_raw(&mut pairs, index, order.column, dir, &order.name);
        }
    }

    if req.include_search {
        pairs.push(("search[value]".into(), req.search_value.clone()));
        pairs.push((
            "search[regex]".into(),
            bool_value(req.search_regex).to_string(),
        ));
    }

    for (key, value) in &req.extra_values {
        pairs.push((key.clone(), value.clone()));
    }

    pairs
}

fn default_i32() -> i32 {
    0
}

fn deserialize_rows<'de, D, T>(deserializer: D) -> std::result::Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

fn push_order_raw(
    pairs: &mut Vec<(String, String)>,
    index: usize,
    column: i32,
    dir: &str,
    name: &str,
) {
    let prefix = format!("order[{index}]");
    pairs.push((format!("{prefix}[column]"), column.to_string()));
    pairs.push((format!("{prefix}[dir]"), dir.to_string()));
    if !name.is_empty() {
        pairs.push((format!("{prefix}[name]"), name.to_string()));
    }
}

fn bool_value(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn encode_value(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(char::from(byte));
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push('%');
                encoded.push(hex_digit(byte >> 4));
                encoded.push(hex_digit(byte & 0x0f));
            }
        }
    }
    encoded
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'A' + value - 10),
        _ => unreachable!("nibble values are always in 0..=15"),
    }
}

/// Paginate a DataTables endpoint, collecting up to `limit` records.
///
/// Each page posts `request` with an updated `start` offset and deserializes
/// the response as [`DataTablesResponse<T>`]. Pagination stops when any of:
/// - `records_total` is zero (no data available)
/// - The page returned fewer than [`PAGE_SIZE`] items (last page)
/// - Total collected items reaches `limit`
pub async fn fetch_limit<T>(
    client: &Client,
    path: &str,
    mut request: DataTablesRequest,
    limit: usize,
) -> Result<Vec<T>>
where
    T: DeserializeOwned + Send,
{
    let mut results = Vec::new();
    request.length = PAGE_SIZE as i32;

    loop {
        request.start = results.len() as i32;
        let body = client.post_form(path, request.to_pairs()).await?;
        let response: DataTablesResponse<T> = serde_json::from_str(&body)?;
        let page_len = response.data.len();
        results.extend(response.data);

        if response.records_total == 0 || page_len < PAGE_SIZE || results.len() >= limit {
            break;
        }
    }

    results.truncate(limit);
    Ok(results)
}

/// Paginate a DataTables endpoint, collecting all available records.
pub async fn fetch_all<T>(client: &Client, path: &str, request: DataTablesRequest) -> Result<Vec<T>>
where
    T: DeserializeOwned + Send,
{
    fetch_limit(client, path, request, usize::MAX).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datatables_encode_defaults_match_go_encoder() {
        let encoded = DataTablesRequest::default().encode();

        assert_eq!(
            encoded,
            "draw=1&start=0&length=25&order[0][column]=1&order[0][dir]=desc"
        );
    }

    #[test]
    fn datatables_encode_columns_search_order_and_extra_values_match_go_fields() {
        let req = DataTablesRequest {
            draw: 3,
            start: 25,
            length: 50,
            columns: vec![
                DataTablesColumn::new("Ticker", "Ticker", true, true),
                DataTablesColumn::new("Volume", "Sh", false, true),
            ],
            order: vec![
                DataTablesOrder::new(1, "asc", "Sh"),
                DataTablesOrder::new(0, "", ""),
            ],
            include_search: true,
            search_value: "AXP".to_string(),
            search_regex: true,
            extra_values: vec![
                ("Tickers".to_string(), "AXP".to_string()),
                ("Tickers".to_string(), "MSFT".to_string()),
                ("MinSize".to_string(), "1000".to_string()),
            ],
        };

        assert_eq!(
            req.encode(),
            "draw=3&start=25&length=50&columns[0][data]=Ticker&columns[0][name]=Ticker&columns[0][searchable]=true&columns[0][orderable]=true&columns[0][search][value]=&columns[0][search][regex]=false&columns[1][data]=Volume&columns[1][name]=Sh&columns[1][searchable]=false&columns[1][orderable]=true&columns[1][search][value]=&columns[1][search][regex]=false&order[0][column]=1&order[0][dir]=asc&order[0][name]=Sh&order[1][column]=0&order[1][dir]=desc&search[value]=AXP&search[regex]=true&Tickers=AXP&Tickers=MSFT&MinSize=1000"
        );
    }

    #[test]
    fn datatables_encode_three_column_golden_matches_go_encoder_order() {
        let req = DataTablesRequest {
            draw: 1,
            start: 0,
            length: 100,
            columns: vec![
                DataTablesColumn::new("Symbol", "", true, true),
                DataTablesColumn::new("Last", "Last Price", false, true),
                DataTablesColumn::new("Change", "", true, false),
            ],
            order: vec![DataTablesOrder::new(0, "asc", "")],
            include_search: true,
            extra_values: vec![("Sector".to_string(), "Financial Services".to_string())],
            ..DataTablesRequest::default()
        };

        assert_eq!(
            req.encode(),
            "draw=1&start=0&length=100&columns[0][data]=Symbol&columns[0][name]=&columns[0][searchable]=true&columns[0][orderable]=true&columns[0][search][value]=&columns[0][search][regex]=false&columns[1][data]=Last&columns[1][name]=Last+Price&columns[1][searchable]=false&columns[1][orderable]=true&columns[1][search][value]=&columns[1][search][regex]=false&columns[2][data]=Change&columns[2][name]=&columns[2][searchable]=true&columns[2][orderable]=false&columns[2][search][value]=&columns[2][search][regex]=false&order[0][column]=0&order[0][dir]=asc&search[value]=&search[regex]=false&Sector=Financial+Services"
        );
    }

    #[test]
    fn datatables_encode_uses_form_encoding_for_values_only() {
        let req = DataTablesRequest {
            include_search: true,
            search_value: "A&B value".to_string(),
            extra_values: vec![("columns[extra]".to_string(), "x/y".to_string())],
            ..DataTablesRequest::default()
        };

        assert_eq!(
            req.encode(),
            "draw=1&start=0&length=25&order[0][column]=1&order[0][dir]=desc&search[value]=A%26B+value&search[regex]=false&columns[extra]=x%2Fy"
        );
    }

    #[test]
    fn datatables_response_deserializes_fixture_json() {
        let fixture = crate::test_support::read_fixture("trades_get_trades_response.json");
        let response: DataTablesResponse<serde_json::Value> =
            serde_json::from_str(&fixture).unwrap();

        assert_eq!(response.draw, 1);
        assert_eq!(response.records_total, 465);
        assert_eq!(response.records_filtered, 465);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.error, None);
    }

    #[test]
    fn datatables_response_deserializes_data_only_json() {
        let json = r#"{"data":[{"id":1},{"id":2}]}"#;
        let response: DataTablesResponse<TestRow> = serde_json::from_str(json).unwrap();

        assert_eq!(response.draw, 0);
        assert_eq!(response.records_total, 0);
        assert_eq!(response.records_filtered, 0);
        assert_eq!(response.data, vec![TestRow { id: 1 }, TestRow { id: 2 }]);
        assert_eq!(response.error, None);
    }

    #[test]
    fn datatables_response_deserializes_null_data_as_empty_json() {
        let json = r#"{"data":null}"#;
        let response: DataTablesResponse<TestRow> = serde_json::from_str(json).unwrap();

        assert_eq!(response.draw, 0);
        assert_eq!(response.records_total, 0);
        assert_eq!(response.records_filtered, 0);
        assert!(response.data.is_empty());
        assert_eq!(response.error, None);
    }

    // --- paginator tests ---

    use crate::client::ClientConfig;
    use crate::session::{
        COOKIE_DOMAIN, Cookie, FORMS_AUTH_COOKIE_NAME, SESSION_COOKIE_NAME, Session,
    };

    #[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
    struct TestRow {
        id: i32,
    }

    fn test_session() -> Session {
        Session::new(
            vec![
                Cookie::new(SESSION_COOKIE_NAME, "session-123", COOKIE_DOMAIN),
                Cookie::new(FORMS_AUTH_COOKIE_NAME, "auth-456", COOKIE_DOMAIN),
            ],
            "xsrf-789",
        )
    }

    fn test_client(server: &mockito::Server) -> Client {
        Client::with_config(
            test_session(),
            ClientConfig {
                base_url: server.url(),
                ..ClientConfig::default()
            },
        )
        .unwrap()
    }

    fn make_response(data: Vec<TestRow>, records_total: i32) -> String {
        serde_json::to_string(&DataTablesResponse {
            draw: 1,
            records_total,
            records_filtered: records_total,
            data,
            error: None,
        })
        .unwrap()
    }

    #[tokio::test]
    async fn fetch_limit_single_page_returns_all_items() {
        let mut server = mockito::Server::new_async().await;
        let rows: Vec<TestRow> = (1..=3).map(|i| TestRow { id: i }).collect();
        let mock = server
            .mock("POST", "/data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(make_response(rows.clone(), 3))
            .create_async()
            .await;
        let client = test_client(&server);

        let result: Vec<TestRow> = fetch_all(&client, "/data", DataTablesRequest::default())
            .await
            .unwrap();

        assert_eq!(result, rows);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn fetch_limit_multi_page_collects_all_pages() {
        let mut server = mockito::Server::new_async().await;
        let page1: Vec<TestRow> = (1..=100).map(|i| TestRow { id: i }).collect();
        let page2: Vec<TestRow> = (101..=150).map(|i| TestRow { id: i }).collect();

        let mock1 = server
            .mock("POST", "/data")
            .match_body(mockito::Matcher::Regex(r"(?:^|&)start=0(&|$)".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(make_response(page1, 150))
            .create_async()
            .await;
        let mock2 = server
            .mock("POST", "/data")
            .match_body(mockito::Matcher::Regex(
                r"(?:^|&)start=100(&|$)".to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(make_response(page2, 150))
            .create_async()
            .await;
        let client = test_client(&server);

        let result: Vec<TestRow> = fetch_all(&client, "/data", DataTablesRequest::default())
            .await
            .unwrap();

        assert_eq!(result.len(), 150);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[99].id, 100);
        assert_eq!(result[100].id, 101);
        assert_eq!(result[149].id, 150);
        mock1.assert_async().await;
        mock2.assert_async().await;
    }

    #[tokio::test]
    async fn fetch_limit_respects_limit() {
        let mut server = mockito::Server::new_async().await;
        let page: Vec<TestRow> = (1..=100).map(|i| TestRow { id: i }).collect();
        server
            .mock("POST", "/data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(make_response(page, 1000))
            .create_async()
            .await;
        let client = test_client(&server);

        let result: Vec<TestRow> = fetch_limit(&client, "/data", DataTablesRequest::default(), 5)
            .await
            .unwrap();

        assert_eq!(result.len(), 5);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[4].id, 5);
    }

    #[tokio::test]
    async fn fetch_limit_empty_result_returns_empty_vec() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("POST", "/data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(make_response(vec![], 0))
            .create_async()
            .await;
        let client = test_client(&server);

        let result: Vec<TestRow> = fetch_all(&client, "/data", DataTablesRequest::default())
            .await
            .unwrap();

        assert!(result.is_empty());
    }
}
