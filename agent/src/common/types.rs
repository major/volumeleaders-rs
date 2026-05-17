/// Toggle-style filter used by the VolumeLeaders API.
///
/// A value of -1 leaves the filter unselected, 0 excludes matching rows,
/// and 1 returns only matching rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, clap::ValueEnum)]
pub enum TriStateFilter {
    /// Leave the filter unselected (default).
    #[default]
    #[value(name = "-1")]
    All,
    /// Exclude rows matching the filter.
    #[value(name = "0")]
    Disabled,
    /// Return only rows matching the filter.
    #[value(name = "1")]
    Enabled,
}

impl TriStateFilter {
    /// Returns the integer value expected by API filter parameters.
    pub fn as_i8(self) -> i8 {
        match self {
            Self::All => -1,
            Self::Disabled => 0,
            Self::Enabled => 1,
        }
    }
}

/// Sort direction for DataTables API requests.
#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
pub enum OrderDirection {
    /// Ascending order.
    #[value(name = "asc")]
    Asc,
    /// Descending order.
    #[value(name = "desc")]
    Desc,
}

/// Supported output formats for CLI commands.
#[derive(Clone, Copy, Debug, Default, PartialEq, clap::ValueEnum)]
pub enum OutputFormat {
    /// JSON output (default).
    #[default]
    #[value(name = "json")]
    Json,
    /// Comma-separated values.
    #[value(name = "csv")]
    Csv,
    /// Tab-separated values.
    #[value(name = "tsv")]
    Tsv,
}

/// Grouping dimension for summary commands.
#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
pub enum SummaryGroup {
    /// Group by ticker symbol.
    #[value(name = "ticker")]
    Ticker,
    /// Group by trading day.
    #[value(name = "day")]
    Day,
    /// Group by ticker and day combined.
    #[value(name = "ticker,day")]
    TickerDay,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tri_state_filter_as_i8() {
        assert_eq!(TriStateFilter::All.as_i8(), -1);
        assert_eq!(TriStateFilter::Disabled.as_i8(), 0);
        assert_eq!(TriStateFilter::Enabled.as_i8(), 1);
    }

    #[test]
    fn tri_state_filter_default_is_all() {
        assert_eq!(TriStateFilter::default(), TriStateFilter::All);
    }

    #[test]
    fn output_format_default_is_json() {
        assert_eq!(OutputFormat::default(), OutputFormat::Json);
    }
}
