use std::io::{self, Write};
use std::sync::{LazyLock, Mutex};

use serde::Serialize;
use serde_json::Value;

use crate::cli::error::CliExit;

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
/// Record output uses VolumeLeaders JSON keys.
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

/// Writes pre-serialized record values to `writer`.
pub(crate) fn write_record_values<W: Write>(
    mut writer: W,
    records: &[Value],
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    allowed_fields: Option<&[String]>,
) -> io::Result<()> {
    let custom_fields = selected_fields(fields);
    let raw_fields_requested =
        fields.is_some_and(|fields| fields.trim().eq_ignore_ascii_case("all"));

    if let (Some(fields), Some(allowed_fields)) = (custom_fields.as_deref(), allowed_fields) {
        validate_selected_fields(allowed_fields.to_vec(), fields)?;
    }

    strict_empty_error_if_needed(records.is_empty())?;

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
    let custom_fields = selected_fields(fields);
    let raw_fields_requested =
        fields.is_some_and(|fields| fields.trim().eq_ignore_ascii_case("all"));

    if let (Some(fields), Some(allowed_fields)) = (custom_fields.as_deref(), allowed_fields) {
        validate_selected_fields(allowed_fields.to_vec(), fields)?;
    }

    strict_empty_error_if_needed(records.is_empty())?;

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
    let mut selected = serde_json::Map::new();
    for field in fields {
        if let Some(value) = map.remove(field) {
            selected.insert(field.clone(), value);
        }
    }
    *map = selected;
}

/// Prints `value` as compact JSON.
pub fn print_result<T: Serialize>(value: &T) -> io::Result<()> {
    print_json(value)
}

/// Convert an output write result into the `Result<(), CliExit>` convention.
pub fn finish_output(result: io::Result<()>) -> Result<(), CliExit> {
    Ok(result?)
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
#[path = "output_tests.rs"]
mod tests;
