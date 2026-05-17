//! VolumeLeaders exhaustion score model.

use serde::{Deserialize, Serialize};

/// Market exhaustion rank data from VolumeLeaders.
///
/// All fields are `Option` to handle missing or null values from the API.
/// Field names match the API's PascalCase convention via `rename_all`, with
/// explicit `rename` overrides for numeric suffixes that `PascalCase` would
/// mangle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExhaustionScore {
    pub date_key: Option<i64>,
    pub exhaustion_score_rank: Option<i64>,
    #[serde(rename = "ExhaustionScoreRank30Day")]
    pub exhaustion_score_rank_30_day: Option<i64>,
    #[serde(rename = "ExhaustionScoreRank90Day")]
    pub exhaustion_score_rank_90_day: Option<i64>,
    #[serde(rename = "ExhaustionScoreRank365Day")]
    pub exhaustion_score_rank_365_day: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_exhaustion_score_from_json() {
        let json = r#"{
            "DateKey": 20260501,
            "ExhaustionScoreRank": 4,
            "ExhaustionScoreRank30Day": 8,
            "ExhaustionScoreRank90Day": 11,
            "ExhaustionScoreRank365Day": 22
        }"#;
        let score: ExhaustionScore = serde_json::from_str(json).unwrap();
        assert_eq!(score.date_key, Some(20_260_501));
        assert_eq!(score.exhaustion_score_rank, Some(4));
        assert_eq!(score.exhaustion_score_rank_30_day, Some(8));
        assert_eq!(score.exhaustion_score_rank_90_day, Some(11));
        assert_eq!(score.exhaustion_score_rank_365_day, Some(22));
    }

    #[test]
    fn deserialize_exhaustion_score_with_nulls() {
        let json = r#"{
            "DateKey": null,
            "ExhaustionScoreRank": null,
            "ExhaustionScoreRank30Day": null,
            "ExhaustionScoreRank90Day": null,
            "ExhaustionScoreRank365Day": null
        }"#;
        let score: ExhaustionScore = serde_json::from_str(json).unwrap();
        assert!(score.date_key.is_none());
        assert!(score.exhaustion_score_rank.is_none());
    }
}
