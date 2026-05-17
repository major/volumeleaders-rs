//! VolumeLeaders alert configuration and alert row models.

use serde::{Deserialize, Serialize};

use super::types::{AspNetDate, FlexBool};

/// A saved VolumeLeaders alert configuration row.
///
/// All fields are optional because the browser DataTables payload may omit or
/// null fields depending on which alert criteria are enabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AlertConfig {
    pub alert_config_key: Option<i64>,
    pub user_key: Option<i64>,
    pub name: Option<String>,
    pub tickers: Option<String>,
    #[serde(rename = "TradeRankLTE")]
    pub trade_rank_lte: Option<i64>,
    #[serde(rename = "TradeVCDGTE")]
    pub trade_vcd_gte: Option<f64>,
    #[serde(rename = "TradeMultGTE")]
    pub trade_mult_gte: Option<f64>,
    #[serde(rename = "TradeVolumeGTE")]
    pub trade_volume_gte: Option<i64>,
    #[serde(rename = "TradeDollarsGTE")]
    pub trade_dollars_gte: Option<f64>,
    pub trade_conditions: Option<String>,
    #[serde(rename = "TradeClusterRankLTE")]
    pub trade_cluster_rank_lte: Option<i64>,
    #[serde(rename = "TradeClusterVCDGTE")]
    pub trade_cluster_vcd_gte: Option<f64>,
    #[serde(rename = "TradeClusterMultGTE")]
    pub trade_cluster_mult_gte: Option<f64>,
    #[serde(rename = "TradeClusterVolumeGTE")]
    pub trade_cluster_volume_gte: Option<i64>,
    #[serde(rename = "TradeClusterDollarsGTE")]
    pub trade_cluster_dollars_gte: Option<f64>,
    #[serde(rename = "TotalRankLTE")]
    pub total_rank_lte: Option<i64>,
    #[serde(rename = "TotalVolumeGTE")]
    pub total_volume_gte: Option<i64>,
    #[serde(rename = "TotalDollarsGTE")]
    pub total_dollars_gte: Option<f64>,
    #[serde(rename = "AHRankLTE")]
    pub ah_rank_lte: Option<i64>,
    #[serde(rename = "AHVolumeGTE")]
    pub ah_volume_gte: Option<i64>,
    #[serde(rename = "AHDollarsGTE")]
    pub ah_dollars_gte: Option<f64>,
    #[serde(rename = "ClosingTradeRankLTE")]
    pub closing_trade_rank_lte: Option<i64>,
    #[serde(rename = "ClosingTradeVCDGTE")]
    pub closing_trade_vcd_gte: Option<f64>,
    #[serde(rename = "ClosingTradeMultGTE")]
    pub closing_trade_mult_gte: Option<f64>,
    #[serde(rename = "ClosingTradeVolumeGTE")]
    pub closing_trade_volume_gte: Option<i64>,
    #[serde(rename = "ClosingTradeDollarsGTE")]
    pub closing_trade_dollars_gte: Option<f64>,
    pub closing_trade_conditions: Option<String>,
    pub offsetting_print: Option<bool>,
    pub phantom_print: Option<bool>,
    pub sweep: Option<bool>,
    pub dark_pool: Option<bool>,
}

/// A VolumeLeaders trade alert row with alert metadata and trade details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradeAlert {
    pub date: Option<AspNetDate>,
    pub start_date: Option<AspNetDate>,
    pub end_date: Option<AspNetDate>,
    #[serde(rename = "FullTimeString24")]
    pub full_time_string_24: Option<String>,
    pub date_key: Option<i64>,
    pub security_key: Option<i64>,
    pub time_key: Option<i64>,
    #[serde(rename = "TradeID")]
    pub trade_id: Option<i64>,
    pub sequence_number: Option<i64>,
    pub user_key: Option<i64>,
    pub user_keys: Option<String>,
    pub sent: Option<bool>,
    pub email: Option<String>,
    pub emails: Option<String>,
    pub ticker: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub name: Option<String>,
    pub alert_type: Option<String>,
    pub price: Option<f64>,
    pub trade_rank: Option<i64>,
    pub volume_cumulative_distribution: Option<f64>,
    pub dollars_multiplier: Option<f64>,
    pub volume: Option<i64>,
    pub dollars: Option<f64>,
    pub last_comparible_trade_date_key: Option<i64>,
    pub last_comparible_trade_date: Option<AspNetDate>,
    pub offsetting_trade_date: Option<AspNetDate>,
    pub phantom_print_fulfillment_date: Option<AspNetDate>,
    pub full_date_time: Option<String>,
    #[serde(rename = "IPODate")]
    pub ipo_date: Option<AspNetDate>,
    #[serde(rename = "RSIHour")]
    pub rsi_hour: Option<f64>,
    #[serde(rename = "RSIDay")]
    pub rsi_day: Option<f64>,
    pub in_process: Option<bool>,
    pub complete: Option<bool>,
    pub sweep: Option<FlexBool>,
    pub dark_pool: Option<FlexBool>,
    pub late_print: Option<FlexBool>,
    pub closing_trade: Option<FlexBool>,
    pub signature_print: Option<FlexBool>,
    pub phantom_print: Option<FlexBool>,
}

/// A VolumeLeaders trade cluster alert row.
///
/// The alert endpoint returns the same shape as trade cluster rows, but the
/// dedicated type keeps the alerts API surface self-documenting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradeClusterAlert {
    pub date: Option<AspNetDate>,
    #[serde(rename = "IPODate")]
    pub ipo_date: Option<AspNetDate>,
    pub date_key: Option<i64>,
    pub security_key: Option<i64>,
    pub ticker: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub name: Option<String>,
    pub min_full_date_time: Option<String>,
    pub max_full_date_time: Option<String>,
    #[serde(rename = "MinFullTimeString24")]
    pub min_full_time_string_24: Option<String>,
    #[serde(rename = "MaxFullTimeString24")]
    pub max_full_time_string_24: Option<String>,
    pub price: Option<f64>,
    pub close_price: Option<f64>,
    pub dollars: Option<f64>,
    pub average_block_size_shares: Option<i64>,
    pub average_block_size_dollars: Option<f64>,
    pub volume: Option<i64>,
    pub average_daily_volume: Option<i64>,
    pub trade_count: Option<i64>,
    pub dollars_multiplier: Option<f64>,
    pub cumulative_distribution: Option<f64>,
    pub trade_cluster_rank: Option<i64>,
    pub last_comparible_trade_cluster_date: Option<AspNetDate>,
    #[serde(rename = "EOM")]
    pub eom: Option<FlexBool>,
    #[serde(rename = "EOQ")]
    pub eoq: Option<FlexBool>,
    #[serde(rename = "EOY")]
    pub eoy: Option<FlexBool>,
    #[serde(rename = "OPEX")]
    pub opex: Option<FlexBool>,
    #[serde(rename = "VOLEX")]
    pub volex: Option<FlexBool>,
    pub inside_bar: Option<FlexBool>,
    pub double_inside_bar: Option<FlexBool>,
    pub total_rows: Option<i64>,
    pub external_feed: Option<FlexBool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_config_deserializes_nullable_criteria() {
        let alert: AlertConfig = serde_json::from_str(
            r#"{"AlertConfigKey":42088,"Name":"testing","TradeConditions":null,"Sweep":true}"#,
        )
        .unwrap();

        assert_eq!(alert.alert_config_key, Some(42088));
        assert_eq!(alert.name.as_deref(), Some("testing"));
        assert_eq!(alert.trade_conditions, None);
        assert_eq!(alert.sweep, Some(true));
    }

    #[test]
    fn trade_alert_deserializes_flex_bool_flags() {
        let alert: TradeAlert = serde_json::from_str(
            r#"{"Ticker":"AMD","TradeID":123456,"AlertType":"Trade","Sector":null,"Sweep":1,"DarkPool":false}"#,
        )
        .unwrap();

        assert_eq!(alert.ticker.as_deref(), Some("AMD"));
        assert_eq!(alert.trade_id, Some(123456));
        assert_eq!(alert.sector, None);
        assert_eq!(alert.sweep, Some(FlexBool(Some(true))));
        assert_eq!(alert.dark_pool, Some(FlexBool(Some(false))));
    }

    #[test]
    fn trade_cluster_alert_deserializes_cluster_shape() {
        let alert: TradeClusterAlert =
            serde_json::from_str(r#"{"Ticker":"AMD","TradeClusterRank":8,"TradeCount":4}"#)
                .unwrap();

        assert_eq!(alert.ticker.as_deref(), Some("AMD"));
        assert_eq!(alert.trade_cluster_rank, Some(8));
        assert_eq!(alert.trade_count, Some(4));
    }
}
