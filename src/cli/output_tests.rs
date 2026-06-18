use std::sync::{LazyLock, Mutex};

use serde::Serialize;

use super::{
    configure_strict_empty, empty_result_suggestion, finish_output, print_record_values,
    print_record_values_with_allowed_fields, print_records, print_records_with_allowed_fields,
    records_to_values, selected_fields, strict_empty_command_from_args, write_json,
    write_record_values,
};

static STRICT_EMPTY_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

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

fn with_strict_empty_test_lock<T>(test: impl FnOnce() -> T) -> T {
    let _guard = STRICT_EMPTY_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    configure_strict_empty(false, None);
    let result = test();
    configure_strict_empty(false, None);
    result
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
fn selected_fields_preserves_exact_names_and_duplicates() {
    assert_eq!(
        selected_fields(Some(" Ticker,Dollars,Ticker ")),
        Some(vec![
            "Ticker".to_string(),
            "Dollars".to_string(),
            "Ticker".to_string()
        ])
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
    let err = print_records(&records, &["symbol"], Some("ticker"), false).unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains("unknown output field"));
    assert!(err.to_string().contains("symbol"));
}

#[test]
fn write_record_values_outputs_custom_field() {
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
fn write_record_values_preserves_requested_projection_order() {
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
        Some("price,symbol"),
        false,
        None,
    )
    .unwrap();

    let output = String::from_utf8(buf).unwrap();
    assert!(
        output.starts_with(r#"[{"price":150.5,"symbol":"AAPL"}"#),
        "projection order should follow --fields order: {output}"
    );
}

#[test]
fn write_record_values_rejects_unknown_custom_fields() {
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
fn write_record_values_validates_metadata_before_strict_empty() {
    with_strict_empty_test_lock(|| {
        configure_strict_empty(true, Some("watchlist configs".to_string()));
        let allowed_fields = vec!["symbol".to_string()];
        let mut buf = Vec::new();

        let err = write_record_values(
            &mut buf,
            &[],
            &["symbol"],
            Some("price"),
            false,
            Some(&allowed_fields),
        )
        .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("price"));
    });
}

#[test]
fn print_records_validates_metadata_before_strict_empty() {
    with_strict_empty_test_lock(|| {
        configure_strict_empty(true, Some("alert configs".to_string()));
        let records: Vec<TestRecord> = Vec::new();
        let allowed_fields = vec!["symbol".to_string()];

        let err = print_records_with_allowed_fields(
            &records,
            &["symbol"],
            Some("price"),
            false,
            Some(&allowed_fields),
        )
        .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("price"));
    });
}

#[test]
fn write_record_values_outputs_raw_fields_for_all_sentinel() {
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
    assert!(finish_output(Ok(())).is_ok());
    assert!(
        finish_output(Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "trade list returned no rows; try widening filters"
        )))
        .is_err()
    );
    assert!(
        finish_output(Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "unknown output field"
        )))
        .is_err()
    );
    assert!(finish_output(Err(std::io::Error::other("broken pipe"))).is_err());
}

#[test]
fn print_records_maps_strict_empty_to_sentinel_error() {
    with_strict_empty_test_lock(|| {
        let records: Vec<TestRecord> = Vec::new();
        configure_strict_empty(true, Some("trade list".to_string()));

        let err = print_records(&records, &["symbol"], None, false).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        assert_eq!(
            err.to_string(),
            "trade list returned no rows; try checking the ticker or widening the date range"
        );
    });
}

#[test]
fn print_records_allows_empty_without_strict_flag() {
    with_strict_empty_test_lock(|| {
        let records: Vec<TestRecord> = Vec::new();
        configure_strict_empty(false, Some("trade list".to_string()));

        assert!(print_records(&records, &["symbol"], None, false).is_ok());
    });
}

#[test]
fn print_records_writes_non_empty_records_with_strict_flag() {
    with_strict_empty_test_lock(|| {
        let records = sample_records();
        configure_strict_empty(true, Some("trade list".to_string()));

        assert!(print_records(&records, &["symbol"], None, false).is_ok());
    });
}

#[test]
fn write_record_values_maps_strict_empty_to_sentinel_error() {
    with_strict_empty_test_lock(|| {
        let records = Vec::new();
        let mut buf = Vec::new();
        configure_strict_empty(true, Some("watchlist configs".to_string()));

        let err = write_record_values(&mut buf, &records, &["Name"], Some("all"), false, None)
            .unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        assert!(err.to_string().contains("valid account state"));
    });
}

#[test]
fn print_record_values_maps_strict_empty_to_sentinel_error() {
    with_strict_empty_test_lock(|| {
        configure_strict_empty(true, Some("alert configs".to_string()));

        let err = print_record_values(&[], &["Name"], None, false).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
        assert!(err.to_string().contains("valid account state"));
    });
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
