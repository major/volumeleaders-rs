//! VolumeLeaders institutional trade model.

use serde::{Deserialize, Serialize};

use super::types::{AspNetDate, FlexBool};

/// A single VolumeLeaders institutional trade row.
///
/// All fields are `Option` to handle missing or null values from the API.
/// Field names match the API's PascalCase convention via `rename_all`, with
/// explicit `rename` overrides for all-caps abbreviations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Trade {
    // -- Dates --
    pub date: Option<AspNetDate>,
    pub start_date: Option<AspNetDate>,
    pub end_date: Option<AspNetDate>,
    #[serde(rename = "TD30")]
    pub td_30: Option<AspNetDate>,
    #[serde(rename = "TD90")]
    pub td_90: Option<AspNetDate>,
    #[serde(rename = "TD1CY")]
    pub td_1cy: Option<AspNetDate>,

    // -- Keys --
    pub date_key: Option<i64>,
    pub time_key: Option<i64>,
    pub security_key: Option<i64>,
    #[serde(rename = "TradeID")]
    pub trade_id: Option<i64>,
    pub sequence_number: Option<i64>,

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

    // -- Security info --
    pub ticker: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub name: Option<String>,
    pub full_date_time: Option<String>,
    #[serde(rename = "FullTimeString24")]
    pub full_time_string_24: Option<String>,

    // -- Price / size --
    pub price: Option<f64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub dollars: Option<f64>,
    pub average_block_size_dollars: Option<f64>,
    pub average_block_size_shares: Option<i64>,
    pub dollars_multiplier: Option<f64>,
    pub volume: Option<i64>,
    pub average_daily_volume: Option<i64>,
    pub percent_daily_volume: Option<f64>,
    pub relative_size: Option<f64>,

    // -- Reference dates --
    pub last_comparible_trade_date: Option<AspNetDate>,
    #[serde(rename = "IPODate")]
    pub ipo_date: Option<AspNetDate>,
    pub offsetting_trade_date: Option<AspNetDate>,
    pub phantom_print_fulfillment_date: Option<AspNetDate>,
    pub phantom_print_fulfillment_days: Option<i64>,

    // -- Rank / distribution --
    pub trade_count: Option<i64>,
    pub cumulative_distribution: Option<f64>,
    pub trade_rank: Option<i64>,
    pub trade_rank_snapshot: Option<i64>,

    // -- Trade flags --
    pub late_print: Option<FlexBool>,
    pub sweep: Option<FlexBool>,
    pub dark_pool: Option<FlexBool>,
    pub opening_trade: Option<FlexBool>,
    pub closing_trade: Option<FlexBool>,
    pub phantom_print: Option<FlexBool>,
    pub inside_bar: Option<FlexBool>,
    pub double_inside_bar: Option<FlexBool>,
    pub signature_print: Option<FlexBool>,
    pub new_position: Option<FlexBool>,

    // -- After-hours / institutional aggregates --
    #[serde(rename = "AHInstitutionalDollars")]
    pub ah_institutional_dollars: Option<f64>,
    #[serde(rename = "AHInstitutionalDollarsRank")]
    pub ah_institutional_dollars_rank: Option<i64>,
    #[serde(rename = "AHInstitutionalVolume")]
    pub ah_institutional_volume: Option<i64>,
    pub total_institutional_dollars: Option<f64>,
    pub total_institutional_dollars_rank: Option<i64>,
    pub total_institutional_volume: Option<i64>,

    // -- Closing / total aggregates --
    pub closing_trade_dollars: Option<f64>,
    pub closing_trade_dollars_rank: Option<i64>,
    pub closing_trade_volume: Option<i64>,
    pub total_dollars: Option<f64>,
    pub total_dollars_rank: Option<i64>,
    pub total_volume: Option<i64>,

    // -- Indicators --
    pub close_price: Option<f64>,
    #[serde(rename = "RSIHour")]
    pub rsi_hour: Option<f64>,
    #[serde(rename = "RSIDay")]
    pub rsi_day: Option<f64>,

    // -- Metadata --
    pub total_rows: Option<i64>,
    pub trade_conditions: Option<String>,
    #[serde(rename = "FrequencyLast30TD")]
    pub frequency_last_30_td: Option<i64>,
    #[serde(rename = "FrequencyLast90TD")]
    pub frequency_last_90_td: Option<i64>,
    #[serde(rename = "FrequencyLast1CY")]
    pub frequency_last_1cy: Option<i64>,
    pub cancelled: Option<FlexBool>,
    pub total_trades: Option<i64>,
    pub external_feed: Option<FlexBool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper struct matching the DataTables response envelope.
    #[derive(Deserialize)]
    struct TradesResponse {
        data: Vec<Trade>,
    }

    #[test]
    fn deserialize_trades_from_fixture() {
        let json = include_str!("../../tests/fixtures/trades_get_trades_response.json");
        let response: TradesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 2);

        let first = &response.data[0];
        assert_eq!(first.ticker.as_deref(), Some("AXP"));
        assert_eq!(first.sector.as_deref(), Some("Financial Services"));
        assert_eq!(first.industry.as_deref(), Some("Consumer Finance"));
        assert_eq!(first.trade_id, Some(71_774_613_157_188));
        assert_eq!(first.price, Some(319.68));
        assert_eq!(first.volume, Some(276_248));

        // FlexBool from integer 1 -> true
        assert_eq!(first.dark_pool, Some(FlexBool(Some(true))));
        // FlexBool from integer 0 -> false
        assert_eq!(first.late_print, Some(FlexBool(Some(false))));
        // FlexBool from JSON bool
        assert_eq!(first.eom, Some(FlexBool(Some(false))));

        // AspNetDate sentinel (.NET min) -> None inner
        let td30 = first.td_30.as_ref().unwrap();
        assert!(td30.0.is_none());

        // AspNetDate with valid timestamp
        let date = first.date.as_ref().unwrap();
        assert!(date.0.is_some());

        // Null fields -> outer Option is None
        assert!(first.phantom_print_fulfillment_days.is_none());
        assert!(first.trade_conditions.is_none());
    }

    #[test]
    fn deserialize_trade_empty_phantom_date() {
        let json = include_str!("../../tests/fixtures/trades_get_trades_response.json");
        let response: TradesResponse = serde_json::from_str(json).unwrap();
        let second = &response.data[1];

        // PhantomPrintFulfillmentDate is "" in fixture -> Some(AspNetDate(None))
        let phantom = second.phantom_print_fulfillment_date.as_ref().unwrap();
        assert!(phantom.0.is_none());

        // Verify second trade has different ticker
        assert_eq!(second.ticker.as_deref(), Some("MRVL"));
        // EOM is true (JSON bool) in second trade
        assert_eq!(second.eom, Some(FlexBool(Some(true))));
    }

    #[test]
    fn deserialize_trade_nullable_strings_null() {
        let json = r#"{
            "Industry": null,
            "FullDateTime": null,
            "FullTimeString24": null
        }"#;
        let trade: Trade = serde_json::from_str(json).unwrap();
        assert!(trade.industry.is_none());
        assert!(trade.full_date_time.is_none());
        assert!(trade.full_time_string_24.is_none());
    }

    #[test]
    fn deserialize_trade_nullable_strings_present() {
        let json = r#"{
            "Industry": "Consumer Finance",
            "FullDateTime": "2026-05-01T16:20:51",
            "FullTimeString24": "16:20:51"
        }"#;
        let trade: Trade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.industry.as_deref(), Some("Consumer Finance"));
        assert_eq!(trade.full_date_time.as_deref(), Some("2026-05-01T16:20:51"));
        assert_eq!(trade.full_time_string_24.as_deref(), Some("16:20:51"));
    }
}
