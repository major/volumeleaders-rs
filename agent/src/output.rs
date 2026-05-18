use std::io::{self, Write};

use serde::Serialize;
use serde_json::Value;

use crate::common::trade_transforms::{TradeRecordKind, transformed_trade_values};

/// Writes `value` as compact JSON to stdout, newline-terminated.
///
/// When `json_table` is true, arrays of objects are converted to
/// array-of-arrays format with a header row before serialization.
pub fn print_json<T: Serialize>(value: &T, json_table: bool) -> io::Result<()> {
    if json_table {
        let v = serde_json::to_value(value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        match &v {
            Value::Array(arr) if arr.first().is_some_and(Value::is_object) => {
                let table = values_to_table(arr);
                write_json(&mut io::stdout().lock(), &table)
            }
            _ => write_json(&mut io::stdout().lock(), &v),
        }
    } else {
        write_json(&mut io::stdout().lock(), value)
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

/// Outputs pre-serialized record values with compact JSON defaults and optional custom fields.
pub fn print_record_values(
    records: &[Value],
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    json_table: bool,
) -> io::Result<()> {
    write_record_values(
        io::stdout().lock(),
        records,
        compact_headers,
        fields,
        all_fields,
        json_table,
    )
}

/// Transforms trade-shaped records and outputs them with field filtering.
pub fn print_transformed_record_values<T: Serialize>(
    records: &[T],
    kind: TradeRecordKind,
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    json_table: bool,
) -> io::Result<()> {
    transformed_trade_values(records, kind)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        .and_then(|values| {
            print_record_values(&values, compact_headers, fields, all_fields, json_table)
        })
}

/// Writes pre-serialized record values to `writer`.
pub(crate) fn write_record_values<W: Write>(
    mut writer: W,
    records: &[Value],
    compact_headers: &[&str],
    fields: Option<&str>,
    all_fields: bool,
    json_table: bool,
) -> io::Result<()> {
    let custom_fields = selected_fields(fields);
    let raw_fields_requested =
        fields.is_some_and(|fields| fields.trim().eq_ignore_ascii_case("all"));

    if let Some(fields) = custom_fields.as_deref() {
        validate_value_fields(records, fields)?;
    }

    if all_fields || raw_fields_requested {
        if json_table {
            let table = values_to_table(records);
            return write_json(&mut writer, &table);
        }
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
    if json_table {
        let table = values_to_table(&values);
        write_json(&mut writer, &table)
    } else {
        write_json(&mut writer, &values)
    }
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
    json_table: bool,
) -> io::Result<()> {
    let custom_fields = selected_fields(fields);
    let raw_fields_requested =
        fields.is_some_and(|fields| fields.trim().eq_ignore_ascii_case("all"));

    if let Some(fields) = custom_fields.as_deref() {
        validate_record_fields(records, fields)?;
    }

    if all_fields || raw_fields_requested {
        return print_json(&records, json_table);
    }

    let default_fields: Vec<String> = compact_headers
        .iter()
        .map(|field| (*field).to_string())
        .collect();
    let selected = custom_fields
        .as_deref()
        .unwrap_or(default_fields.as_slice());
    let values = records_to_values(records, Some(selected));
    print_json(&values, json_table)
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

/// Prints `value` as compact JSON.
pub fn print_result<T: Serialize>(value: &T, json_table: bool) -> io::Result<()> {
    print_json(value, json_table)
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

/// Converts an array of JSON objects into JSON Table format: an array whose
/// first element is the header row (field names) and remaining elements are
/// value rows. Headers are the union of all object keys, ordered by first
/// appearance across all rows. Missing keys in any row produce `null`.
pub(crate) fn values_to_table(records: &[Value]) -> Value {
    if !records.first().is_some_and(Value::is_object) {
        return Value::Array(records.to_vec());
    }

    let mut headers: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for record in records {
        if let Value::Object(obj) = record {
            for key in obj.keys() {
                if seen.insert(key.clone()) {
                    headers.push(key.clone());
                }
            }
        }
    }

    let header_row = Value::Array(headers.iter().map(|h| Value::String(h.clone())).collect());

    let mut table = Vec::with_capacity(records.len() + 1);
    table.push(header_row);

    for record in records {
        if let Value::Object(obj) = record {
            let row: Vec<Value> = headers
                .iter()
                .map(|h| obj.get(h).cloned().unwrap_or(Value::Null))
                .collect();
            table.push(Value::Array(row));
        }
    }

    Value::Array(table)
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

    use super::{print_records, records_to_values, selected_fields, values_to_table, write_json};

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
        let err = print_records(&records, &["symbol"], Some("ticker"), false, false).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("unknown output field"));
        assert!(err.to_string().contains("symbol"));
    }

    #[test]
    fn values_to_table_converts_array_of_objects() {
        let records = sample_records();
        let values: Vec<serde_json::Value> = records
            .iter()
            .map(|r| serde_json::to_value(r).unwrap())
            .collect();
        let table = values_to_table(&values);
        let rows = table.as_array().unwrap();

        assert_eq!(rows.len(), 3, "header row + 2 data rows");

        let headers = rows[0].as_array().unwrap();
        assert!(headers.contains(&serde_json::Value::String("symbol".to_string())));
        assert!(headers.contains(&serde_json::Value::String("price".to_string())));
        assert!(headers.contains(&serde_json::Value::String("volume".to_string())));

        let first_row = rows[1].as_array().unwrap();
        assert_eq!(first_row.len(), headers.len());
        assert!(first_row.contains(&serde_json::json!("AAPL")));
        assert!(first_row.contains(&serde_json::json!(150.5)));
    }

    #[test]
    fn values_to_table_returns_non_object_array_unchanged() {
        let values = vec![serde_json::json!(1), serde_json::json!(2)];
        let result = values_to_table(&values);
        assert_eq!(result, serde_json::json!([1, 2]));
    }

    #[test]
    fn values_to_table_handles_empty_array() {
        let result = values_to_table(&[]);
        assert_eq!(result, serde_json::json!([]));
    }

    #[test]
    fn values_to_table_builds_union_of_all_keys() {
        use serde_json::json;

        let records = vec![
            json!({"a": 1, "b": 2}),
            json!({"a": 3, "c": 4}),
            json!({"b": 5, "d": 6}),
        ];
        let table = values_to_table(&records);
        let rows = table.as_array().unwrap();

        assert_eq!(rows.len(), 4, "header + 3 data rows");

        let headers: Vec<&str> = rows[0]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(headers, ["a", "b", "c", "d"]);

        // Row 0: has a,b; missing c,d
        let r0 = rows[1].as_array().unwrap();
        assert_eq!(r0, &[json!(1), json!(2), json!(null), json!(null)]);

        // Row 1: has a,c; missing b,d
        let r1 = rows[2].as_array().unwrap();
        assert_eq!(r1, &[json!(3), json!(null), json!(4), json!(null)]);

        // Row 2: has b,d; missing a,c
        let r2 = rows[3].as_array().unwrap();
        assert_eq!(r2, &[json!(null), json!(5), json!(null), json!(6)]);
    }
}
