//! VolumeLeaders API client library.
//!
//! This project is unofficial and is not affiliated with, endorsed by, or
//! sponsored by [volumeleaders.com](https://www.volumeleaders.com).

pub mod alerts;
pub mod auth;
pub mod browser_auth;
pub mod client;
pub mod clusters;
pub mod datatables;
pub mod earnings;
pub mod error;
pub mod executive_summary;
pub mod levels;
pub mod models;
pub mod session;
pub mod trades;
pub mod volume;
pub mod watchlists;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

pub use alerts::{
    AlertConfigsRequest, DeleteAlertConfigRequest, SaveAlertConfigFields, SaveAlertConfigRequest,
    TradeAlertsRequest, TradeClusterAlertsRequest,
};
pub use auth::{extract_xsrf_token, is_login_page};
pub use browser_auth::{extract_browser_cookies, session_from_browser};
pub use client::{Client, ClientConfig};
pub use clusters::{TradeClusterBombsRequest, TradeClustersRequest};
pub use datatables::{DataTablesColumn, DataTablesResponse};
pub use earnings::EarningsRequest;
pub use error::{ClientError, Result};
pub use executive_summary::{
    ExhaustionScoresRequest, WelcomeTradeClustersRequest, WelcomeTradesRequest,
};
pub use levels::{TradeLevelTouchesRequest, TradeLevelsRequest};
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
