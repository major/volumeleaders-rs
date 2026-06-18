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

impl OrderDirection {
    /// Returns the API string value for DataTables sorting.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

impl From<OrderDirection> for crate::datatables::SortDir {
    fn from(dir: OrderDirection) -> Self {
        match dir {
            OrderDirection::Asc => Self::Asc,
            OrderDirection::Desc => Self::Desc,
        }
    }
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
    fn order_direction_as_str() {
        assert_eq!(OrderDirection::Asc.as_str(), "asc");
        assert_eq!(OrderDirection::Desc.as_str(), "desc");
    }
}
