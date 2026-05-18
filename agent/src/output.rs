use std::io::{self, Write};

use serde::Serialize;
use serde_json::Value;

/// Writes `value` as JSON to stdout, newline-terminated.
///
/// Uses compact format when `pretty` is false, 2-space-indented when true.
pub fn print_json<T: Serialize>(value: &T, pretty: bool) -> io::Result<()> {
    write_json(&mut io::stdout().lock(), value, pretty)
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
    pretty: bool,
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> io::Result<()> {
    write_record_values(
        io::stdout().lock(),
        records,
        pretty,
        compact_headers,
        fields,
        all_fields,
    )
}

/// Writes pre-serialized record values to `writer`.
pub(crate) fn write_record_values<W: Write>(
    mut writer: W,
    records: &[Value],
    pretty: bool,
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> io::Result<()> {
    let custom_fields = selected_fields(fields);
    let raw_fields_requested =
        fields.is_some_and(|fields| fields.trim().eq_ignore_ascii_case("all"));

    if let Some(fields) = custom_fields.as_deref() {
        validate_value_fields(records, fields)?;
    }

    if all_fields || raw_fields_requested {
        return write_json(&mut writer, &records, pretty);
    }

    let default_fields: Vec<String> = compact_headers
        .iter()
        .map(|field| (*field).to_string())
        .collect();
    let selected = custom_fields
        .as_deref()
        .unwrap_or(default_fields.as_slice());
    let values = filter_record_values(records, selected);
    write_json(&mut writer, &values, pretty)
}

/// Checks custom output fields against record keys when records are available.
pub fn validate_record_fields<T: Serialize>(records: &[T], fields: &[String]) -> io::Result<()> {
    validate_selected_fields(available_record_fields(records)?, fields)
}

/// Outputs record lists with compact JSON defaults and optional custom fields.
pub fn print_records<T: Serialize>(
    records: &[T],
    pretty: bool,
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> io::Result<()> {
    let custom_fields = selected_fields(fields);
    let raw_fields_requested =
        fields.is_some_and(|fields| fields.trim().eq_ignore_ascii_case("all"));

    if let Some(fields) = custom_fields.as_deref() {
        validate_record_fields(records, fields)?;
    }

    if all_fields || raw_fields_requested {
        return print_json(&records, pretty);
    }

    let default_fields: Vec<String> = compact_headers
        .iter()
        .map(|field| (*field).to_string())
        .collect();
    let selected = custom_fields
        .as_deref()
        .unwrap_or(default_fields.as_slice());
    let values = records_to_values(records, Some(selected));
    print_json(&values, pretty)
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

fn validate_value_fields(records: &[Value], fields: &[String]) -> io::Result<()> {
    validate_selected_fields(available_value_fields(records), fields)
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

/// Prints `value` as JSON.
pub fn print_result<T: Serialize>(value: &T, pretty: bool) -> io::Result<()> {
    print_json(value, pretty)
}

/// Convert an output write result into the CLI exit code convention.
pub fn finish_output(result: io::Result<()>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("output error: {err}");
            1
        }
    }
}

/// Writes `value` as JSON to `writer`, newline-terminated.
fn write_json<W: Write, T: Serialize>(writer: &mut W, value: &T, pretty: bool) -> io::Result<()> {
    if pretty {
        serde_json::to_writer_pretty(&mut *writer, value)
    } else {
        serde_json::to_writer(&mut *writer, value)
    }
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    writer.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{print_records, records_to_values, selected_fields, write_json};

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
        write_json(&mut buf, record, false).unwrap();
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
    fn output_pretty_json() {
        let record = &sample_records()[0];
        let mut buf = Vec::new();
        write_json(&mut buf, record, true).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Pretty JSON is multi-line with 2-space indentation.
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() > 1, "pretty JSON should be multi-line");
        assert!(
            output.contains("  \"symbol\""),
            "should use 2-space indentation"
        );
        assert!(output.ends_with('\n'));

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["symbol"], "AAPL");
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
        let records = sample_records();
        let err = print_records(&records, false, &["symbol"], Some("ticker"), false).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("unknown output field"));
        assert!(err.to_string().contains("symbol"));
    }
}
