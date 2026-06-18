//! Shared semantic transforms for trade-shaped agent output.

use serde::Serialize;
use serde_json::{Map, Value};

/// Trade-shaped row families that share output transforms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TradeRecordKind {
    /// Individual institutional trade rows.
    Trade,
    /// Aggregated trade cluster rows.
    Cluster,
    /// Trade price-level rows.
    Level,
    /// Trade cluster bomb rows.
    ClusterBomb,
}

const DROPPED_FIELDS: &[&str] = &["EOM", "EOQ", "EOY", "OPEX", "VOLEX", "RSIDay", "RSIHour"];

/// Default compact column headers for trade-shaped output.
///
/// Shared by commands that display individual institutional trades (e.g. `trade
/// list` and `report` presets).
pub const TRADE_HEADERS: &[&str] = &[
    "Ticker",
    "Date",
    "Time",
    "Price",
    "Dollars",
    "RelativeSize",
    "CumulativeDistribution",
    "TradeRank",
    "type",
    "venue",
    "Sector",
    "Industry",
];

/// Serialize records and apply the transforms for their trade-shaped row kind.
pub fn transformed_trade_values<T: Serialize>(
    records: &[T],
    kind: TradeRecordKind,
) -> serde_json::Result<Vec<Value>> {
    let mut values: Vec<Value> = records
        .iter()
        .map(serde_json::to_value)
        .collect::<serde_json::Result<_>>()?;
    transform_trade_values(&mut values, kind);
    Ok(values)
}

/// Apply transforms to already-serialized trade-shaped record values.
pub fn transform_trade_values(values: &mut [Value], kind: TradeRecordKind) {
    for value in values {
        let Some(row) = value.as_object_mut() else {
            continue;
        };
        transform_trade_row(row, kind);
    }
}

/// Apply semantic transforms to trade dashboard sections before field filtering.
pub fn transform_trade_dashboard(map: &mut Map<String, Value>) {
    for (section, kind) in [
        ("trades", TradeRecordKind::Trade),
        ("clusters", TradeRecordKind::Cluster),
        ("levels", TradeRecordKind::Level),
        ("cluster_bombs", TradeRecordKind::ClusterBomb),
    ] {
        let Some(Value::Array(rows)) = map.get_mut(section) else {
            continue;
        };
        transform_trade_values(rows, kind);
    }
}

/// Apply semantic transforms to one trade-shaped row.
pub fn transform_trade_row(row: &mut Map<String, Value>, kind: TradeRecordKind) {
    match kind {
        TradeRecordKind::Trade => {
            collapse_trade_type(row);
            collapse_venue(row);
            rename_trade_time_fields(row);
        }
        TradeRecordKind::Cluster | TradeRecordKind::ClusterBomb => {
            rename_cluster_time_fields(row);
            collapse_time_window(row);
        }
        TradeRecordKind::Level => {}
    }
    drop_noisy_fields(row);
    strip_question_marks(row);
    if kind == TradeRecordKind::Trade {
        alias_trade_relative_size(row);
        omit_trade_dollars_multiplier(row);
    }
    compact_date_timezone(row);
}

/// Remove calendar marker and RSI fields from CLI output.
fn drop_noisy_fields(row: &mut Map<String, Value>) {
    for &field in DROPPED_FIELDS {
        row.remove(field);
    }
}

/// Remove fields whose value is the literal string `"?"`.
fn strip_question_marks(row: &mut Map<String, Value>) {
    row.retain(|_, v| v.as_str() != Some("?"));
}

/// Surface the browser RS value under the user-facing `RelativeSize` field.
///
/// `/Trades/GetTrades` trade rows often return `RelativeSize` as null while the
/// browser displays its RS column from `DollarsMultiplier`. Preserve any real
/// upstream `RelativeSize` value, but fill null or missing values from the
/// column the site actually renders.
fn alias_trade_relative_size(row: &mut Map<String, Value>) {
    let relative_size_missing = row.get("RelativeSize").is_none_or(Value::is_null);
    if relative_size_missing
        && let Some(value) = row.get("DollarsMultiplier").cloned()
        && !value.is_null()
    {
        row.insert("RelativeSize".to_string(), value);
    }
}

/// Remove the raw API field after `RelativeSize` has been populated for trade rows.
fn omit_trade_dollars_multiplier(row: &mut Map<String, Value>) {
    row.remove("DollarsMultiplier");
}

/// Collapse `OpeningTrade` and `ClosingTrade` booleans into a single
/// `"type"` field: `"opening"`, `"closing"`, or omitted when neither.
fn collapse_trade_type(row: &mut Map<String, Value>) {
    let opening = row
        .remove("OpeningTrade")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let closing = row
        .remove("ClosingTrade")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if opening {
        row.insert("type".to_string(), Value::String("opening".to_string()));
    } else if closing {
        row.insert("type".to_string(), Value::String("closing".to_string()));
    }
}

/// Collapse `DarkPool` and `Sweep` booleans into a single `"venue"` field.
fn collapse_venue(row: &mut Map<String, Value>) {
    let dark_pool = row
        .remove("DarkPool")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let sweep = row
        .remove("Sweep")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let venue = match (dark_pool, sweep) {
        (false, false) => return,
        (false, true) => "lit_sweep",
        (true, false) => "dark_pool",
        (true, true) => "dark_pool_sweep",
    };
    row.insert("venue".to_string(), Value::String(venue.to_string()));
}

/// Rename verbose trade time fields to shorter display names.
///
/// `FullTimeString24` -> `Time`, `FullDateTime` -> `DateTime`.
fn rename_trade_time_fields(row: &mut Map<String, Value>) {
    if let Some(v) = row.remove("FullTimeString24") {
        row.insert("Time".to_string(), v);
    }
    if let Some(v) = row.remove("FullDateTime") {
        row.insert("DateTime".to_string(), v);
    }
}

/// Rename verbose cluster/bomb time fields to shorter display names.
///
/// Runs before `collapse_time_window` so both the collapsed `window` path
/// and the `--all-fields` raw path see consistent short names.
fn rename_cluster_time_fields(row: &mut Map<String, Value>) {
    for (old, new) in [
        ("MinFullDateTime", "MinDateTime"),
        ("MaxFullDateTime", "MaxDateTime"),
        ("MinFullTimeString24", "MinTime"),
        ("MaxFullTimeString24", "MaxTime"),
    ] {
        if let Some(v) = row.remove(old) {
            row.insert(new.to_string(), v);
        }
    }
}

/// Compact date-time string values.
fn compact_date_timezone(row: &mut Map<String, Value>) {
    for value in row.values_mut() {
        let Some(s) = value.as_str() else { continue };
        if let Some(prefix) = s.strip_suffix("+00:00") {
            if let Some(date) = prefix.strip_suffix("T00:00:00") {
                *value = Value::String(date.to_string());
            } else {
                *value = Value::String(format!("{prefix}Z"));
            }
        } else if let Some(date) = s.strip_suffix("T00:00:00Z") {
            *value = Value::String(date.to_string());
        }
    }
}

/// Collapse `MinDateTime` and `MaxDateTime` into a `"window"` field.
///
/// Runs after `rename_cluster_time_fields` so the keys are already shortened.
fn collapse_time_window(row: &mut Map<String, Value>) {
    let extract_time = |v: &Value| -> Option<String> {
        let s = v.as_str()?;
        let after_t = s.split('T').nth(1)?;
        let time = after_t
            .strip_suffix("+00:00")
            .or_else(|| after_t.strip_suffix('Z'))
            .unwrap_or(after_t);
        Some(time.to_string())
    };

    let min_time = row.get("MinDateTime").and_then(&extract_time);
    let max_time = row.get("MaxDateTime").and_then(&extract_time);

    if let (Some(min), Some(max)) = (min_time, max_time) {
        row.remove("MinDateTime");
        row.remove("MaxDateTime");
        row.insert("window".to_string(), Value::String(format!("{min}-{max}")));
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use serde_json::json;

    use super::*;

    #[test]
    fn trade_transform_collapses_trade_semantics() {
        let mut value = json!({
            "FullTimeString24": "16:00:00",
            "Dollars": 10.126,
            "TradeRank": 9999,
            "DarkPool": true,
            "Sweep": true,
            "ClosingTrade": true,
            "OPEX": true,
            "EOM": false,
            "RSIDay": 72.456,
            "RSIHour": 65.123
        });

        let row = value.as_object_mut().unwrap();
        transform_trade_row(row, TradeRecordKind::Trade);

        assert_eq!(row["Dollars"], 10.126);
        assert_eq!(row["type"], "closing");
        assert_eq!(row["venue"], "dark_pool_sweep");
        assert!(!row.contains_key("events"));
        assert!(!row.contains_key("OPEX"));
        assert!(!row.contains_key("EOM"));
        assert!(!row.contains_key("RSIDay"));
        assert!(!row.contains_key("RSIHour"));
        assert!(!row.contains_key("FullTimeString24"));
        assert_eq!(row["Time"], "16:00:00");
        assert_eq!(row["TradeRank"], 9999);
        assert!(!row.contains_key("DarkPool"));
        assert!(!row.contains_key("Sweep"));
        assert!(!row.contains_key("ClosingTrade"));
    }

    #[test]
    fn trade_transform_aliases_relative_size_from_dollars_multiplier_when_source_is_null() {
        let mut value = json!({
            "DollarsMultiplier": 8.041,
            "RelativeSize": null
        });

        let row = value.as_object_mut().unwrap();
        transform_trade_row(row, TradeRecordKind::Trade);

        assert_eq!(row["RelativeSize"], 8.041);
        assert!(!row.contains_key("DollarsMultiplier"));
    }

    #[test]
    fn trade_transform_renames_time_fields() {
        let mut value = json!({
            "FullTimeString24": "14:30:00",
            "FullDateTime": "2026-01-02T14:30:00",
            "Ticker": "AAPL"
        });

        let row = value.as_object_mut().unwrap();
        transform_trade_row(row, TradeRecordKind::Trade);

        assert_eq!(row["Time"], "14:30:00");
        assert_eq!(row["DateTime"], "2026-01-02T14:30:00");
        assert!(!row.contains_key("FullTimeString24"));
        assert!(!row.contains_key("FullDateTime"));
    }

    #[test]
    fn cluster_transform_collapses_time_window() {
        let mut value = json!({
            "MinFullDateTime": "2026-01-02T16:00:00+00:00",
            "MaxFullDateTime": "2026-01-02T16:49:31+00:00",
            "TradeClusterRank": 2
        });

        let row = value.as_object_mut().unwrap();
        transform_trade_row(row, TradeRecordKind::Cluster);

        assert_eq!(row["window"], "16:00:00-16:49:31");
        assert!(!row.contains_key("MinFullDateTime"));
        assert!(!row.contains_key("MaxFullDateTime"));
        assert!(!row.contains_key("MinDateTime"));
        assert!(!row.contains_key("MaxDateTime"));
    }

    #[test]
    fn cluster_transform_renames_time_fields_for_all_fields() {
        let mut value = json!({
            "MinFullTimeString24": "16:00:00",
            "MaxFullTimeString24": "16:49:31",
            "Ticker": "AAPL"
        });

        let row = value.as_object_mut().unwrap();
        transform_trade_row(row, TradeRecordKind::Cluster);

        assert_eq!(row["MinTime"], "16:00:00");
        assert_eq!(row["MaxTime"], "16:49:31");
        assert!(!row.contains_key("MinFullTimeString24"));
        assert!(!row.contains_key("MaxFullTimeString24"));
    }

    #[test]
    fn drop_noisy_fields_removes_calendar_and_rsi_fields() {
        let mut value = json!({
            "RSIDay": 55.5,
            "RSIHour": 42.0,
            "EOM": true,
            "Ticker": "AAPL"
        });
        let row = value.as_object_mut().unwrap();
        drop_noisy_fields(row);

        assert!(!row.contains_key("RSIDay"));
        assert!(!row.contains_key("RSIHour"));
        assert!(!row.contains_key("EOM"));
        assert_eq!(row["Ticker"], "AAPL");
    }

    #[test]
    fn strip_question_marks_removes_placeholder_values() {
        let mut value = json!({
            "type": "?",
            "venue": "?",
            "Ticker": "AAPL",
            "Price": 200.0
        });
        let row = value.as_object_mut().unwrap();
        strip_question_marks(row);

        assert!(!row.contains_key("type"));
        assert!(!row.contains_key("venue"));
        assert_eq!(row["Ticker"], "AAPL");
        assert_eq!(row["Price"], 200.0);
    }

    #[test]
    fn strip_question_marks_preserves_non_placeholder_strings() {
        let mut value = json!({
            "Ticker": "AAPL",
            "Sector": "Technology",
            "type": "closing"
        });
        let row = value.as_object_mut().unwrap();
        strip_question_marks(row);

        assert_eq!(row["Ticker"], "AAPL");
        assert_eq!(row["Sector"], "Technology");
        assert_eq!(row["type"], "closing");
    }

    #[test]
    fn transformed_trade_values_surfaces_serialization_errors() {
        #[derive(Debug)]
        struct FailingRecord;

        impl Serialize for FailingRecord {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom("serialize failed"))
            }
        }

        let err = transformed_trade_values(&[FailingRecord], TradeRecordKind::Trade).unwrap_err();

        assert!(err.to_string().contains("serialize failed"));
    }
}
