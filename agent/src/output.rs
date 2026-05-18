use std::io::{self, Write};

use serde::Serialize;
use serde_json::Value;

use crate::common::trade_transforms::{TradeRecordKind, transformed_trade_values};

/// Controls the output format for CLI results.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum OutputFormat {
    /// Tab-separated values (default).
    #[default]
    Tsv,
    /// Compact JSON.
    Json,
    /// Pretty-printed JSON.
    JsonPretty,
}

/// Writes `value` to stdout in the requested output format.
///
/// JSON output is newline-terminated. TSV output prints scalar values as a
/// single line, objects as a header row plus one value row, and arrays as rows.
pub fn print_value<T: Serialize>(value: &T, format: &OutputFormat) -> io::Result<()> {
    let mut writer = io::stdout().lock();
    match format {
        OutputFormat::Tsv => {
            let value = serde_json::to_value(value)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            write_tsv_single(&mut writer, &value)
        }
        OutputFormat::Json => write_json(&mut writer, value, false),
        OutputFormat::JsonPretty => write_json(&mut writer, value, true),
    }
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

/// Outputs pre-serialized record values with compact defaults and optional custom fields.
pub fn print_record_values(
    records: &[Value],
    format: &OutputFormat,
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> io::Result<()> {
    let mut writer = io::stdout().lock();
    write_record_values(
        &mut writer,
        records,
        format,
        compact_headers,
        fields,
        all_fields,
    )
}

/// Transforms trade-shaped records and outputs them with field filtering.
pub fn print_transformed_record_values<T: Serialize>(
    records: &[T],
    kind: TradeRecordKind,
    format: &OutputFormat,
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> io::Result<()> {
    transformed_trade_values(records, kind)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        .and_then(|values| {
            print_record_values(&values, format, compact_headers, fields, all_fields)
        })
}

/// Writes pre-serialized record values to `writer`.
pub(crate) fn write_record_values<W: Write>(
    writer: &mut W,
    records: &[Value],
    format: &OutputFormat,
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
        return match format {
            OutputFormat::Tsv => {
                let headers = record_headers(records);
                write_tsv(writer, records, &headers)
            }
            OutputFormat::Json => write_json(writer, &records, false),
            OutputFormat::JsonPretty => write_json(writer, &records, true),
        };
    }

    let default_fields: Vec<String> = compact_headers
        .iter()
        .map(|field| (*field).to_string())
        .collect();
    let selected = custom_fields
        .as_deref()
        .unwrap_or(default_fields.as_slice());
    let values = filter_record_values(records, selected);
    match format {
        OutputFormat::Tsv => {
            let headers: Vec<&str> = selected.iter().map(String::as_str).collect();
            write_tsv(writer, &values, &headers)
        }
        OutputFormat::Json => write_json(writer, &values, false),
        OutputFormat::JsonPretty => write_json(writer, &values, true),
    }
}

/// Checks custom output fields against record keys when records are available.
pub fn validate_record_fields<T: Serialize>(records: &[T], fields: &[String]) -> io::Result<()> {
    validate_selected_fields(available_record_fields(records)?, fields)
}

/// Outputs record lists with compact defaults and optional custom fields.
pub fn print_records<T: Serialize>(
    records: &[T],
    format: &OutputFormat,
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
) -> io::Result<()> {
    let values = records
        .iter()
        .map(|record| {
            serde_json::to_value(record).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        })
        .collect::<io::Result<Vec<_>>>()?;
    let mut writer = io::stdout().lock();
    write_record_values(
        &mut writer,
        &values,
        format,
        compact_headers,
        fields,
        all_fields,
    )
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

/// Convenience alias for [`print_value`] for callers that prefer the name.
pub fn print_output<T: Serialize>(value: &T, format: &OutputFormat) -> io::Result<()> {
    print_value(value, format)
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

/// Escapes a TSV field value so tabs and newlines do not break row structure.
fn escape_tsv_field(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn write_tsv<W: Write>(writer: &mut W, records: &[Value], headers: &[&str]) -> io::Result<()> {
    writeln!(writer, "{}", headers.join("\t"))?;
    for record in records {
        let row: Vec<String> = headers
            .iter()
            .map(|header| match record.get(header) {
                Some(Value::String(s)) => escape_tsv_field(s),
                Some(Value::Null) | None => String::new(),
                Some(v) => escape_tsv_field(&v.to_string()),
            })
            .collect();
        writeln!(writer, "{}", row.join("\t"))?;
    }
    Ok(())
}

fn write_tsv_single<W: Write>(writer: &mut W, value: &Value) -> io::Result<()> {
    match value {
        Value::Object(map) => {
            let keys: Vec<&str> = map.keys().map(String::as_str).collect();
            writeln!(writer, "{}", keys.join("\t"))?;
            let vals: Vec<String> = map
                .values()
                .map(|v| match v {
                    Value::String(s) => escape_tsv_field(s),
                    Value::Null => String::new(),
                    v => escape_tsv_field(&v.to_string()),
                })
                .collect();
            writeln!(writer, "{}", vals.join("\t"))?;
        }
        Value::Array(arr) => {
            if let Some(Value::Object(first)) = arr.first() {
                let keys: Vec<&str> = first.keys().map(String::as_str).collect();
                writeln!(writer, "{}", keys.join("\t"))?;
                for item in arr {
                    let vals: Vec<String> = keys
                        .iter()
                        .map(|key| match item.get(*key) {
                            Some(Value::String(s)) => escape_tsv_field(s),
                            Some(Value::Null) | None => String::new(),
                            Some(v) => escape_tsv_field(&v.to_string()),
                        })
                        .collect();
                    writeln!(writer, "{}", vals.join("\t"))?;
                }
            } else {
                for item in arr {
                    match item {
                        Value::String(s) => writeln!(writer, "{}", escape_tsv_field(s))?,
                        Value::Null => writeln!(writer)?,
                        v => writeln!(writer, "{}", escape_tsv_field(&v.to_string()))?,
                    }
                }
            }
        }
        other => match other {
            Value::String(s) => writeln!(writer, "{}", escape_tsv_field(s))?,
            Value::Null => writeln!(writer)?,
            v => writeln!(writer, "{}", escape_tsv_field(&v.to_string()))?,
        },
    }
    Ok(())
}

fn record_headers(records: &[Value]) -> Vec<&str> {
    records
        .first()
        .and_then(Value::as_object)
        .map(|map| map.keys().map(String::as_str).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use serde_json::json;

    use super::{
        OutputFormat, escape_tsv_field, print_records, records_to_values, selected_fields,
        write_json, write_record_values, write_tsv_single,
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
    fn write_record_values_outputs_tsv_records() {
        let records = sample_records();
        let values = records_to_values(&records, None);
        let mut output = Vec::new();

        write_record_values(
            &mut output,
            &values,
            &OutputFormat::Tsv,
            &["symbol", "price"],
            None,
            false,
        )
        .unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "symbol\tprice\nAAPL\t150.5\nMSFT\t320.75\n"
        );
    }

    #[test]
    fn write_tsv_single_outputs_object_header_and_values() {
        let value = json!({"success": true, "action": "created", "key": null});
        let mut output = Vec::new();

        write_tsv_single(&mut output, &value).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        let mut lines = rendered.lines();
        let headers: Vec<&str> = lines.next().unwrap().split('\t').collect();
        let vals: Vec<&str> = lines.next().unwrap().split('\t').collect();
        let value_for =
            |name: &str| vals[headers.iter().position(|header| *header == name).unwrap()];

        assert_eq!(lines.next(), None);
        assert_eq!(value_for("action"), "created");
        assert_eq!(value_for("key"), "");
        assert_eq!(value_for("success"), "true");
    }

    #[test]
    fn escape_tsv_field_handles_special_characters() {
        assert_eq!(escape_tsv_field("plain"), "plain");
        assert_eq!(escape_tsv_field("tab\there"), "tab\\there");
        assert_eq!(escape_tsv_field("new\nline"), "new\\nline");
        assert_eq!(escape_tsv_field("cr\rreturn"), "cr\\rreturn");
        assert_eq!(escape_tsv_field("back\\slash"), "back\\\\slash");
        assert_eq!(
            escape_tsv_field("all\t\n\r\\mixed"),
            "all\\t\\n\\r\\\\mixed"
        );
    }

    #[test]
    fn write_record_values_tsv_escapes_special_characters() {
        let values = vec![json!({"name": "Tab\there", "desc": "New\nline"})];
        let mut output = Vec::new();

        write_record_values(
            &mut output,
            &values,
            &OutputFormat::Tsv,
            &["name", "desc"],
            None,
            false,
        )
        .unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "name\tdesc\nTab\\there\tNew\\nline\n"
        );
    }

    #[test]
    fn write_tsv_single_escapes_special_characters() {
        let value = json!({"note": "has\ttab\nand\nnewlines"});
        let mut output = Vec::new();

        write_tsv_single(&mut output, &value).unwrap();

        let rendered = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "note");
        assert_eq!(lines[1], "has\\ttab\\nand\\nnewlines");
    }

    #[test]
    fn print_records_rejects_unknown_custom_fields() {
        let records = sample_records();
        let err = print_records(
            &records,
            &OutputFormat::Tsv,
            &["symbol"],
            Some("ticker"),
            false,
        )
        .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("unknown output field"));
        assert!(err.to_string().contains("symbol"));
    }
}
