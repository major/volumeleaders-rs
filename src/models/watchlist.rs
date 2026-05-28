//! VolumeLeaders watchlist configuration and ticker row models.

use serde::{Deserialize, Serialize};

use super::types::AspNetDate;

/// A saved VolumeLeaders watchlist configuration row.
///
/// All fields are optional because the DataTables payload can omit disabled
/// criteria and returns `null` for unset filters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WatchListConfig {
    pub search_template_key: Option<i64>,
    pub user_key: Option<i64>,
    pub search_template_type_key: Option<i64>,
    pub name: Option<String>,
    pub tickers: Option<String>,
    pub criteria: Option<String>,
    pub sort_order: Option<i64>,
    pub min_volume: Option<i64>,
    pub max_volume: Option<i64>,
    pub min_dollars: Option<f64>,
    pub max_dollars: Option<f64>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    #[serde(rename = "RSIOverboughtHourly")]
    pub rsi_overbought_hourly: Option<i64>,
    #[serde(rename = "RSIOverboughtDaily")]
    pub rsi_overbought_daily: Option<i64>,
    #[serde(rename = "RSIOversoldHourly")]
    pub rsi_oversold_hourly: Option<i64>,
    #[serde(rename = "RSIOversoldDaily")]
    pub rsi_oversold_daily: Option<i64>,
    #[serde(rename = "RSIOverboughtHourlySelected")]
    pub rsi_overbought_hourly_selected: Option<bool>,
    #[serde(rename = "RSIOverboughtDailySelected")]
    pub rsi_overbought_daily_selected: Option<bool>,
    #[serde(rename = "RSIOversoldHourlySelected")]
    pub rsi_oversold_hourly_selected: Option<bool>,
    #[serde(rename = "RSIOversoldDailySelected")]
    pub rsi_oversold_daily_selected: Option<bool>,
    pub conditions: Option<String>,
    pub min_relative_size: Option<i64>,
    pub min_relative_size_selected: Option<bool>,
    pub max_trade_rank: Option<i64>,
    pub security_type_key: Option<i64>,
    pub security_type: Option<String>,
    pub max_trade_rank_selected: Option<bool>,
    #[serde(rename = "MinVCD")]
    pub min_vcd: Option<f64>,
    pub normal_prints: Option<bool>,
    pub normal_prints_selected: Option<bool>,
    pub signature_prints: Option<bool>,
    pub signature_prints_selected: Option<bool>,
    pub late_prints: Option<bool>,
    pub late_prints_selected: Option<bool>,
    pub timely_prints: Option<bool>,
    pub timely_prints_selected: Option<bool>,
    pub dark_pools: Option<bool>,
    pub dark_pools_selected: Option<bool>,
    pub lit_exchanges: Option<bool>,
    pub lit_exchanges_selected: Option<bool>,
    pub sweeps: Option<bool>,
    pub sweeps_selected: Option<bool>,
    pub blocks: Option<bool>,
    pub blocks_selected: Option<bool>,
    pub premarket_trades: Option<bool>,
    pub premarket_trades_selected: Option<bool>,
    #[serde(rename = "RTHTrades")]
    pub rth_trades: Option<bool>,
    #[serde(rename = "RTHTradesSelected")]
    pub rth_trades_selected: Option<bool>,
    #[serde(rename = "AHTrades")]
    pub ah_trades: Option<bool>,
    #[serde(rename = "AHTradesSelected")]
    pub ah_trades_selected: Option<bool>,
    pub opening_trades: Option<bool>,
    pub opening_trades_selected: Option<bool>,
    pub closing_trades: Option<bool>,
    pub closing_trades_selected: Option<bool>,
    pub phantom_trades: Option<bool>,
    pub phantom_trades_selected: Option<bool>,
    pub offsetting_trades: Option<bool>,
    pub offsetting_trades_selected: Option<bool>,
    pub sector_industry: Option<String>,
    #[serde(rename = "APIKey")]
    pub api_key: Option<String>,
}

/// A ticker row inside a VolumeLeaders watchlist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WatchListTicker {
    pub watch_list_key: Option<i64>,
    pub security_key: Option<i64>,
    pub ticker: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_cap: Option<f64>,
    pub price: Option<f64>,
    pub nearest_top10_trade_date: Option<AspNetDate>,
    pub nearest_top10_trade_price: Option<f64>,
    pub nearest_top10_trade_volume: Option<i64>,
    pub nearest_top10_trade_dollars: Option<f64>,
    pub nearest_top10_trade_rank: Option<i64>,
    pub nearest_top10_trade_cluster_date: Option<AspNetDate>,
    pub nearest_top10_trade_cluster_price: Option<f64>,
    pub nearest_top10_trade_cluster_volume: Option<i64>,
    pub nearest_top10_trade_cluster_dollars: Option<f64>,
    pub nearest_top10_trade_cluster_rank: Option<i64>,
    pub nearest_top10_trade_level: Option<f64>,
    pub nearest_top10_trade_level_date: Option<AspNetDate>,
    pub nearest_top10_trade_level_price: Option<f64>,
    pub nearest_top10_trade_level_volume: Option<i64>,
    pub nearest_top10_trade_level_dollars: Option<f64>,
    pub nearest_top10_trade_level_rank: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watchlist_config_deserializes_nullable_rsi_fields() {
        let config: WatchListConfig = serde_json::from_str(
            r#"{"SearchTemplateKey":6307,"Name":"Testing","Tickers":"AMD,NVDA","Criteria":"Volume >= 1000000","RSIOverboughtHourly":70,"RSIOverboughtDaily":null,"RSIOverboughtHourlySelected":true,"APIKey":null}"#,
        )
        .unwrap();

        assert_eq!(config.search_template_key, Some(6307));
        assert_eq!(config.name.as_deref(), Some("Testing"));
        assert_eq!(config.tickers.as_deref(), Some("AMD,NVDA"));
        assert_eq!(config.criteria.as_deref(), Some("Volume >= 1000000"));
        assert_eq!(config.rsi_overbought_hourly, Some(70));
        assert_eq!(config.rsi_overbought_daily, None);
        assert_eq!(config.rsi_overbought_hourly_selected, Some(true));
        assert_eq!(config.api_key, None);
    }

    #[test]
    fn watchlist_ticker_deserializes_captured_schema() {
        let ticker: WatchListTicker = serde_json::from_str(
            r#"{"WatchListKey":0,"SecurityKey":63,"Ticker":"AAPL","NearestTop10TradeDate":"/Date(1766102400000)/","NearestTop10TradeLevel":273.7}"#,
        )
        .unwrap();

        assert_eq!(ticker.watch_list_key, Some(0));
        assert_eq!(ticker.security_key, Some(63));
        assert_eq!(ticker.ticker.as_deref(), Some("AAPL"));
        assert!(ticker.nearest_top10_trade_date.is_some());
        assert_eq!(ticker.nearest_top10_trade_level, Some(273.7));
    }
}
