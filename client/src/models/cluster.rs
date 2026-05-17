//! VolumeLeaders trade cluster and trade cluster bomb models.

use serde::{Deserialize, Serialize};

use super::types::{AspNetDate, FlexBool};

/// A single VolumeLeaders trade cluster row.
///
/// All fields are `Option` to handle missing or null values from the API.
/// Field names match the API's PascalCase convention via `rename_all`, with
/// explicit `rename` overrides for all-caps abbreviations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradeCluster {
    // -- Dates --
    pub date: Option<AspNetDate>,
    #[serde(rename = "IPODate")]
    pub ipo_date: Option<AspNetDate>,

    // -- Keys --
    pub date_key: Option<i64>,
    pub security_key: Option<i64>,

    // -- Security info --
    pub ticker: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub name: Option<String>,

    // -- Time range --
    pub min_full_date_time: Option<String>,
    pub max_full_date_time: Option<String>,
    #[serde(rename = "MinFullTimeString24")]
    pub min_full_time_string_24: Option<String>,
    #[serde(rename = "MaxFullTimeString24")]
    pub max_full_time_string_24: Option<String>,

    // -- Price / size --
    pub price: Option<f64>,
    pub close_price: Option<f64>,
    pub dollars: Option<f64>,
    pub average_block_size_shares: Option<i64>,
    pub average_block_size_dollars: Option<f64>,
    pub volume: Option<i64>,
    pub average_daily_volume: Option<i64>,

    // -- Rank / distribution --
    pub trade_count: Option<i64>,
    pub dollars_multiplier: Option<f64>,
    pub cumulative_distribution: Option<f64>,
    pub trade_cluster_rank: Option<i64>,

    // -- Reference dates --
    pub last_comparible_trade_cluster_date: Option<AspNetDate>,

    // -- Calendar flags --
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

    // -- Metadata --
    pub total_rows: Option<i64>,
    pub external_feed: Option<FlexBool>,
}

/// A single VolumeLeaders trade cluster bomb row.
///
/// All fields are `Option` to handle missing or null values from the API.
/// Field names match the API's PascalCase convention via `rename_all`, with
/// explicit `rename` overrides for all-caps abbreviations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradeClusterBomb {
    // -- Dates --
    pub date: Option<AspNetDate>,
    #[serde(rename = "IPODate")]
    pub ipo_date: Option<AspNetDate>,

    // -- Keys --
    pub date_key: Option<i64>,
    pub security_key: Option<i64>,

    // -- Security info --
    pub ticker: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub name: Option<String>,

    // -- Time range --
    pub min_full_date_time: Option<String>,
    pub max_full_date_time: Option<String>,
    #[serde(rename = "MinFullTimeString24")]
    pub min_full_time_string_24: Option<String>,
    #[serde(rename = "MaxFullTimeString24")]
    pub max_full_time_string_24: Option<String>,

    // -- Price / size --
    pub close_price: Option<f64>,
    pub dollars: Option<f64>,
    pub average_block_size_shares: Option<i64>,
    pub average_block_size_dollars: Option<f64>,
    pub volume: Option<i64>,
    pub average_daily_volume: Option<i64>,

    // -- Rank / distribution --
    pub trade_count: Option<i64>,
    pub dollars_multiplier: Option<f64>,
    pub cumulative_distribution: Option<f64>,
    pub trade_cluster_bomb_rank: Option<i64>,

    // -- Reference dates --
    pub last_comparable_trade_cluster_bomb_date: Option<AspNetDate>,

    // -- Calendar flags --
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

    // -- Metadata --
    pub total_rows: Option<i64>,
    pub external_feed: Option<FlexBool>,
}
