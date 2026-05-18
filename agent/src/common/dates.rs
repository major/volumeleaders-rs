use chrono::Local;

/// Date format string used for all CLI date arguments and API requests.
pub const DATE_FMT: &str = "%Y-%m-%d";
const DEFAULT_LOOKBACK_DAYS: u32 = 5;

/// Returns today's date as a `YYYY-MM-DD` string in local time.
pub fn today_str() -> String {
    Local::now().date_naive().format(DATE_FMT).to_string()
}

/// Returns the date `n` days ago as a `YYYY-MM-DD` string in local time.
pub fn n_days_ago(n: u32) -> String {
    let today = Local::now().date_naive();
    (today - chrono::Duration::days(i64::from(n)))
        .format(DATE_FMT)
        .to_string()
}

/// Resolves start/end date range from CLI arguments.
///
/// Priority order:
/// 1. `days` set: end defaults to today (or `end` if given), start = end - days.
/// 2. Both `start` and `end` provided: use as-is.
/// 3. Only `start` provided: end = today.
/// 4. Nothing provided: 5-day lookback from today.
pub fn resolve_date_range(
    start: Option<&str>,
    end: Option<&str>,
    days: Option<u32>,
) -> (String, String) {
    let today = Local::now().date_naive();

    // If --days is set, it takes priority.
    if let Some(d) = days {
        let base = end
            .and_then(|e| chrono::NaiveDate::parse_from_str(e, DATE_FMT).ok())
            .unwrap_or(today);
        let start_date = (base - chrono::Duration::days(i64::from(d)))
            .format(DATE_FMT)
            .to_string();
        let end_date = base.format(DATE_FMT).to_string();
        return (start_date, end_date);
    }

    match (start, end) {
        (Some(s), Some(e)) => (s.to_owned(), e.to_owned()),
        (Some(s), None) => (s.to_owned(), today.format(DATE_FMT).to_string()),
        _ => {
            // Default: last 5 days.
            let start_date = (today - chrono::Duration::days(i64::from(DEFAULT_LOOKBACK_DAYS)))
                .format(DATE_FMT)
                .to_string();
            (start_date, today.format(DATE_FMT).to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_str_format() {
        let today = today_str();
        // Should be YYYY-MM-DD format.
        assert_eq!(today.len(), 10);
        assert_eq!(&today[4..5], "-");
        assert_eq!(&today[7..8], "-");
    }

    #[test]
    fn n_days_ago_is_before_today() {
        let ago = n_days_ago(5);
        let today = today_str();
        assert!(ago < today);
    }

    #[test]
    fn resolve_with_days() {
        let (start, end) = resolve_date_range(None, None, Some(3));
        // End should be today.
        assert_eq!(end, today_str());
        // Start should be 3 days before today.
        assert_eq!(start, n_days_ago(3));
    }

    #[test]
    fn resolve_with_explicit_dates() {
        let (start, end) = resolve_date_range(Some("2025-01-01"), Some("2025-01-15"), None);
        assert_eq!(start, "2025-01-01");
        assert_eq!(end, "2025-01-15");
    }

    #[test]
    fn resolve_start_only() {
        let (start, end) = resolve_date_range(Some("2025-01-01"), None, None);
        assert_eq!(start, "2025-01-01");
        assert_eq!(end, today_str());
    }

    #[test]
    fn resolve_default_5_day_lookback() {
        let (start, end) = resolve_date_range(None, None, None);
        assert_eq!(end, today_str());
        assert_eq!(start, n_days_ago(5));
    }

    #[test]
    fn resolve_days_with_end_date() {
        let (start, end) = resolve_date_range(None, Some("2025-06-15"), Some(10));
        assert_eq!(end, "2025-06-15");
        assert_eq!(start, "2025-06-05");
    }
}
