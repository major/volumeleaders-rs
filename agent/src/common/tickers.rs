/// Splits a comma-separated ticker string, trims whitespace, uppercases each,
/// removes empties, and deduplicates while preserving first-seen order.
pub fn parse_tickers(input: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    input
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .filter(|s| seen.insert(s.clone()))
        .collect()
}

/// Trims whitespace and uppercases a single ticker string.
pub fn parse_single_ticker(input: &str) -> String {
    input.trim().to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tickers_basic() {
        assert_eq!(
            parse_tickers("AAPL,GOOG,MSFT"),
            vec!["AAPL", "GOOG", "MSFT"]
        );
    }

    #[test]
    fn parse_tickers_whitespace() {
        assert_eq!(
            parse_tickers("  aapl , goog , msft  "),
            vec!["AAPL", "GOOG", "MSFT"]
        );
    }

    #[test]
    fn parse_tickers_uppercase() {
        assert_eq!(parse_tickers("aapl,goog"), vec!["AAPL", "GOOG"]);
    }

    #[test]
    fn parse_tickers_dedup() {
        assert_eq!(parse_tickers("AAPL,goog,aapl,GOOG"), vec!["AAPL", "GOOG"]);
    }

    #[test]
    fn parse_tickers_empty_input() {
        assert!(parse_tickers("").is_empty());
    }

    #[test]
    fn parse_tickers_empty_segments() {
        assert_eq!(parse_tickers("AAPL,,GOOG,"), vec!["AAPL", "GOOG"]);
    }

    #[test]
    fn parse_single_ticker_trims_and_uppercases() {
        assert_eq!(parse_single_ticker("  aapl  "), "AAPL");
    }

    #[test]
    fn parse_single_ticker_already_upper() {
        assert_eq!(parse_single_ticker("MSFT"), "MSFT");
    }
}
