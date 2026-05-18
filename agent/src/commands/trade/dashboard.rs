use serde::Serialize;
use serde_json::{Map, Value};

use crate::common::trade_transforms::transform_trade_dashboard;

use super::{DashboardArgs, DateRange};

const DASHBOARD_TOP_LEVEL_FIELDS: [&str; 3] = ["ticker", "date_range", "count"];
const DASHBOARD_COMPACT_TRADE_FIELDS: [&str; 11] = [
    "Date",
    "FullTimeString24",
    "Price",
    "Dollars",
    "Volume",
    "TradeRank",
    "TradeCount",
    "type",
    "venue",
    "events",
    "TradeConditions",
];
const DASHBOARD_COMPACT_CLUSTER_FIELDS: [&str; 8] = [
    "Date",
    "Price",
    "Dollars",
    "Volume",
    "TradeCount",
    "TradeClusterRank",
    "window",
    "events",
];
const DASHBOARD_COMPACT_LEVEL_FIELDS: [&str; 6] = [
    "Price",
    "Dollars",
    "Volume",
    "Trades",
    "RelativeSize",
    "TradeLevelRank",
];
const DASHBOARD_COMPACT_BOMB_FIELDS: [&str; 7] = [
    "Date",
    "Dollars",
    "Volume",
    "TradeCount",
    "TradeClusterBombRank",
    "window",
    "events",
];

#[derive(Debug, Serialize)]
pub(super) struct TradeDashboard {
    pub(super) ticker: String,
    pub(super) date_range: DateRange,
    pub(super) count: usize,
    pub(super) trades: Vec<volumeleaders_client::Trade>,
    pub(super) clusters: Vec<volumeleaders_client::TradeCluster>,
    pub(super) levels: Vec<volumeleaders_client::TradeLevel>,
    pub(super) cluster_bombs: Vec<volumeleaders_client::TradeClusterBomb>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct DashboardFieldSelection {
    pub(super) unqualified: Vec<String>,
    pub(super) trades: Vec<String>,
    pub(super) clusters: Vec<String>,
    pub(super) levels: Vec<String>,
    pub(super) cluster_bombs: Vec<String>,
}

pub(super) fn dashboard_output_value(
    dashboard: &TradeDashboard,
    args: &DashboardArgs,
) -> Result<Value, String> {
    let mut value = serde_json::to_value(dashboard).unwrap_or(Value::Null);
    let Some(map) = value.as_object_mut() else {
        return Ok(value);
    };

    transform_trade_dashboard(map);

    match args.fields.as_deref().map(str::trim) {
        _ if args.all_fields => Ok(value),
        Some(fields) if fields.eq_ignore_ascii_case("all") => Ok(value),
        Some(fields) if !fields.is_empty() => {
            let selection = parse_dashboard_fields(fields)?;
            apply_selected_dashboard_fields(map, &selection)?;
            Ok(value)
        }
        _ => {
            apply_compact_dashboard_fields(map);
            Ok(value)
        }
    }
}

pub(super) fn parse_dashboard_fields(fields: &str) -> Result<DashboardFieldSelection, String> {
    let mut selection = DashboardFieldSelection::default();
    for field in fields
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
    {
        if let Some((section, name)) = field.split_once('.') {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            match section.trim().to_ascii_lowercase().as_str() {
                "trades" | "trade" => selection.trades.push(name.to_string()),
                "clusters" | "cluster" => selection.clusters.push(name.to_string()),
                "levels" | "level" => selection.levels.push(name.to_string()),
                "cluster_bombs" | "cluster-bombs" | "bombs" | "bomb" => {
                    selection.cluster_bombs.push(name.to_string());
                }
                _ => {
                    return Err(format!(
                        "unknown dashboard field section `{}` in `{}`; use trades, clusters, levels, or cluster_bombs",
                        section.trim(),
                        field
                    ));
                }
            }
        } else {
            selection.unqualified.push(field.to_string());
        }
    }
    Ok(selection)
}

fn apply_compact_dashboard_fields(map: &mut Map<String, Value>) {
    retain_dashboard_top_level(map);
    filter_dashboard_section(map, "trades", &DASHBOARD_COMPACT_TRADE_FIELDS, true);
    filter_dashboard_section(map, "clusters", &DASHBOARD_COMPACT_CLUSTER_FIELDS, true);
    filter_dashboard_section(map, "levels", &DASHBOARD_COMPACT_LEVEL_FIELDS, true);
    filter_dashboard_section(map, "cluster_bombs", &DASHBOARD_COMPACT_BOMB_FIELDS, true);
}

fn apply_selected_dashboard_fields(
    map: &mut Map<String, Value>,
    selection: &DashboardFieldSelection,
) -> Result<(), String> {
    retain_dashboard_top_level(map);
    apply_selected_dashboard_section(map, "trades", &selection.trades, selection)?;
    apply_selected_dashboard_section(map, "clusters", &selection.clusters, selection)?;
    apply_selected_dashboard_section(map, "levels", &selection.levels, selection)?;
    apply_selected_dashboard_section(map, "cluster_bombs", &selection.cluster_bombs, selection)?;
    Ok(())
}

fn apply_selected_dashboard_section(
    map: &mut Map<String, Value>,
    section: &str,
    section_fields: &[String],
    selection: &DashboardFieldSelection,
) -> Result<(), String> {
    let fields = dashboard_section_fields(section_fields, &selection.unqualified);
    if fields.is_empty() {
        map.remove(section);
        return Ok(());
    }
    let matched = filter_dashboard_section(map, section, &fields, false);
    if matched == 0 && section_has_rows(map, section) {
        return Err(format!(
            "no requested dashboard fields matched `{section}` rows; field names are case-sensitive"
        ));
    }
    Ok(())
}

fn dashboard_section_fields(section_fields: &[String], unqualified: &[String]) -> Vec<String> {
    section_fields
        .iter()
        .chain(unqualified)
        .filter(|field| !field.trim().is_empty())
        .cloned()
        .collect()
}

fn retain_dashboard_top_level(map: &mut Map<String, Value>) {
    map.retain(|key, _| {
        DASHBOARD_TOP_LEVEL_FIELDS.contains(&key.as_str()) || is_dashboard_section(key)
    });
}

fn is_dashboard_section(key: &str) -> bool {
    matches!(key, "trades" | "clusters" | "levels" | "cluster_bombs")
}

fn filter_dashboard_section<F>(
    map: &mut Map<String, Value>,
    section: &str,
    fields: &[F],
    omit_empty: bool,
) -> usize
where
    F: AsRef<str>,
{
    let Some(Value::Array(rows)) = map.get_mut(section) else {
        return 0;
    };
    let mut matched = 0;
    for row in rows {
        let Some(row_map) = row.as_object_mut() else {
            continue;
        };
        row_map.retain(|key, value| {
            let selected = fields.iter().any(|field| field.as_ref() == key);
            if selected {
                matched += 1;
            }
            selected && (!omit_empty || !is_empty_dashboard_value(value))
        });
    }
    matched
}

fn section_has_rows(map: &Map<String, Value>, section: &str) -> bool {
    matches!(map.get(section), Some(Value::Array(rows)) if !rows.is_empty())
}

fn is_empty_dashboard_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(false) => true,
        Value::Number(_) => false,
        Value::String(value) => value.is_empty(),
        Value::Array(values) => values.is_empty(),
        Value::Object(values) => values.is_empty(),
        Value::Bool(true) => false,
    }
}
