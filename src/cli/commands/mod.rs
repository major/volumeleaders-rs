//! Command handlers for each CLI subcommand group.

/// Alert configuration management commands.
pub mod alert;
/// Shell completion generation.
pub mod completions;
/// Output field discovery commands.
pub mod fields;
/// Market data commands (earnings, exhaustion scores).
pub mod market;
/// Preset report commands for common trade scans.
pub mod report;
/// Shared scaffolding for simple client-to-stdout commands.
pub(crate) mod scaffold;
/// Trade lookup, dashboards, sentiment, and analysis commands.
pub mod trade;
/// Volume leaderboard commands.
pub mod volume;
/// Watchlist management and inspection commands.
pub mod watchlist;
