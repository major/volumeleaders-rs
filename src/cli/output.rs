use std::io::{self, Write};
use std::sync::{LazyLock, Mutex};

use serde::Serialize;
use serde_json::Value;

use crate::cli::common::trade_transforms::{TradeRecordKind, transformed_trade_values};
use crate::cli::error::{empty_result, json_error, usage_error};

static STRICT_EMPTY_CONTEXT: LazyLock<Mutex<Option<EmptyResultContext>>> =
    LazyLock::new(|| Mutex::new(None));

/// Describes how to explain an empty array for a specific command.
#[derive(Clone, Debug)]
pub struct EmptyResultContext {
    command: String,
    suggestion: &'static str,
}

impl EmptyResultContext {
    /// Creates command-aware empty-result guidance.
    #[must_use]
    pub fn new(command: impl Into<String>, suggestion: &'static str) -> Self {
        Self {
            command: command.into(),
            suggestion,
        }
    }

    fn for_command(command: impl Into<String>) -> Self {
        let command = command.into();
        let suggestion = empty_result_suggestion(&command);
        Self::new(command, suggestion)
    }

    fn message(&self) -> String {
        format!("{} returned no rows; {}", self.command, self.suggestion)
    }
}

/// Configures strict empty-result handling for record-array output.
pub fn configure_strict_empty(enabled: bool, command: Option<String>) {
    let context = enabled
        .then(|| EmptyResultContext::for_command(command.unwrap_or_else(|| "command".to_string())));
    set_strict_empty_context(context);
}

/// Infers the command path from raw CLI arguments.
pub fn strict_empty_command_from_args(args: impl IntoIterator<Item = String>) -> Option<String> {
    let mut words = args
        .into_iter()
        .filter(|arg| !arg.starts_with('-'))
        .take(2)
        .collect::<Vec<_>>();

    if words.is_empty() {
        return None;
    }

    if !is_group_command(&words[0]) || words.len() == 1 {
        words.truncate(1);
    }

    Some(words.join(" "))
}

/// Writes `value` as compact JSON to stdout, newline-terminated.
pub fn print_json<T: Serialize>(value: &T) -> io::Result<()> {
    write_json(&mut io::stdout().lock(), value)
}

/// Parses a comma-separated output field list.
///
/// Empty input and `all` both mean no filtering. Field names are case-sensitive.
/// Raw record output uses VolumeLeaders JSON keys; transformed output may expose
/// semantic keys such as `type`, `venue`, `events`, and `window` instead.
pub fn selected_fields(fields: Option<&str>) -> Option<Vec<String>> {
    let fields = fields?.trim();
    if fields.is_empty() || fields.eq_ignore_ascii_case("all") {
        return None;
    }

    let fields: Vec<String> = fields
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if fields.is_empty() {
        None
    } else {
        Some(fields)
    }
}

/// Serializes records to JSON values and retains only selected fields.
pub fn records_to_values<T: Serialize>(records: &[T], fields: Option<&[String]>) -> Vec<Value> {
    records
        .iter()
        .map(|record| {
            let mut value = serde_json::to_value(record).unwrap_or(Value::Null);
            if let Some(fields) = fields
                && let Some(map) = value.as_object_mut()
            {
                retain_selected_fields(map, fields);
            }
            value
        })
        .collect()
}

/// Outputs pre-serialized record values with compact JSON defaults and optional custom fields.
pub fn print_record_values(
    records: &[Value],
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> io::Result<()> {
    print_record_values_with_allowed_fields(records, compact_headers, fields, all_fields, None)
}

/// Outputs pre-serialized record values with command metadata-backed field validation.
pub(crate) fn print_record_values_with_allowed_fields(
    records: &[Value],
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    allowed_fields: Option<&[String]>,
) -> io::Result<()> {
    strict_empty_error_if_needed(records.is_empty())?;
    write_record_values(
        io::stdout().lock(),
        records,
        compact_headers,
        fields,
        all_fields,
        allowed_fields,
    )
}

/// Transforms trade-shaped records and outputs them with field filtering.
pub fn print_transformed_record_values<T: Serialize>(
    records: &[T],
    kind: TradeRecordKind,
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> io::Result<()> {
    print_transformed_record_values_with_allowed_fields(
        records,
        kind,
        compact_headers,
        fields,
        all_fields,
        None,
    )
}

/// Transforms trade-shaped records and validates custom fields against command metadata.
pub(crate) fn print_transformed_record_values_with_allowed_fields<T: Serialize>(
    records: &[T],
    kind: TradeRecordKind,
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    allowed_fields: Option<&[String]>,
) -> io::Result<()> {
    strict_empty_error_if_needed(records.is_empty())?;
    transformed_trade_values(records, kind)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        .and_then(|values| {
            print_record_values_with_allowed_fields(
                &values,
                compact_headers,
                fields,
                all_fields,
                allowed_fields,
            )
        })
}

/// Writes pre-serialized record values to `writer`.
pub(crate) fn write_record_values<W: Write>(
    mut writer: W,
    records: &[Value],
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    allowed_fields: Option<&[String]>,
) -> io::Result<()> {
    strict_empty_error_if_needed(records.is_empty())?;
    let custom_fields = selected_fields(fields);
    let raw_fields_requested =
        fields.is_some_and(|fields| fields.trim().eq_ignore_ascii_case("all"));

    if let Some(fields) = custom_fields.as_deref() {
        validate_value_fields(records, fields, allowed_fields)?;
    }

    if all_fields || raw_fields_requested {
        return write_json(&mut writer, &records);
    }

    let default_fields: Vec<String> = compact_headers
        .iter()
        .map(|field| (*field).to_string())
        .collect();
    let selected = custom_fields
        .as_deref()
        .unwrap_or(default_fields.as_slice());
    let values = filter_record_values(records, selected);
    write_json(&mut writer, &values)
}

/// Checks custom output fields against record keys when records are available.
pub fn validate_record_fields<T: Serialize>(records: &[T], fields: &[String]) -> io::Result<()> {
    validate_selected_fields(available_record_fields(records)?, fields)
}

/// Outputs record lists with compact JSON defaults and optional custom fields.
pub fn print_records<T: Serialize>(
    records: &[T],
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> io::Result<()> {
    print_records_with_allowed_fields(records, compact_headers, fields, all_fields, None)
}

/// Outputs record lists with command metadata-backed field validation.
pub(crate) fn print_records_with_allowed_fields<T: Serialize>(
    records: &[T],
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    allowed_fields: Option<&[String]>,
) -> io::Result<()> {
    strict_empty_error_if_needed(records.is_empty())?;
    let custom_fields = selected_fields(fields);
    let raw_fields_requested =
        fields.is_some_and(|fields| fields.trim().eq_ignore_ascii_case("all"));

    if let Some(fields) = custom_fields.as_deref() {
        if let Some(allowed_fields) = allowed_fields {
            validate_selected_fields(allowed_fields.to_vec(), fields)?;
        } else {
            validate_record_fields(records, fields)?;
        }
    }

    if all_fields || raw_fields_requested {
        return print_json(&records);
    }

    let default_fields: Vec<String> = compact_headers
        .iter()
        .map(|field| (*field).to_string())
        .collect();
    let selected = custom_fields
        .as_deref()
        .unwrap_or(default_fields.as_slice());
    let values = records_to_values(records, Some(selected));
    print_json(&values)
}

fn available_record_fields<T: Serialize>(records: &[T]) -> io::Result<Vec<String>> {
    let mut fields = Vec::new();
    for record in records {
        let value = serde_json::to_value(record)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        collect_unique_fields(value.as_object(), &mut fields);
    }
    fields.sort();
    Ok(fields)
}

fn filter_record_values(records: &[Value], fields: &[String]) -> Vec<Value> {
    records
        .iter()
        .map(|record| {
            let mut value = record.clone();
            if let Some(map) = value.as_object_mut() {
                retain_selected_fields(map, fields);
            }
            value
        })
        .collect()
}

fn validate_value_fields(
    records: &[Value],
    fields: &[String],
    allowed_fields: Option<&[String]>,
) -> io::Result<()> {
    let available = allowed_fields
        .map(<[String]>::to_vec)
        .unwrap_or_else(|| available_value_fields(records));
    validate_selected_fields(available, fields)
}

fn validate_selected_fields(available: Vec<String>, fields: &[String]) -> io::Result<()> {
    if available.is_empty() {
        return Ok(());
    }

    let missing = missing_fields(&available, fields);

    if missing.is_empty() {
        return Ok(());
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "unknown output field(s): {}. Available fields: {}",
            missing.join(", "),
            available.join(", ")
        ),
    ))
}

fn missing_fields<'a>(available: &[String], requested: &'a [String]) -> Vec<&'a str> {
    requested
        .iter()
        .map(String::as_str)
        .filter(|field| !available.iter().any(|available| available == field))
        .collect()
}

fn available_value_fields(records: &[Value]) -> Vec<String> {
    let mut fields = Vec::new();
    for record in records {
        collect_unique_fields(record.as_object(), &mut fields);
    }
    fields.sort();
    fields
}

fn collect_unique_fields(map: Option<&serde_json::Map<String, Value>>, fields: &mut Vec<String>) {
    if let Some(map) = map {
        for key in map.keys() {
            if !fields.iter().any(|field| field == key) {
                fields.push(key.clone());
            }
        }
    }
}

fn retain_selected_fields(map: &mut serde_json::Map<String, Value>, fields: &[String]) {
    map.retain(|key, _| fields.iter().any(|field| field == key));
}

/// Prints `value` as compact JSON.
pub fn print_result<T: Serialize>(value: &T) -> io::Result<()> {
    print_json(value)
}

/// Convert an output write result into the CLI exit code convention.
pub fn finish_output(result: io::Result<()>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(err) if err.kind() == io::ErrorKind::NotFound => empty_result(err.to_string()),
        Err(err) if err.kind() == io::ErrorKind::InvalidInput => {
            usage_error(format!("output error: {err}"))
        }
        Err(err) => json_error(format!("output error: {err}")),
    }
}

fn strict_empty_error_if_needed(records_empty: bool) -> io::Result<()> {
    if !records_empty {
        return Ok(());
    }

    let context = strict_empty_context();
    if let Some(context) = context {
        return Err(io::Error::new(io::ErrorKind::NotFound, context.message()));
    }

    Ok(())
}

fn set_strict_empty_context(context: Option<EmptyResultContext>) {
    *STRICT_EMPTY_CONTEXT
        .lock()
        .expect("strict empty context lock poisoned") = context;
}

fn strict_empty_context() -> Option<EmptyResultContext> {
    STRICT_EMPTY_CONTEXT
        .lock()
        .expect("strict empty context lock poisoned")
        .clone()
}

fn is_group_command(command: &str) -> bool {
    matches!(
        command,
        "alert" | "market" | "report" | "trade" | "volume" | "watchlist"
    )
}

fn empty_result_suggestion(command: &str) -> &'static str {
    if command.starts_with("trade ") || matches!(command, "trades" | "dashboard" | "levels") {
        "try checking the ticker or widening the date range"
    } else if command.starts_with("report ") {
        "try a broader report, longer lookback, or fewer filters"
    } else if command.starts_with("volume ") || command == "market earnings" {
        "try a different date range or fewer ticker filters"
    } else if command == "alert configs" {
        "no alert configurations may be valid account state"
    } else if command.starts_with("watchlist ") {
        "no watchlist rows may be valid account state"
    } else {
        "try widening filters or removing optional constraints"
    }
}

/// Writes `value` as compact JSON to `writer`, newline-terminated.
fn write_json<W: Write, T: Serialize>(writer: &mut W, value: &T) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    writer.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use crate::cli::common::trade_transforms::TradeRecordKind;
    use crate::cli::error::{EXIT_EMPTY_RESULT, EXIT_JSON_ERROR, EXIT_USAGE_ERROR};

    use super::{
        configure_strict_empty, empty_result_suggestion, finish_output, print_record_values,
        print_record_values_with_allowed_fields, print_records, print_records_with_allowed_fields,
        print_transformed_record_values, print_transformed_record_values_with_allowed_fields,
        records_to_values, selected_fields, set_strict_empty_context,
        strict_empty_command_from_args, write_json, write_record_values,
    };

    #[derive(Debug, Serialize)]
    struct TestRecord {
        symbol: String,
        price: f64,
        volume: u64,
    }

    fn sample_records() -> Vec<TestRecord> {
        vec![
            TestRecord {
                symbol: "AAPL".to_string(),
                price: 150.5,
                volume: 1_000_000,
            },
            TestRecord {
                symbol: "MSFT".to_string(),
                price: 320.75,
                volume: 500_000,
            },
        ]
    }

    #[test]
    fn output_compact_json() {
        let record = &sample_records()[0];
        let mut buf = Vec::new();
        write_json(&mut buf, record).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Compact JSON is a single line plus trailing newline.
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 1, "compact JSON should be a single line");
        assert!(output.ends_with('\n'));

        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed["symbol"], "AAPL");
        assert_eq!(parsed["price"], 150.5);
        assert_eq!(parsed["volume"], 1_000_000);
    }

    #[test]
    fn selected_fields_trims_all_sentinel() {
        assert_eq!(selected_fields(Some(" all ")), None);
        assert_eq!(
            selected_fields(Some(" symbol, price ")),
            Some(vec!["symbol".to_string(), "price".to_string()])
        );
    }

    #[test]
    fn selected_fields_returns_none_for_empty_input() {
        assert_eq!(selected_fields(None), None);
        assert_eq!(selected_fields(Some("")), None);
        assert_eq!(selected_fields(Some(",,,")), None);
    }

    #[test]
    fn records_to_values_filters_to_selected_fields() {
        let records = sample_records();
        let values = records_to_values(&records, Some(&["symbol".to_string()]));

        assert_eq!(values[0]["symbol"], "AAPL");
        assert!(values[0].get("price").is_none());
    }

    #[test]
    fn records_to_values_without_fields_preserves_all_fields() {
        let records = sample_records();
        let values = records_to_values(&records, None);

        assert_eq!(values[0]["symbol"], "AAPL");
        assert_eq!(values[0]["price"], 150.5);
        assert_eq!(values[0]["volume"], 1_000_000);
    }

    #[test]
    fn print_records_rejects_unknown_custom_fields() {
        set_strict_empty_context(None);
        let records = sample_records();
        let err = print_records(&records, &["symbol"], Some("ticker"), false).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("unknown output field"));
        assert!(err.to_string().contains("symbol"));
    }

    #[test]
    fn write_record_values_outputs_custom_field() {
        set_strict_empty_context(None);
        let records = sample_records();
        let values: Vec<serde_json::Value> = records
            .iter()
            .map(|r| serde_json::to_value(r).unwrap())
            .collect();
        let mut buf = Vec::new();

        write_record_values(
            &mut buf,
            &values,
            &["symbol", "price"],
            Some("symbol"),
            false,
            None,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(
            parsed,
            serde_json::json!([
                {"symbol": "AAPL"},
                {"symbol": "MSFT"}
            ])
        );
    }

    #[test]
    fn write_record_values_rejects_unknown_custom_fields() {
        set_strict_empty_context(None);
        let records = sample_records();
        let values: Vec<serde_json::Value> = records
            .iter()
            .map(|r| serde_json::to_value(r).unwrap())
            .collect();
        let mut buf = Vec::new();

        let err = write_record_values(
            &mut buf,
            &values,
            &["symbol", "price"],
            Some("ticker"),
            false,
            None,
        )
        .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("unknown output field"));
        assert!(err.to_string().contains("ticker"));
    }

    #[test]
    fn write_record_values_accepts_metadata_field_absent_from_rows() {
        set_strict_empty_context(None);
        let values = vec![serde_json::json!({"symbol": "AAPL"})];
        let allowed_fields = vec!["symbol".to_string(), "events".to_string()];
        let mut buf = Vec::new();

        write_record_values(
            &mut buf,
            &values,
            &["symbol"],
            Some("events"),
            false,
            Some(&allowed_fields),
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed, serde_json::json!([{}]));
    }

    #[test]
    fn write_record_values_rejects_field_missing_from_metadata() {
        set_strict_empty_context(None);
        let values = vec![serde_json::json!({"symbol": "AAPL", "price": 150.5})];
        let allowed_fields = vec!["symbol".to_string()];
        let mut buf = Vec::new();

        let err = write_record_values(
            &mut buf,
            &values,
            &["symbol"],
            Some("price"),
            false,
            Some(&allowed_fields),
        )
        .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("price"));
    }

    #[test]
    fn print_record_values_with_allowed_fields_accepts_metadata_fields() {
        set_strict_empty_context(None);
        let values = vec![serde_json::json!({"symbol": "AAPL"})];
        let allowed_fields = vec!["symbol".to_string(), "events".to_string()];

        print_record_values_with_allowed_fields(
            &values,
            &["symbol"],
            Some("events"),
            false,
            Some(&allowed_fields),
        )
        .unwrap();
    }

    #[test]
    fn print_records_with_allowed_fields_accepts_metadata_fields() {
        set_strict_empty_context(None);
        let records = sample_records();
        let allowed_fields = vec!["symbol".to_string(), "events".to_string()];

        print_records_with_allowed_fields(
            &records,
            &["symbol"],
            Some("events"),
            false,
            Some(&allowed_fields),
        )
        .unwrap();
    }

    #[test]
    fn print_transformed_record_values_with_allowed_fields_accepts_metadata_fields() {
        set_strict_empty_context(None);
        let records = vec![serde_json::json!({
            "Ticker": "AAPL",
            "Date": "/Date(1767312000000)/",
            "Price": 150.5,
            "Dollars": 1_000_000.0
        })];
        let allowed_fields = vec!["Ticker".to_string(), "events".to_string()];

        print_transformed_record_values_with_allowed_fields(
            &records,
            TradeRecordKind::Trade,
            &["Ticker"],
            Some("events"),
            false,
            Some(&allowed_fields),
        )
        .unwrap();
    }

    #[test]
    fn write_record_values_outputs_raw_fields_for_all_sentinel() {
        set_strict_empty_context(None);
        let records = sample_records();
        let values: Vec<serde_json::Value> = records
            .iter()
            .map(|r| serde_json::to_value(r).unwrap())
            .collect();
        let mut buf = Vec::new();

        write_record_values(&mut buf, &values, &["symbol"], Some("all"), false, None).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed, serde_json::Value::Array(values));
    }

    #[test]
    fn finish_output_maps_result_to_exit_code() {
        assert_eq!(finish_output(Ok(())), 0);
        assert_eq!(
            finish_output(Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "trade list returned no rows; try widening filters"
            ))),
            EXIT_EMPTY_RESULT
        );
        assert_eq!(
            finish_output(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "unknown output field"
            ))),
            EXIT_USAGE_ERROR
        );
        assert_eq!(
            finish_output(Err(std::io::Error::other("broken pipe"))),
            EXIT_JSON_ERROR
        );
    }

    #[test]
    fn print_records_maps_strict_empty_to_sentinel_error() {
        let records: Vec<TestRecord> = Vec::new();
        configure_strict_empty(true, Some("trade list".to_string()));

        let err = print_records(&records, &["symbol"], None, false).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        assert_eq!(
            err.to_string(),
            "trade list returned no rows; try checking the ticker or widening the date range"
        );
    }

    #[test]
    fn print_records_allows_empty_without_strict_flag() {
        let records: Vec<TestRecord> = Vec::new();
        configure_strict_empty(false, Some("trade list".to_string()));

        assert!(print_records(&records, &["symbol"], None, false).is_ok());
    }

    #[test]
    fn print_records_writes_non_empty_records_with_strict_flag() {
        let records = sample_records();
        configure_strict_empty(true, Some("trade list".to_string()));

        assert!(print_records(&records, &["symbol"], None, false).is_ok());
    }

    #[test]
    fn write_record_values_maps_strict_empty_to_sentinel_error() {
        let records = Vec::new();
        let mut buf = Vec::new();
        configure_strict_empty(true, Some("watchlist configs".to_string()));

        let err = write_record_values(&mut buf, &records, &["Name"], Some("all"), false, None)
            .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        assert!(err.to_string().contains("valid account state"));
    }

    #[test]
    fn print_record_values_maps_strict_empty_to_sentinel_error() {
        configure_strict_empty(true, Some("alert configs".to_string()));

        let err = print_record_values(&[], &["Name"], None, false).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        assert!(err.to_string().contains("valid account state"));
    }

    #[test]
    fn print_transformed_record_values_maps_strict_empty_to_sentinel_error() {
        let records: Vec<TestRecord> = Vec::new();
        configure_strict_empty(true, Some("report dark-pool-sweeps".to_string()));

        let err = print_transformed_record_values(
            &records,
            TradeRecordKind::Trade,
            &["Ticker"],
            None,
            false,
        )
        .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        assert!(err.to_string().contains("broader report"));
    }

    #[test]
    fn strict_empty_command_from_args_uses_leaf_path_when_available() {
        let command = strict_empty_command_from_args([
            "--strict-empty".to_string(),
            "trade".to_string(),
            "list".to_string(),
            "NVDA".to_string(),
        ]);

        assert_eq!(command.as_deref(), Some("trade list"));
    }

    #[test]
    fn strict_empty_command_from_args_uses_single_local_command() {
        let command = strict_empty_command_from_args([
            "commands".to_string(),
            "--strict-empty".to_string(),
            "--grouped".to_string(),
        ]);

        assert_eq!(command.as_deref(), Some("commands"));
    }

    #[test]
    fn strict_empty_command_from_args_returns_none_without_command_words() {
        let command = strict_empty_command_from_args(["--strict-empty".to_string()]);

        assert_eq!(command, None);
    }

    #[test]
    fn empty_result_suggestions_are_command_aware() {
        assert!(empty_result_suggestion("report top-100-rank").contains("broader report"));
        assert!(empty_result_suggestion("trades").contains("ticker"));
        assert!(empty_result_suggestion("dashboard").contains("ticker"));
        assert!(empty_result_suggestion("levels").contains("ticker"));
        assert!(empty_result_suggestion("volume institutional").contains("different date"));
        assert!(empty_result_suggestion("market earnings").contains("different date"));
        assert!(empty_result_suggestion("alert configs").contains("valid account state"));
        assert!(empty_result_suggestion("watchlist tickers").contains("valid account state"));
        assert!(empty_result_suggestion("command").contains("widening filters"));
    }
}
