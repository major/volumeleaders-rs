//! VolumeLeaders API client library and CLI.
//!
//! This project is unofficial and is not affiliated with, endorsed by, or
//! sponsored by [volumeleaders.com](https://www.volumeleaders.com).

#![deny(missing_docs)]

/// Alert configuration and alert DataTables endpoints.
pub mod alerts;
/// Login-page detection and XSRF token extraction.
pub mod auth;
/// Session cookie cache persistence.
pub mod cache;
/// Command-line interface for VolumeLeaders data.
#[cfg(feature = "cli")]
pub mod cli;
/// HTTP client with cookie and XSRF header management.
pub mod client;
/// Trade cluster and cluster bomb DataTables endpoints.
pub mod clusters;
/// Credential configuration loading.
pub mod config;
/// ASP.NET DataTables wire format encoding and pagination.
pub mod datatables;
/// Earnings calendar DataTables endpoint.
pub mod earnings;
/// Error types and result alias.
pub mod error;
/// Executive summary endpoints (exhaustion scores, welcome trades/clusters).
pub mod executive_summary;
/// Trade level and level-touch DataTables endpoints.
pub mod levels;
/// Username/password login for VolumeLeaders.
pub mod login;
/// API response models for trade data.
pub mod models;
/// Browser session material (cookies and XSRF token).
pub mod session;
/// Institutional trade DataTables endpoint.
pub mod trades;
/// Volume DataTables endpoint.
pub mod volume;
/// Watchlist configuration and ticker DataTables endpoints.
pub mod watchlists;

/// Test utilities and fixture helpers (test and `test-support` feature only).
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

pub use alerts::{
    AlertConfigsRequest, DeleteAlertConfigRequest, SaveAlertConfigFields, SaveAlertConfigRequest,
    TradeAlertsRequest, TradeClusterAlertsRequest,
};
pub use auth::{extract_xsrf_token, is_login_page};
pub use cache::{clear_cache, load_cached_session, save_session};
#[cfg(feature = "cli")]
pub use cli::{Cli, run};
pub use client::{Client, ClientConfig};
pub use clusters::{TradeClusterBombsRequest, TradeClustersRequest};
pub use config::{CredentialSource, Credentials, ResolvedCredentials, resolve_credentials};
pub use datatables::{DataTablesColumn, DataTablesResponse};
pub use earnings::EarningsRequest;
pub use error::{ClientError, Result};
pub use executive_summary::{
    ExhaustionScoresRequest, WelcomeTradeClustersRequest, WelcomeTradesRequest,
};
pub use levels::{TradeLevelTouchesRequest, TradeLevelsRequest};
pub use login::login;
pub use models::{
    AlertConfig, AspNetDate, Earning, ExhaustionScore, FlexBool, Trade, TradeAlert, TradeCluster,
    TradeClusterAlert, TradeClusterBomb, TradeLevel, WatchListConfig, WatchListTicker,
};
pub use session::Session;
pub use trades::TradesRequest;
pub use volume::VolumeRequest;
pub use watchlists::{
    AddTickerToWatchListRequest, AddTickerToWatchListResponse, DeleteWatchListRequest,
    SaveWatchListConfigFields, SaveWatchListConfigRequest, WatchListConfigsRequest,
    WatchListTickersRequest,
};
