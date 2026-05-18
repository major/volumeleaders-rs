use std::collections::HashMap;

use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;

use super::{DateRange, trade_day};

const BULL_TICKERS: &[&str] = &[
    "AAPU", "AMDL", "BITU", "BOIL", "BRZU", "CURE", "CWEB", "DFEN", "DIG", "DPST", "DRN", "EDC",
    "ERX", "FAS", "FNGU", "GUSH", "HIBL", "LABU", "MIDU", "NAIL", "NVDL", "QLD", "ROM", "SOXL",
    "SPXL", "SSO", "TECL", "TMF", "TNA", "TQQQ", "TSLL", "TURB", "UDOW", "UMDD", "UPRO", "URTY",
    "USD", "UWM", "WEBL", "YINN",
];
const BEAR_TICKERS: &[&str] = &[
    "AAPD", "AMDD", "BERZ", "BITI", "BNKD", "BZQ", "DUST", "EDZ", "ERY", "FAZ", "HIBS", "KOLD",
    "LABD", "MEXZ", "MYY", "NVDD", "QID", "REK", "REW", "RXD", "SARK", "SCO", "SDD", "SDOW", "SDS",
    "SEF", "SH", "SMDD", "SOXS", "SPDN", "SPXU", "SPXS", "SQQQ", "SRS", "SSG", "SVIX", "TSDD",
    "TSLQ", "TSLS", "TZA", "UVIX", "WEBS", "YANG", "YCS", "ZSL",
];

#[derive(Debug, Serialize)]
pub(super) struct TradeSentiment {
    date_range: DateRange,
    daily: Vec<TradeSentimentDay>,
    totals: TradeSentimentTotals,
}

#[derive(Debug, Serialize)]
struct TradeSentimentDay {
    date: String,
    bear: TradeSentimentSide,
    bull: TradeSentimentSide,
    ratio: Option<f64>,
    signal: TradeSentimentSignal,
}

#[derive(Debug, Serialize)]
struct TradeSentimentTotals {
    bear: TradeSentimentSide,
    bull: TradeSentimentSide,
    ratio: Option<f64>,
    signal: TradeSentimentSignal,
}

#[derive(Clone, Debug, Default, Serialize)]
struct TradeSentimentSide {
    trades: usize,
    dollars: f64,
    top_tickers: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(super) enum TradeSentimentSignal {
    ExtremeBear,
    ModerateBear,
    Neutral,
    ModerateBull,
    ExtremeBull,
}

#[derive(Default)]
struct SentimentAccumulator {
    trades: usize,
    dollars: f64,
    ticker_dollars: HashMap<String, f64>,
}

#[derive(Default)]
struct SentimentDayAccumulator {
    bear: SentimentAccumulator,
    bull: SentimentAccumulator,
}

pub(super) fn summarize_trade_sentiment(
    trades: &[volumeleaders_client::Trade],
    start: &str,
    end: &str,
) -> TradeSentiment {
    let mut days = HashMap::<String, SentimentDayAccumulator>::new();
    let mut totals = SentimentDayAccumulator::default();
    for trade in trades {
        let Some(side) = classify_trade_sentiment_side(trade) else {
            continue;
        };
        let day = trade_day(trade);
        if day == "unknown" {
            continue;
        }
        days.entry(day).or_default().add(side, trade);
        totals.add(side, trade);
    }
    let mut day_keys: Vec<String> = days.keys().cloned().collect();
    day_keys.sort();
    let daily = day_keys
        .into_iter()
        .filter_map(|day| days.remove(&day).map(|acc| acc.summary(day)))
        .collect();
    TradeSentiment {
        date_range: DateRange {
            start: start.to_string(),
            end: end.to_string(),
        },
        daily,
        totals: totals.summary_totals(),
    }
}

impl SentimentDayAccumulator {
    fn add(&mut self, side: SentimentSide, trade: &volumeleaders_client::Trade) {
        match side {
            SentimentSide::Bear => self.bear.add(trade),
            SentimentSide::Bull => self.bull.add(trade),
        }
    }

    fn summary(self, date: String) -> TradeSentimentDay {
        let bear_dollars = self.bear.dollars;
        let bull_dollars = self.bull.dollars;
        let ratio = sentiment_ratio(bull_dollars, bear_dollars);
        TradeSentimentDay {
            date,
            bear: self.bear.summary(),
            bull: self.bull.summary(),
            ratio,
            signal: sentiment_signal(ratio, bull_dollars, bear_dollars),
        }
    }

    fn summary_totals(self) -> TradeSentimentTotals {
        let bear_dollars = self.bear.dollars;
        let bull_dollars = self.bull.dollars;
        let ratio = sentiment_ratio(bull_dollars, bear_dollars);
        TradeSentimentTotals {
            bear: self.bear.summary(),
            bull: self.bull.summary(),
            ratio,
            signal: sentiment_signal(ratio, bull_dollars, bear_dollars),
        }
    }
}

impl SentimentAccumulator {
    fn add(&mut self, trade: &volumeleaders_client::Trade) {
        self.trades += 1;
        let dollars = trade.dollars.and_then(|d| d.to_f64()).unwrap_or(0.0);
        self.dollars += dollars;
        let ticker = trade.ticker.as_deref().unwrap_or("unknown").to_string();
        *self.ticker_dollars.entry(ticker).or_default() += dollars;
    }

    fn summary(self) -> TradeSentimentSide {
        TradeSentimentSide {
            trades: self.trades,
            dollars: self.dollars,
            top_tickers: top_sentiment_tickers(self.ticker_dollars, 3),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum SentimentSide {
    Bear,
    Bull,
}

pub(super) fn classify_trade_sentiment_side(
    trade: &volumeleaders_client::Trade,
) -> Option<SentimentSide> {
    for field in [&trade.sector, &trade.name, &trade.industry]
        .into_iter()
        .filter_map(Option::as_deref)
    {
        let lower = field.to_ascii_lowercase();
        if lower.contains("bear") {
            return Some(SentimentSide::Bear);
        }
        if lower.contains("bull") {
            return Some(SentimentSide::Bull);
        }
    }
    leveraged_etf_direction(trade.ticker.as_deref().unwrap_or_default())
}

fn leveraged_etf_direction(ticker: &str) -> Option<SentimentSide> {
    let ticker = ticker.trim().to_ascii_uppercase();
    if BEAR_TICKERS.contains(&ticker.as_str()) {
        Some(SentimentSide::Bear)
    } else if BULL_TICKERS.contains(&ticker.as_str()) {
        Some(SentimentSide::Bull)
    } else {
        None
    }
}

fn sentiment_ratio(bull_dollars: f64, bear_dollars: f64) -> Option<f64> {
    if bear_dollars == 0.0 {
        None
    } else {
        Some(bull_dollars / bear_dollars)
    }
}

pub(super) fn sentiment_signal(
    ratio: Option<f64>,
    bull_dollars: f64,
    bear_dollars: f64,
) -> TradeSentimentSignal {
    match ratio {
        None => {
            if bull_dollars > 0.0 {
                TradeSentimentSignal::ExtremeBull
            } else if bear_dollars > 0.0 {
                TradeSentimentSignal::ExtremeBear
            } else {
                TradeSentimentSignal::Neutral
            }
        }
        Some(value) if value < 0.2 => TradeSentimentSignal::ExtremeBear,
        Some(value) if value < 0.5 => TradeSentimentSignal::ModerateBear,
        Some(value) if value <= 2.0 => TradeSentimentSignal::Neutral,
        Some(value) if value <= 5.0 => TradeSentimentSignal::ModerateBull,
        Some(_) => TradeSentimentSignal::ExtremeBull,
    }
}

fn top_sentiment_tickers(ticker_dollars: HashMap<String, f64>, limit: usize) -> Vec<String> {
    let mut totals: Vec<(String, f64)> = ticker_dollars.into_iter().collect();
    totals.sort_by(|(ticker_a, dollars_a), (ticker_b, dollars_b)| {
        dollars_b
            .total_cmp(dollars_a)
            .then_with(|| ticker_a.cmp(ticker_b))
    });
    totals
        .into_iter()
        .take(limit)
        .map(|(ticker, _)| ticker)
        .collect()
}
