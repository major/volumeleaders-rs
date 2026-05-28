//! Static output field metadata for commands that support `--fields`.

use serde::Serialize;

/// Machine-readable field discovery response for one command path.
#[derive(Debug, Serialize)]
pub(crate) struct FieldDiscovery {
    pub(crate) command_path: &'static str,
    pub(crate) preferred_path: &'static str,
    pub(crate) fields: &'static [FieldMetadata],
}

/// One output field accepted by `--fields` for a command.
#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct FieldMetadata {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    #[serde(rename = "type")]
    pub(crate) type_hint: FieldType,
}

/// Stable, coarse JSON type hints for output fields.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FieldType {
    String,
    Number,
    Boolean,
    Date,
    Datetime,
    Array,
    Unknown,
}

macro_rules! field {
    ($name:expr, $description:expr, $type_hint:expr) => {
        FieldMetadata {
            name: $name,
            description: $description,
            type_hint: $type_hint,
        }
    };
}

/// Looks up output field metadata by canonical command path.
#[must_use]
pub(crate) fn discover(command_path: &[String]) -> Option<FieldDiscovery> {
    let path = command_path.join(" ");
    let (command_path, fields) = fields_for_path(path.as_str())?;

    Some(FieldDiscovery {
        command_path,
        preferred_path: command_path,
        fields,
    })
}

/// Returns exact output field names accepted by `--fields` for a command path.
#[must_use]
pub(crate) fn field_names(command_path: &str) -> Option<Vec<String>> {
    fields_for_path(command_path)
        .map(|(_, fields)| fields.iter().map(|field| field.name.to_string()).collect())
}

fn fields_for_path(path: &str) -> Option<(&'static str, &'static [FieldMetadata])> {
    match path {
        "trade list" => Some(("trade list", TRADE_FIELDS)),
        "trade dashboard" => Some(("trade dashboard", DASHBOARD_FIELDS)),
        "trade levels" => Some(("trade levels", LEVEL_FIELDS)),
        "trade clusters" => Some(("trade clusters", CLUSTER_FIELDS)),
        "trade cluster-bombs" => Some(("trade cluster-bombs", BOMB_FIELDS)),
        "trade alerts" => Some(("trade alerts", TRADE_ALERT_FIELDS)),
        "report top-100-rank" => Some(("report top-100-rank", TRADE_FIELDS)),
        "report top-10-rank" => Some(("report top-10-rank", TRADE_FIELDS)),
        "report dark-pool-sweeps" => Some(("report dark-pool-sweeps", TRADE_FIELDS)),
        "report disproportionately-large" => {
            Some(("report disproportionately-large", TRADE_FIELDS))
        }
        "report leveraged-etfs" => Some(("report leveraged-etfs", TRADE_FIELDS)),
        "report rsi-overbought" => Some(("report rsi-overbought", TRADE_FIELDS)),
        "report rsi-oversold" => Some(("report rsi-oversold", TRADE_FIELDS)),
        "report dark-pool-20x" => Some(("report dark-pool-20x", TRADE_FIELDS)),
        "report top-30-rank-10x-99th" => Some(("report top-30-rank-10x-99th", TRADE_FIELDS)),
        "report phantom-trades" => Some(("report phantom-trades", TRADE_FIELDS)),
        "report offsetting-trades" => Some(("report offsetting-trades", TRADE_FIELDS)),
        "volume institutional" => Some(("volume institutional", VOLUME_FIELDS)),
        "volume ah-institutional" => Some(("volume ah-institutional", VOLUME_FIELDS)),
        "volume total" => Some(("volume total", VOLUME_FIELDS)),
        "market earnings" => Some(("market earnings", EARNINGS_FIELDS)),
        "watchlist configs" => Some(("watchlist configs", WATCHLIST_CONFIG_FIELDS)),
        "watchlist tickers" => Some(("watchlist tickers", WATCHLIST_TICKER_FIELDS)),
        "alert configs" => Some(("alert configs", ALERT_CONFIG_FIELDS)),
        _ => None,
    }
}

const TRADE_FIELDS: &[FieldMetadata] = &[
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Date", "Trading date.", FieldType::Date),
    field!(
        "Time",
        "Trade time in the market session.",
        FieldType::String
    ),
    field!("DateTime", "Trade timestamp.", FieldType::Datetime),
    field!("Price", "Trade price.", FieldType::Number),
    field!("Dollars", "Trade notional value.", FieldType::Number),
    field!(
        "DollarsMultiplier",
        "Relative dollar-size multiplier.",
        FieldType::Number
    ),
    field!(
        "CumulativeDistribution",
        "Cumulative distribution score.",
        FieldType::Number
    ),
    field!(
        "TradeRank",
        "Trade rank from VolumeLeaders.",
        FieldType::Number
    ),
    field!("RSI", "Daily RSI value when available.", FieldType::Number),
    field!(
        "type",
        "Opening or closing trade classification.",
        FieldType::String
    ),
    field!(
        "venue",
        "Dark-pool or sweep venue classification.",
        FieldType::String
    ),
    field!("Sector", "Issuer sector.", FieldType::String),
    field!("Industry", "Issuer industry.", FieldType::String),
    field!("events", "Calendar event markers.", FieldType::Array),
];

const DASHBOARD_FIELDS: &[FieldMetadata] = &[
    field!(
        "trades.Ticker",
        "Dashboard trade ticker symbol.",
        FieldType::String
    ),
    field!("trades.Date", "Dashboard trade date.", FieldType::Date),
    field!("trades.Price", "Dashboard trade price.", FieldType::Number),
    field!(
        "trades.Dollars",
        "Dashboard trade notional value.",
        FieldType::Number
    ),
    field!(
        "trades.venue",
        "Dashboard trade venue classification.",
        FieldType::String
    ),
    field!("clusters.Date", "Dashboard cluster date.", FieldType::Date),
    field!(
        "clusters.Price",
        "Dashboard cluster price.",
        FieldType::Number
    ),
    field!(
        "clusters.Dollars",
        "Dashboard cluster notional value.",
        FieldType::Number
    ),
    field!(
        "clusters.TradeCount",
        "Dashboard cluster trade count.",
        FieldType::Number
    ),
    field!("levels.Price", "Dashboard level price.", FieldType::Number),
    field!(
        "levels.Dollars",
        "Dashboard level notional value.",
        FieldType::Number
    ),
    field!(
        "levels.Trades",
        "Dashboard level trade count.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.Dollars",
        "Dashboard cluster-bomb notional value.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.TradeCount",
        "Dashboard cluster-bomb trade count.",
        FieldType::Number
    ),
];

const CLUSTER_FIELDS: &[FieldMetadata] = &[
    field!("Date", "Cluster date.", FieldType::Date),
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Price", "Cluster price.", FieldType::Number),
    field!("Dollars", "Cluster notional value.", FieldType::Number),
    field!(
        "TradeCount",
        "Number of trades in the cluster.",
        FieldType::Number
    ),
    field!(
        "DollarsMultiplier",
        "Relative dollar-size multiplier.",
        FieldType::Number
    ),
    field!(
        "CumulativeDistribution",
        "Cumulative distribution score.",
        FieldType::Number
    ),
    field!(
        "TradeClusterRank",
        "Cluster rank from VolumeLeaders.",
        FieldType::Number
    ),
    field!(
        "window",
        "Collapsed min and max cluster time window.",
        FieldType::String
    ),
    field!("events", "Calendar event markers.", FieldType::Array),
];

const LEVEL_FIELDS: &[FieldMetadata] = &[
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Price", "Significant level price.", FieldType::Number),
    field!("Dollars", "Level notional value.", FieldType::Number),
    field!(
        "Trades",
        "Number of trades at the level.",
        FieldType::Number
    ),
    field!("RelativeSize", "Relative size score.", FieldType::Number),
    field!(
        "CumulativeDistribution",
        "Cumulative distribution score.",
        FieldType::Number
    ),
    field!(
        "TradeLevelRank",
        "Level rank from VolumeLeaders.",
        FieldType::Number
    ),
];

const BOMB_FIELDS: &[FieldMetadata] = &[
    field!("Date", "Cluster-bomb date.", FieldType::Date),
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Dollars", "Cluster-bomb notional value.", FieldType::Number),
    field!(
        "TradeCount",
        "Number of trades in the cluster bomb.",
        FieldType::Number
    ),
    field!(
        "DollarsMultiplier",
        "Relative dollar-size multiplier.",
        FieldType::Number
    ),
    field!(
        "CumulativeDistribution",
        "Cumulative distribution score.",
        FieldType::Number
    ),
    field!(
        "TradeClusterBombRank",
        "Cluster-bomb rank from VolumeLeaders.",
        FieldType::Number
    ),
    field!(
        "window",
        "Collapsed min and max cluster-bomb time window.",
        FieldType::String
    ),
    field!("events", "Calendar event markers.", FieldType::Array),
];

const TRADE_ALERT_FIELDS: &[FieldMetadata] = &[
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Date", "Alert date.", FieldType::Date),
    field!("Time", "Alert time.", FieldType::String),
    field!(
        "AlertType",
        "VolumeLeaders alert category.",
        FieldType::String
    ),
    field!("TradeID", "Trade identifier.", FieldType::Number),
    field!("Price", "Alert trade price.", FieldType::Number),
    field!("Volume", "Alert trade share volume.", FieldType::Number),
    field!("Dollars", "Alert trade notional value.", FieldType::Number),
    field!(
        "TradeRank",
        "Trade rank from VolumeLeaders.",
        FieldType::Number
    ),
    field!(
        "type",
        "Opening or closing trade classification.",
        FieldType::String
    ),
    field!(
        "venue",
        "Dark-pool or sweep venue classification.",
        FieldType::String
    ),
    field!("events", "Calendar event markers.", FieldType::Array),
];

const VOLUME_FIELDS: &[FieldMetadata] = &[
    field!("Date", "Volume row date.", FieldType::Date),
    field!("FullDateTime", "Volume row timestamp.", FieldType::Datetime),
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Sector", "Issuer sector.", FieldType::String),
    field!("Industry", "Issuer industry.", FieldType::String),
    field!("Price", "Trade price.", FieldType::Number),
    field!("Dollars", "Notional value.", FieldType::Number),
    field!(
        "DollarsMultiplier",
        "Relative dollar-size multiplier.",
        FieldType::Number
    ),
    field!(
        "CumulativeDistribution",
        "Cumulative distribution score.",
        FieldType::Number
    ),
    field!(
        "TradeRank",
        "Trade rank from VolumeLeaders.",
        FieldType::Number
    ),
    field!(
        "type",
        "Opening or closing trade classification.",
        FieldType::String
    ),
    field!(
        "venue",
        "Dark-pool or sweep venue classification.",
        FieldType::String
    ),
    field!(
        "LatePrint",
        "Whether the row is a late print.",
        FieldType::Boolean
    ),
    field!(
        "SignaturePrint",
        "Whether the row is a signature print.",
        FieldType::Boolean
    ),
    field!(
        "PhantomPrint",
        "Whether the row is a phantom print.",
        FieldType::Boolean
    ),
    field!("events", "Calendar event markers.", FieldType::Array),
];

const EARNINGS_FIELDS: &[FieldMetadata] = &[
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("EarningsDate", "Reported earnings date.", FieldType::Date),
    field!(
        "AfterMarketClose",
        "Whether earnings are after market close.",
        FieldType::Boolean
    ),
    field!("TradeCount", "Related trade count.", FieldType::Number),
    field!(
        "TradeClusterCount",
        "Related trade cluster count.",
        FieldType::Number
    ),
    field!(
        "TradeClusterBombCount",
        "Related cluster-bomb count.",
        FieldType::Number
    ),
];

const WATCHLIST_CONFIG_FIELDS: &[FieldMetadata] = &[
    field!(
        "SearchTemplateKey",
        "Watchlist configuration key.",
        FieldType::Number
    ),
    field!("Name", "Watchlist name.", FieldType::String),
    field!("Tickers", "Configured ticker symbols.", FieldType::String),
    field!(
        "Criteria",
        "Serialized watchlist criteria.",
        FieldType::Unknown
    ),
];

const WATCHLIST_TICKER_FIELDS: &[FieldMetadata] = &[
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Price", "Latest known price.", FieldType::Number),
    field!(
        "NearestTop10TradeDate",
        "Nearest top-ten trade date.",
        FieldType::Date
    ),
    field!(
        "NearestTop10TradeClusterDate",
        "Nearest top-ten cluster date.",
        FieldType::Date
    ),
    field!(
        "NearestTop10TradeLevel",
        "Nearest top-ten trade level.",
        FieldType::Number
    ),
];

const ALERT_CONFIG_FIELDS: &[FieldMetadata] = &[
    field!(
        "AlertConfigKey",
        "Alert configuration key.",
        FieldType::Number
    ),
    field!("Name", "Alert configuration name.", FieldType::String),
    field!("Tickers", "Configured ticker symbols.", FieldType::String),
    field!(
        "TradeConditions",
        "Opening trade condition filter.",
        FieldType::String
    ),
    field!(
        "ClosingTradeConditions",
        "Closing trade condition filter.",
        FieldType::String
    ),
    field!(
        "DarkPool",
        "Whether dark-pool trades are included.",
        FieldType::Boolean
    ),
    field!(
        "Sweep",
        "Whether sweep trades are included.",
        FieldType::Boolean
    ),
    field!(
        "OffsettingPrint",
        "Whether offsetting prints are included.",
        FieldType::Boolean
    ),
    field!(
        "PhantomPrint",
        "Whether phantom prints are included.",
        FieldType::Boolean
    ),
];

#[cfg(test)]
mod tests {
    use super::{discover, field_names};

    #[test]
    fn discovers_required_issue_command_paths() {
        for path in [
            "trade list",
            "trade dashboard",
            "trade levels",
            "trade clusters",
            "trade cluster-bombs",
            "trade alerts",
            "report top-100-rank",
            "report top-10-rank",
            "report dark-pool-sweeps",
            "report disproportionately-large",
            "report leveraged-etfs",
            "report rsi-overbought",
            "report rsi-oversold",
            "report dark-pool-20x",
            "report top-30-rank-10x-99th",
            "report phantom-trades",
            "report offsetting-trades",
            "volume institutional",
            "volume total",
            "volume ah-institutional",
            "market earnings",
            "watchlist configs",
            "watchlist tickers",
            "alert configs",
        ] {
            let parts = path
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>();
            let discovery = discover(&parts).unwrap_or_else(|| panic!("missing {path}"));

            assert_eq!(discovery.preferred_path, path);
            assert!(!discovery.fields.is_empty(), "empty fields for {path}");
        }
    }

    #[test]
    fn rejects_commands_without_field_projection() {
        let path = ["doctor".to_string()];

        assert!(discover(&path).is_none());
    }

    #[test]
    fn returns_field_names_for_registered_commands() {
        let names = field_names("trade list").unwrap();

        assert!(names.iter().any(|name| name == "Ticker"));
        assert!(names.iter().any(|name| name == "events"));
        assert!(!names.iter().any(|name| name == "ticker"));
    }
}
