//! VolumeLeaders trade level model.

use serde::{Deserialize, Serialize};

use super::types::AspNetDate;

/// A single VolumeLeaders trade level row.
///
/// Used for both trade levels and trade level touches endpoints.
/// All fields are `Option` to handle missing or null values from the API.
/// Field names match the API's PascalCase convention via `rename_all`, with
/// explicit `rename` overrides for digit suffixes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TradeLevel {
    // -- Security info --
    pub ticker: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub name: Option<String>,

    // -- Dates --
    pub date: Option<AspNetDate>,
    pub min_date: Option<AspNetDate>,
    pub max_date: Option<AspNetDate>,
    pub full_date_time: Option<String>,
    #[serde(rename = "FullTimeString24")]
    pub full_time_string_24: Option<String>,
    pub dates: Option<String>,

    // -- Price / size --
    pub price: Option<f64>,
    pub dollars: Option<f64>,
    pub volume: Option<i64>,
    pub trades: Option<i64>,
    pub relative_size: Option<f64>,

    // -- Rank / distribution --
    pub cumulative_distribution: Option<f64>,
    pub trade_level_rank: Option<i64>,
    pub trade_level_touches: Option<i64>,

    // -- Metadata --
    pub total_rows: Option<i64>,
}
