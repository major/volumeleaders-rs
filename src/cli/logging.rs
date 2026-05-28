//! Stderr-only tracing initialization for CLI diagnostics.

use std::io;

use tracing_subscriber::filter::LevelFilter;

/// Initializes CLI tracing based on count-style verbosity.
pub fn init(verbosity: u8) {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(verbosity_filter(verbosity))
        .with_writer(io::stderr)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn verbosity_filter(verbosity: u8) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::WARN,
        1 => LevelFilter::INFO,
        2 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    }
}

#[cfg(test)]
mod tests {
    use tracing_subscriber::filter::LevelFilter;

    use super::verbosity_filter;

    #[test]
    fn verbosity_count_maps_to_tracing_level() {
        assert_eq!(verbosity_filter(0), LevelFilter::WARN);
        assert_eq!(verbosity_filter(1), LevelFilter::INFO);
        assert_eq!(verbosity_filter(2), LevelFilter::DEBUG);
        assert_eq!(verbosity_filter(3), LevelFilter::TRACE);
        assert_eq!(verbosity_filter(u8::MAX), LevelFilter::TRACE);
    }
}
