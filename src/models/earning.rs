//! VolumeLeaders earnings model.

use serde::{Deserialize, Serialize};

use super::types::AspNetDate;

/// A single VolumeLeaders earnings row.
///
/// All fields are `Option` to handle missing or null values from the API.
/// Field names match the API's PascalCase convention via `rename_all`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Earning {
    // -- Dates --
    pub date: Option<AspNetDate>,
    pub earnings_date: Option<AspNetDate>,

    // -- Security info --
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub current: Option<f64>,
    pub sector: Option<String>,
    pub industry: Option<String>,

    // -- Earnings flags --
    pub after_market_close: Option<bool>,

    // -- Counts --
    pub trade_count: Option<i64>,
    pub trade_cluster_count: Option<i64>,
    pub trade_cluster_bomb_count: Option<i64>,

    // -- Metadata --
    pub total_rows: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper struct matching the DataTables response envelope.
    #[derive(Deserialize)]
    struct EarningsResponse {
        data: Vec<Earning>,
    }

    #[test]
    fn deserialize_earnings_from_fixture() {
        let json = include_str!("../../tests/fixtures/earnings_response.json");
        let response: EarningsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 2);

        let first = &response.data[0];
        assert_eq!(first.ticker.as_deref(), Some("AMD"));
        assert_eq!(first.name.as_deref(), Some("Advanced Micro Devices"));
        assert_eq!(first.current, Some(220.25));
        assert_eq!(first.sector.as_deref(), Some("Technology"));
        assert_eq!(first.industry.as_deref(), Some("Semiconductors"));
        assert_eq!(first.trade_count, Some(9));
        assert_eq!(first.trade_cluster_count, Some(5));
        assert_eq!(first.trade_cluster_bomb_count, Some(2));
        assert_eq!(first.after_market_close, Some(true));

        // EarningsDate with valid timestamp
        let earnings_date = first.earnings_date.as_ref().unwrap();
        assert!(earnings_date.0.is_some());
    }

    #[test]
    fn deserialize_earning_nullable_fields() {
        let json = r#"{
            "Ticker": "NVDA",
            "Sector": null,
            "Industry": null
        }"#;
        let earning: Earning = serde_json::from_str(json).unwrap();
        assert_eq!(earning.ticker.as_deref(), Some("NVDA"));
        assert!(earning.sector.is_none());
        assert!(earning.industry.is_none());
    }
}
