//! Static output field metadata for commands that support `--fields`.

use serde::Serialize;

use crate::cli::commands::report::REPORT_PRESETS;
use crate::cli::common::trade_record_kind::TradeRecordKind;

/// Machine-readable field discovery response for one command path.
#[derive(Debug, Serialize)]
pub(crate) struct FieldDiscovery {
    pub(crate) command_path: String,
    pub(crate) preferred_path: String,
    pub(crate) fields: &'static [FieldMetadata],
}

/// One output field accepted by `--fields` for a command.
#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct FieldMetadata {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    #[serde(rename = "type")]
    pub(crate) type_hint: FieldType,
}

/// Stable, coarse JSON type hints for output fields.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FieldType {
    String,
    Number,
    Boolean,
    Date,
    Datetime,
    Unknown,
}

macro_rules! field {
    ($name:expr, $description:expr, $type_hint:expr) => {
        FieldMetadata {
            name: $name,
            description: $description,
            type_hint: $type_hint,
        }
    };
}

/// Looks up output field metadata by canonical command path.
#[must_use]
pub(crate) fn discover(command_path: &[String]) -> Option<FieldDiscovery> {
    let path = command_path.join(" ");
    let (command_path, fields) = fields_for_path(path.as_str())?;

    Some(FieldDiscovery {
        command_path: command_path.to_string(),
        preferred_path: command_path.to_string(),
        fields,
    })
}

/// Returns exact output field names accepted by `--fields` for a command path.
#[must_use]
pub(crate) fn field_names(command_path: &str) -> Option<Vec<String>> {
    fields_for_path(command_path)
        .map(|(_, fields)| fields.iter().map(|field| field.name.to_string()).collect())
}

fn fields_for_path(path: &str) -> Option<(&str, &'static [FieldMetadata])> {
    if let Some(kind) = trade_record_kind_for_path(path) {
        return Some((path, fields_for_trade_record_kind(kind)));
    }

    if let Some(preset_name) = path.strip_prefix("report ")
        && REPORT_PRESETS
            .iter()
            .any(|preset| preset.use_name == preset_name)
    {
        return Some((path, fields_for_trade_record_kind(TradeRecordKind::Trade)));
    }

    FIELD_TABLES
        .iter()
        .find(|(command_path, _)| *command_path == path)
        .map(|(command_path, fields)| (*command_path, *fields))
}

fn trade_record_kind_for_path(path: &str) -> Option<TradeRecordKind> {
    match path {
        "trade list" => Some(TradeRecordKind::Trade),
        "trade levels" => Some(TradeRecordKind::Level),
        "trade clusters" => Some(TradeRecordKind::Cluster),
        "trade cluster-bombs" => Some(TradeRecordKind::ClusterBomb),
        _ => None,
    }
}

fn fields_for_trade_record_kind(kind: TradeRecordKind) -> &'static [FieldMetadata] {
    match kind {
        TradeRecordKind::Trade => TRADE_FIELDS,
        TradeRecordKind::Cluster => CLUSTER_FIELDS,
        TradeRecordKind::Level => LEVEL_FIELDS,
        TradeRecordKind::ClusterBomb => BOMB_FIELDS,
    }
}

mod tables;

pub(crate) use tables::{
    ALERT_HEADERS, BOMB_HEADERS, CLUSTER_HEADERS, LEVEL_HEADERS, TRADE_HEADERS, VOLUME_HEADERS,
};
use tables::{BOMB_FIELDS, CLUSTER_FIELDS, FIELD_TABLES, LEVEL_FIELDS, TRADE_FIELDS};

#[cfg(test)]
mod tests {
    use super::{discover, field_names, fields_for_path, fields_for_trade_record_kind};
    use crate::cli::commands::report::REPORT_PRESETS;
    use crate::cli::common::trade_record_kind::TradeRecordKind;

    #[test]
    fn discovers_required_issue_command_paths() {
        for path in [
            "trade list",
            "trade dashboard",
            "trade levels",
            "trade clusters",
            "trade cluster-bombs",
            "trade alerts",
            "report top-100-rank",
            "report top-10-rank",
            "report dark-pool-sweeps",
            "report disproportionately-large",
            "report leveraged-etfs",
            "report rsi-overbought",
            "report rsi-oversold",
            "report dark-pool-20x",
            "report top-30-rank-10x-99th",
            "report phantom-trades",
            "report offsetting-trades",
            "volume institutional",
            "volume total",
            "volume ah-institutional",
            "market earnings",
            "watchlist configs",
            "watchlist tickers",
            "alert configs",
        ] {
            let parts = path
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>();
            let discovery = discover(&parts).unwrap_or_else(|| panic!("missing {path}"));

            assert_eq!(discovery.preferred_path, path);
            assert!(!discovery.fields.is_empty(), "empty fields for {path}");
        }
    }

    #[test]
    fn report_fields_are_derived_from_report_presets() {
        for preset in REPORT_PRESETS {
            let path = format!("report {}", preset.use_name);
            let (preferred_path, fields) = fields_for_path(&path).expect("report fields exist");

            assert_eq!(preferred_path, path);
            assert!(std::ptr::eq(
                fields,
                fields_for_trade_record_kind(TradeRecordKind::Trade)
            ));
        }
    }

    #[test]
    fn rejects_commands_without_field_projection() {
        let path = ["doctor".to_string()];

        assert!(discover(&path).is_none());
    }

    #[test]
    fn returns_field_names_for_registered_commands() {
        let names = field_names("trade list").unwrap();

        assert!(names.iter().any(|name| name == "Ticker"));
        assert!(!names.iter().any(|name| name == "ticker"));
    }
}
