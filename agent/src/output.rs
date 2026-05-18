use std::io::{self, Write};

use serde::Serialize;
use serde_json::Value;

use crate::common::types::OutputFormat;

/// Writes `value` as JSON to stdout, newline-terminated.
///
/// Uses compact format when `pretty` is false, 2-space-indented when true.
pub fn print_json<T: Serialize>(value: &T, pretty: bool) -> io::Result<()> {
    write_json(&mut io::stdout().lock(), value, pretty)
}

/// Writes `records` as delimited text (CSV or TSV) to stdout with a header row.
///
/// Each record is serialized to JSON and fields are extracted in header order.
/// Null or missing fields become empty cells, booleans render as true/false.
pub fn print_delimited<T: Serialize>(
    records: &[T],
    format: OutputFormat,
    headers: &[&str],
) -> io::Result<()> {
    write_delimited(io::stdout().lock(), records, format, headers)
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
                map.retain(|key, _| fields.iter().any(|field| field == key));
            }
            value
        })
        .collect()
}

/// Outputs pre-serialized record values with compact JSON defaults and optional custom fields.
pub fn print_record_values(
    records: &[Value],
    format: OutputFormat,
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

    match format {
        OutputFormat::Json if all_fields || raw_fields_requested => print_json(&records, pretty),
        OutputFormat::Json => {
            let default_fields: Vec<String> = compact_headers
                .iter()
                .map(|field| (*field).to_string())
                .collect();
            let selected = custom_fields
                .as_deref()
                .unwrap_or(default_fields.as_slice());
            let values = filter_record_values(records, selected);
            print_json(&values, pretty)
        }
        OutputFormat::Csv | OutputFormat::Tsv => {
            let headers = if all_fields || raw_fields_requested {
                let available = available_value_fields(records);
                if available.is_empty() {
                    compact_headers
                        .iter()
                        .map(|field| (*field).to_string())
                        .collect()
                } else {
                    available
                }
            } else {
                custom_fields.unwrap_or_else(|| {
                    compact_headers
                        .iter()
                        .map(|field| (*field).to_string())
                        .collect()
                })
            };
            let header_refs: Vec<&str> = headers.iter().map(String::as_str).collect();
            print_delimited(records, format, &header_refs)
        }
    }
}

/// Checks custom output fields against record keys when records are available.
pub fn validate_record_fields<T: Serialize>(records: &[T], fields: &[String]) -> io::Result<()> {
    let available = available_record_fields(records)?;
    if available.is_empty() {
        return Ok(());
    }

    let missing: Vec<&str> = fields
        .iter()
        .map(String::as_str)
        .filter(|field| !available.iter().any(|available| available == field))
        .collect();

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

/// Outputs record lists with compact JSON defaults and optional custom fields.
///
/// JSON defaults to `compact_headers` unless `all_fields` is true. CSV and TSV
/// always use compact headers unless custom fields are supplied because they
/// require a stable column list.
pub fn print_records<T: Serialize>(
    records: &[T],
    format: OutputFormat,
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

    match format {
        OutputFormat::Json if all_fields || raw_fields_requested => print_json(&records, pretty),
        OutputFormat::Json => {
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
        OutputFormat::Csv | OutputFormat::Tsv => {
            let headers = if all_fields || raw_fields_requested {
                let available = available_record_fields(records)?;
                if available.is_empty() {
                    compact_headers
                        .iter()
                        .map(|field| (*field).to_string())
                        .collect()
                } else {
                    available
                }
            } else {
                custom_fields.unwrap_or_else(|| {
                    compact_headers
                        .iter()
                        .map(|field| (*field).to_string())
                        .collect()
                })
            };
            let header_refs: Vec<&str> = headers.iter().map(String::as_str).collect();
            print_delimited(records, format, &header_refs)
        }
    }
}

fn available_record_fields<T: Serialize>(records: &[T]) -> io::Result<Vec<String>> {
    let mut fields = Vec::new();
    for record in records {
        let value = serde_json::to_value(record)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if let Some(map) = value.as_object() {
            for key in map.keys() {
                if !fields.iter().any(|field| field == key) {
                    fields.push(key.clone());
                }
            }
        }
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
                map.retain(|key, _| fields.iter().any(|field| field == key));
            }
            value
        })
        .collect()
}

fn validate_value_fields(records: &[Value], fields: &[String]) -> io::Result<()> {
    let available = available_value_fields(records);
    if available.is_empty() {
        return Ok(());
    }

    let missing: Vec<&str> = fields
        .iter()
        .map(String::as_str)
        .filter(|field| !available.iter().any(|available| available == field))
        .collect();

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

fn available_value_fields(records: &[Value]) -> Vec<String> {
    let mut fields = Vec::new();
    for record in records {
        if let Some(map) = record.as_object() {
            for key in map.keys() {
                if !fields.iter().any(|field| field == key) {
                    fields.push(key.clone());
                }
            }
        }
    }
    fields.sort();
    fields
}

/// Prints `value` in the requested format.
///
/// For JSON, respects the `pretty` flag. CSV/TSV on single values falls back
/// to JSON since delimited formats require record slices. Use
/// [`print_delimited`] directly for record lists.
pub fn print_result<T: Serialize>(value: &T, pretty: bool, format: OutputFormat) -> io::Result<()> {
    match format {
        OutputFormat::Json => print_json(value, pretty),
        // Single values don't map to tabular output; fall back to JSON.
        // Use print_delimited directly for record slices.
        OutputFormat::Csv | OutputFormat::Tsv => print_json(value, pretty),
    }
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

/// Writes `records` as delimited text (CSV or TSV) to `writer` with a header row.
fn write_delimited<W: Write, T: Serialize>(
    writer: W,
    records: &[T],
    format: OutputFormat,
    headers: &[&str],
) -> io::Result<()> {
    let delimiter = match format {
        OutputFormat::Csv => b',',
        OutputFormat::Tsv => b'\t',
        OutputFormat::Json => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "use write_json for JSON format",
            ));
        }
    };

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(writer);

    wtr.write_record(headers).map_err(csv_to_io)?;

    for record in records {
        let value = serde_json::to_value(record)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let row: Vec<String> = headers
            .iter()
            .map(|&h| value_to_cell(value.get(h)))
            .collect();
        wtr.write_record(&row).map_err(csv_to_io)?;
    }

    wtr.flush()
}

/// Converts a JSON value to a delimited cell string.
///
/// Null or missing values become empty strings, booleans render as true/false,
/// numbers keep their JSON representation, and complex values serialize to JSON.
fn value_to_cell(value: Option<&serde_json::Value>) -> String {
    match value {
        None | Some(serde_json::Value::Null) => String::new(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Bool(b)) => b.to_string(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        Some(other) => other.to_string(),
    }
}

/// Converts a csv crate error to an io error.
fn csv_to_io(err: csv::Error) -> io::Error {
    io::Error::other(err)
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use crate::common::types::OutputFormat;

    use super::{print_records, records_to_values, selected_fields, write_delimited, write_json};

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
    fn output_csv_with_headers() {
        let records = sample_records();
        let mut buf = Vec::new();
        write_delimited(
            &mut buf,
            &records,
            OutputFormat::Csv,
            &["symbol", "price", "volume"],
        )
        .unwrap();
        let output = String::from_utf8(buf).unwrap();

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "symbol,price,volume");
        assert_eq!(lines[1], "AAPL,150.5,1000000");
        assert_eq!(lines[2], "MSFT,320.75,500000");
    }

    #[test]
    fn output_tsv_with_headers() {
        let records = sample_records();
        let mut buf = Vec::new();
        write_delimited(
            &mut buf,
            &records,
            OutputFormat::Tsv,
            &["symbol", "price", "volume"],
        )
        .unwrap();
        let output = String::from_utf8(buf).unwrap();

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "symbol\tprice\tvolume");
        assert_eq!(lines[1], "AAPL\t150.5\t1000000");
        assert_eq!(lines[2], "MSFT\t320.75\t500000");
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
        let err = print_records(
            &records,
            OutputFormat::Json,
            false,
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
