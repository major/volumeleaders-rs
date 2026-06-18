//! Shared utilities used across command handlers.

/// Browser auth session bootstrapping.
pub mod auth;
/// Date range resolution and formatting.
pub mod dates;
/// Number formatting helpers.
pub mod format;
/// Ticker symbol parsing and normalization.
pub mod tickers;
/// Trade-shaped row kind definitions.
pub mod trade_record_kind;
/// Shared CLI types: order direction, summary groups, tri-state filters.
pub mod types;

pub use dates::DATE_FMT;
pub use types::{OrderDirection, SummaryGroup, TriStateFilter};
