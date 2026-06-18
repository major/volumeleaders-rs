use super::{FieldMetadata, FieldType};

/// Default compact output columns for trade list and report commands.
pub(crate) const TRADE_HEADERS: &[&str] = &[
    "FullTimeString24",
    "Volume",
    "Price",
    "Dollars",
    "DollarsMultiplier",
    "TradeRank",
    "LastComparibleTradeDate",
];

pub(crate) const CLUSTER_HEADERS: &[&str] = &[
    "MinFullTimeString24",
    "TradeCount",
    "Price",
    "Dollars",
    "DollarsMultiplier",
    "TradeClusterRank",
    "LastComparibleTradeClusterDate",
];

pub(crate) const BOMB_HEADERS: &[&str] = &[
    "MinFullTimeString24",
    "TradeCount",
    "Volume",
    "Dollars",
    "DollarsMultiplier",
    "CumulativeDistribution",
    "TradeClusterBombRank",
    "LastComparableTradeClusterBombDate",
];

pub(crate) const LEVEL_HEADERS: &[&str] = &[
    "Price",
    "Dollars",
    "Volume",
    "Trades",
    "RelativeSize",
    "CumulativeDistribution",
    "TradeLevelRank",
    "Dates",
];

pub(crate) const LEVEL_TOUCHES_HEADERS: &[&str] = &[
    "Ticker",
    "FullTimeString24",
    "Price",
    "Dollars",
    "Volume",
    "Trades",
    "RelativeSize",
    "TradeLevelRank",
    "Dates",
];

pub(crate) const ALERT_HEADERS: &[&str] = &[
    "FullTimeString24",
    "AlertType",
    "TradeID",
    "Price",
    "Volume",
    "Dollars",
    "DollarsMultiplier",
    "TradeRank",
    "LastComparibleTradeDate",
    "DarkPool",
    "Sweep",
];

pub(crate) const VOLUME_HEADERS: &[&str] = &[
    "Date",
    "FullDateTime",
    "Ticker",
    "Sector",
    "Industry",
    "Price",
    "Dollars",
    "DollarsMultiplier",
    "CumulativeDistribution",
    "TradeRank",
    "OpeningTrade",
    "ClosingTrade",
    "DarkPool",
    "Sweep",
    "LatePrint",
    "SignaturePrint",
    "PhantomPrint",
];

pub(super) const FIELD_TABLES: &[(&str, &[FieldMetadata])] = &[
    ("trade dashboard", DASHBOARD_FIELDS),
    ("trade alerts", TRADE_ALERT_FIELDS),
    ("volume institutional", VOLUME_FIELDS),
    ("volume ah-institutional", VOLUME_FIELDS),
    ("volume total", VOLUME_FIELDS),
    ("market earnings", EARNINGS_FIELDS),
    ("watchlist configs", WATCHLIST_CONFIG_FIELDS),
    ("watchlist tickers", WATCHLIST_TICKER_FIELDS),
    ("alert configs", ALERT_CONFIG_FIELDS),
];

pub(super) const TRADE_FIELDS: &[FieldMetadata] = &[
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Date", "Trading date.", FieldType::Date),
    field!(
        "FullTimeString24",
        "Trade time in the market session.",
        FieldType::String
    ),
    field!("FullDateTime", "Trade timestamp.", FieldType::Datetime),
    field!("Price", "Trade price.", FieldType::Number),
    field!("Volume", "Trade share volume.", FieldType::Number),
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
    field!(
        "LastComparibleTradeDate",
        "Last date when a trade that large was seen.",
        FieldType::Date
    ),
    field!(
        "DarkPool",
        "Whether the trade was dark-pool volume.",
        FieldType::Boolean
    ),
    field!(
        "Sweep",
        "Whether the trade was a sweep.",
        FieldType::Boolean
    ),
    field!(
        "OpeningTrade",
        "Whether the row is an opening trade.",
        FieldType::Boolean
    ),
    field!(
        "ClosingTrade",
        "Whether the row is a closing trade.",
        FieldType::Boolean
    ),
    field!("Sector", "Issuer sector.", FieldType::String),
    field!("Industry", "Issuer industry.", FieldType::String),
];

pub(super) const DASHBOARD_FIELDS: &[FieldMetadata] = &[
    field!("trades.Date", "Dashboard trade date.", FieldType::Date),
    field!(
        "trades.StartDate",
        "Dashboard trade start date.",
        FieldType::Date
    ),
    field!(
        "trades.EndDate",
        "Dashboard trade end date.",
        FieldType::Date
    ),
    field!("trades.TD30", "Prior 30-trading-day date.", FieldType::Date),
    field!("trades.TD90", "Prior 90-trading-day date.", FieldType::Date),
    field!(
        "trades.TD1CY",
        "Prior one-calendar-year date.",
        FieldType::Date
    ),
    field!(
        "trades.DateKey",
        "Dashboard trade date key.",
        FieldType::Number
    ),
    field!(
        "trades.TimeKey",
        "Dashboard trade time key.",
        FieldType::Number
    ),
    field!(
        "trades.SecurityKey",
        "Dashboard trade security key.",
        FieldType::Number
    ),
    field!(
        "trades.TradeID",
        "Dashboard trade identifier.",
        FieldType::Number
    ),
    field!(
        "trades.SequenceNumber",
        "Dashboard trade sequence number.",
        FieldType::Number
    ),
    field!(
        "trades.EOM",
        "Dashboard trade end-of-month flag.",
        FieldType::Boolean
    ),
    field!(
        "trades.EOQ",
        "Dashboard trade end-of-quarter flag.",
        FieldType::Boolean
    ),
    field!(
        "trades.EOY",
        "Dashboard trade end-of-year flag.",
        FieldType::Boolean
    ),
    field!(
        "trades.OPEX",
        "Dashboard trade options-expiration flag.",
        FieldType::Boolean
    ),
    field!(
        "trades.VOLEX",
        "Dashboard trade volatility-expiration flag.",
        FieldType::Boolean
    ),
    field!(
        "trades.Ticker",
        "Dashboard trade ticker symbol.",
        FieldType::String
    ),
    field!(
        "trades.Sector",
        "Dashboard trade issuer sector.",
        FieldType::String
    ),
    field!(
        "trades.Industry",
        "Dashboard trade issuer industry.",
        FieldType::String
    ),
    field!(
        "trades.Name",
        "Dashboard trade issuer name.",
        FieldType::String
    ),
    field!(
        "trades.FullDateTime",
        "Dashboard trade timestamp.",
        FieldType::Datetime
    ),
    field!(
        "trades.FullTimeString24",
        "Dashboard trade time.",
        FieldType::String
    ),
    field!("trades.Price", "Dashboard trade price.", FieldType::Number),
    field!(
        "trades.Bid",
        "Dashboard trade bid price.",
        FieldType::Number
    ),
    field!(
        "trades.Ask",
        "Dashboard trade ask price.",
        FieldType::Number
    ),
    field!(
        "trades.Dollars",
        "Dashboard trade notional value.",
        FieldType::Number
    ),
    field!(
        "trades.AverageBlockSizeDollars",
        "Dashboard trade average block size in dollars.",
        FieldType::Number
    ),
    field!(
        "trades.AverageBlockSizeShares",
        "Dashboard trade average block size in shares.",
        FieldType::Number
    ),
    field!(
        "trades.DollarsMultiplier",
        "Dashboard trade relative dollar-size multiplier.",
        FieldType::Number
    ),
    field!(
        "trades.Volume",
        "Dashboard trade share volume.",
        FieldType::Number
    ),
    field!(
        "trades.AverageDailyVolume",
        "Dashboard trade average daily volume.",
        FieldType::Number
    ),
    field!(
        "trades.PercentDailyVolume",
        "Dashboard trade percent of daily volume.",
        FieldType::Number
    ),
    field!(
        "trades.RelativeSize",
        "Dashboard trade relative size score.",
        FieldType::Number
    ),
    field!(
        "trades.LastComparibleTradeDate",
        "Dashboard trade last comparable trade date.",
        FieldType::Date
    ),
    field!(
        "trades.IPODate",
        "Dashboard trade issuer IPO date.",
        FieldType::Date
    ),
    field!(
        "trades.OffsettingTradeDate",
        "Dashboard trade offsetting trade date.",
        FieldType::Date
    ),
    field!(
        "trades.PhantomPrintFulfillmentDate",
        "Dashboard trade phantom-print fulfillment date.",
        FieldType::Date
    ),
    field!(
        "trades.PhantomPrintFulfillmentDays",
        "Dashboard trade phantom-print fulfillment days.",
        FieldType::Number
    ),
    field!(
        "trades.TradeCount",
        "Dashboard trade count.",
        FieldType::Number
    ),
    field!(
        "trades.CumulativeDistribution",
        "Dashboard trade cumulative distribution score.",
        FieldType::Number
    ),
    field!(
        "trades.TradeRank",
        "Dashboard trade rank.",
        FieldType::Number
    ),
    field!(
        "trades.TradeRankSnapshot",
        "Dashboard trade rank snapshot.",
        FieldType::Number
    ),
    field!(
        "trades.LatePrint",
        "Whether the dashboard trade is a late print.",
        FieldType::Boolean
    ),
    field!(
        "trades.Sweep",
        "Whether the dashboard trade is a sweep.",
        FieldType::Boolean
    ),
    field!(
        "trades.DarkPool",
        "Whether the dashboard trade is dark-pool volume.",
        FieldType::Boolean
    ),
    field!(
        "trades.OpeningTrade",
        "Whether the dashboard trade is opening flow.",
        FieldType::Boolean
    ),
    field!(
        "trades.ClosingTrade",
        "Whether the dashboard trade is closing flow.",
        FieldType::Boolean
    ),
    field!(
        "trades.PhantomPrint",
        "Whether the dashboard trade is a phantom print.",
        FieldType::Boolean
    ),
    field!(
        "trades.InsideBar",
        "Whether the dashboard trade is inside bar activity.",
        FieldType::Boolean
    ),
    field!(
        "trades.DoubleInsideBar",
        "Whether the dashboard trade is double inside bar activity.",
        FieldType::Boolean
    ),
    field!(
        "trades.SignaturePrint",
        "Whether the dashboard trade is a signature print.",
        FieldType::Boolean
    ),
    field!(
        "trades.NewPosition",
        "Whether the dashboard trade is a new position.",
        FieldType::Boolean
    ),
    field!(
        "trades.AHInstitutionalDollars",
        "After-hours institutional dollars.",
        FieldType::Number
    ),
    field!(
        "trades.AHInstitutionalDollarsRank",
        "After-hours institutional dollars rank.",
        FieldType::Number
    ),
    field!(
        "trades.AHInstitutionalVolume",
        "After-hours institutional volume.",
        FieldType::Number
    ),
    field!(
        "trades.TotalInstitutionalDollars",
        "Total institutional dollars.",
        FieldType::Number
    ),
    field!(
        "trades.TotalInstitutionalDollarsRank",
        "Total institutional dollars rank.",
        FieldType::Number
    ),
    field!(
        "trades.TotalInstitutionalVolume",
        "Total institutional volume.",
        FieldType::Number
    ),
    field!(
        "trades.ClosingTradeDollars",
        "Closing trade dollars.",
        FieldType::Number
    ),
    field!(
        "trades.ClosingTradeDollarsRank",
        "Closing trade dollars rank.",
        FieldType::Number
    ),
    field!(
        "trades.ClosingTradeVolume",
        "Closing trade volume.",
        FieldType::Number
    ),
    field!(
        "trades.TotalDollars",
        "Total trade dollars.",
        FieldType::Number
    ),
    field!(
        "trades.TotalDollarsRank",
        "Total trade dollars rank.",
        FieldType::Number
    ),
    field!(
        "trades.TotalVolume",
        "Total trade volume.",
        FieldType::Number
    ),
    field!(
        "trades.ClosePrice",
        "Dashboard trade close price.",
        FieldType::Number
    ),
    field!(
        "trades.RSIHour",
        "Dashboard trade hourly RSI value.",
        FieldType::Number
    ),
    field!(
        "trades.RSIDay",
        "Dashboard trade daily RSI value.",
        FieldType::Number
    ),
    field!(
        "trades.TotalRows",
        "Dashboard trade total row count.",
        FieldType::Number
    ),
    field!(
        "trades.TradeConditions",
        "Dashboard trade condition text.",
        FieldType::String
    ),
    field!(
        "trades.FrequencyLast30TD",
        "Dashboard trade frequency over the last 30 trading days.",
        FieldType::Number
    ),
    field!(
        "trades.FrequencyLast90TD",
        "Dashboard trade frequency over the last 90 trading days.",
        FieldType::Number
    ),
    field!(
        "trades.FrequencyLast1CY",
        "Dashboard trade frequency over the last calendar year.",
        FieldType::Number
    ),
    field!(
        "trades.Cancelled",
        "Whether the dashboard trade was cancelled.",
        FieldType::Boolean
    ),
    field!(
        "trades.TotalTrades",
        "Dashboard trade total trade count.",
        FieldType::Number
    ),
    field!(
        "trades.ExternalFeed",
        "Whether the dashboard trade came from an external feed.",
        FieldType::Boolean
    ),
    field!("clusters.Date", "Dashboard cluster date.", FieldType::Date),
    field!(
        "clusters.IPODate",
        "Dashboard cluster issuer IPO date.",
        FieldType::Date
    ),
    field!(
        "clusters.DateKey",
        "Dashboard cluster date key.",
        FieldType::Number
    ),
    field!(
        "clusters.SecurityKey",
        "Dashboard cluster security key.",
        FieldType::Number
    ),
    field!(
        "clusters.Ticker",
        "Dashboard cluster ticker symbol.",
        FieldType::String
    ),
    field!(
        "clusters.Sector",
        "Dashboard cluster issuer sector.",
        FieldType::String
    ),
    field!(
        "clusters.Industry",
        "Dashboard cluster issuer industry.",
        FieldType::String
    ),
    field!(
        "clusters.Name",
        "Dashboard cluster issuer name.",
        FieldType::String
    ),
    field!(
        "clusters.MinFullDateTime",
        "Dashboard cluster earliest timestamp.",
        FieldType::Datetime
    ),
    field!(
        "clusters.MaxFullDateTime",
        "Dashboard cluster latest timestamp.",
        FieldType::Datetime
    ),
    field!(
        "clusters.MinFullTimeString24",
        "Dashboard cluster earliest time.",
        FieldType::String
    ),
    field!(
        "clusters.MaxFullTimeString24",
        "Dashboard cluster latest time.",
        FieldType::String
    ),
    field!(
        "clusters.Price",
        "Dashboard cluster price.",
        FieldType::Number
    ),
    field!(
        "clusters.ClosePrice",
        "Dashboard cluster close price.",
        FieldType::Number
    ),
    field!(
        "clusters.Dollars",
        "Dashboard cluster notional value.",
        FieldType::Number
    ),
    field!(
        "clusters.AverageBlockSizeShares",
        "Dashboard cluster average block size in shares.",
        FieldType::Number
    ),
    field!(
        "clusters.AverageBlockSizeDollars",
        "Dashboard cluster average block size in dollars.",
        FieldType::Number
    ),
    field!(
        "clusters.Volume",
        "Dashboard cluster share volume.",
        FieldType::Number
    ),
    field!(
        "clusters.AverageDailyVolume",
        "Dashboard cluster average daily volume.",
        FieldType::Number
    ),
    field!(
        "clusters.TradeCount",
        "Dashboard cluster trade count.",
        FieldType::Number
    ),
    field!(
        "clusters.DollarsMultiplier",
        "Dashboard cluster relative dollar-size multiplier.",
        FieldType::Number
    ),
    field!(
        "clusters.CumulativeDistribution",
        "Dashboard cluster cumulative distribution score.",
        FieldType::Number
    ),
    field!(
        "clusters.TradeClusterRank",
        "Dashboard cluster rank.",
        FieldType::Number
    ),
    field!(
        "clusters.LastComparibleTradeClusterDate",
        "Dashboard cluster last comparable cluster date.",
        FieldType::Date
    ),
    field!(
        "clusters.EOM",
        "Dashboard cluster end-of-month flag.",
        FieldType::Boolean
    ),
    field!(
        "clusters.EOQ",
        "Dashboard cluster end-of-quarter flag.",
        FieldType::Boolean
    ),
    field!(
        "clusters.EOY",
        "Dashboard cluster end-of-year flag.",
        FieldType::Boolean
    ),
    field!(
        "clusters.OPEX",
        "Dashboard cluster options-expiration flag.",
        FieldType::Boolean
    ),
    field!(
        "clusters.VOLEX",
        "Dashboard cluster volatility-expiration flag.",
        FieldType::Boolean
    ),
    field!(
        "clusters.InsideBar",
        "Whether the dashboard cluster is inside bar activity.",
        FieldType::Boolean
    ),
    field!(
        "clusters.DoubleInsideBar",
        "Whether the dashboard cluster is double inside bar activity.",
        FieldType::Boolean
    ),
    field!(
        "clusters.TotalRows",
        "Dashboard cluster total row count.",
        FieldType::Number
    ),
    field!(
        "clusters.ExternalFeed",
        "Whether the dashboard cluster came from an external feed.",
        FieldType::Boolean
    ),
    field!(
        "levels.Ticker",
        "Dashboard level ticker symbol.",
        FieldType::String
    ),
    field!(
        "levels.Sector",
        "Dashboard level issuer sector.",
        FieldType::String
    ),
    field!(
        "levels.Industry",
        "Dashboard level issuer industry.",
        FieldType::String
    ),
    field!(
        "levels.Name",
        "Dashboard level issuer name.",
        FieldType::String
    ),
    field!("levels.Date", "Dashboard level date.", FieldType::Date),
    field!(
        "levels.MinDate",
        "Dashboard level minimum date.",
        FieldType::Date
    ),
    field!(
        "levels.MaxDate",
        "Dashboard level maximum date.",
        FieldType::Date
    ),
    field!(
        "levels.FullDateTime",
        "Dashboard level timestamp.",
        FieldType::Datetime
    ),
    field!(
        "levels.FullTimeString24",
        "Dashboard level time.",
        FieldType::String
    ),
    field!(
        "levels.Dates",
        "Dashboard level date list.",
        FieldType::String
    ),
    field!("levels.Price", "Dashboard level price.", FieldType::Number),
    field!(
        "levels.Dollars",
        "Dashboard level notional value.",
        FieldType::Number
    ),
    field!(
        "levels.Volume",
        "Dashboard level share volume.",
        FieldType::Number
    ),
    field!(
        "levels.Trades",
        "Dashboard level trade count.",
        FieldType::Number
    ),
    field!(
        "levels.RelativeSize",
        "Dashboard level relative size score.",
        FieldType::Number
    ),
    field!(
        "levels.CumulativeDistribution",
        "Dashboard level cumulative distribution score.",
        FieldType::Number
    ),
    field!(
        "levels.TradeLevelRank",
        "Dashboard level rank.",
        FieldType::Number
    ),
    field!(
        "levels.TradeLevelTouches",
        "Dashboard level touch count.",
        FieldType::Number
    ),
    field!(
        "levels.TotalRows",
        "Dashboard level total row count.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.Date",
        "Dashboard cluster-bomb date.",
        FieldType::Date
    ),
    field!(
        "cluster_bombs.IPODate",
        "Dashboard cluster-bomb issuer IPO date.",
        FieldType::Date
    ),
    field!(
        "cluster_bombs.DateKey",
        "Dashboard cluster-bomb date key.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.SecurityKey",
        "Dashboard cluster-bomb security key.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.Ticker",
        "Dashboard cluster-bomb ticker symbol.",
        FieldType::String
    ),
    field!(
        "cluster_bombs.Sector",
        "Dashboard cluster-bomb issuer sector.",
        FieldType::String
    ),
    field!(
        "cluster_bombs.Industry",
        "Dashboard cluster-bomb issuer industry.",
        FieldType::String
    ),
    field!(
        "cluster_bombs.Name",
        "Dashboard cluster-bomb issuer name.",
        FieldType::String
    ),
    field!(
        "cluster_bombs.MinFullDateTime",
        "Dashboard cluster-bomb earliest timestamp.",
        FieldType::Datetime
    ),
    field!(
        "cluster_bombs.MaxFullDateTime",
        "Dashboard cluster-bomb latest timestamp.",
        FieldType::Datetime
    ),
    field!(
        "cluster_bombs.MinFullTimeString24",
        "Dashboard cluster-bomb earliest time.",
        FieldType::String
    ),
    field!(
        "cluster_bombs.MaxFullTimeString24",
        "Dashboard cluster-bomb latest time.",
        FieldType::String
    ),
    field!(
        "cluster_bombs.ClosePrice",
        "Dashboard cluster-bomb close price.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.Dollars",
        "Dashboard cluster-bomb notional value.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.AverageBlockSizeShares",
        "Dashboard cluster-bomb average block size in shares.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.AverageBlockSizeDollars",
        "Dashboard cluster-bomb average block size in dollars.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.Volume",
        "Dashboard cluster-bomb share volume.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.AverageDailyVolume",
        "Dashboard cluster-bomb average daily volume.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.TradeCount",
        "Dashboard cluster-bomb trade count.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.DollarsMultiplier",
        "Dashboard cluster-bomb relative dollar-size multiplier.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.CumulativeDistribution",
        "Dashboard cluster-bomb cumulative distribution score.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.TradeClusterBombRank",
        "Dashboard cluster-bomb rank.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.LastComparableTradeClusterBombDate",
        "Dashboard cluster-bomb last comparable date.",
        FieldType::Date
    ),
    field!(
        "cluster_bombs.EOM",
        "Dashboard cluster-bomb end-of-month flag.",
        FieldType::Boolean
    ),
    field!(
        "cluster_bombs.EOQ",
        "Dashboard cluster-bomb end-of-quarter flag.",
        FieldType::Boolean
    ),
    field!(
        "cluster_bombs.EOY",
        "Dashboard cluster-bomb end-of-year flag.",
        FieldType::Boolean
    ),
    field!(
        "cluster_bombs.OPEX",
        "Dashboard cluster-bomb options-expiration flag.",
        FieldType::Boolean
    ),
    field!(
        "cluster_bombs.VOLEX",
        "Dashboard cluster-bomb volatility-expiration flag.",
        FieldType::Boolean
    ),
    field!(
        "cluster_bombs.InsideBar",
        "Whether the dashboard cluster bomb is inside bar activity.",
        FieldType::Boolean
    ),
    field!(
        "cluster_bombs.DoubleInsideBar",
        "Whether the dashboard cluster bomb is double inside bar activity.",
        FieldType::Boolean
    ),
    field!(
        "cluster_bombs.TotalRows",
        "Dashboard cluster-bomb total row count.",
        FieldType::Number
    ),
    field!(
        "cluster_bombs.ExternalFeed",
        "Whether the dashboard cluster bomb came from an external feed.",
        FieldType::Boolean
    ),
];

pub(super) const CLUSTER_FIELDS: &[FieldMetadata] = &[
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
        "MinFullTimeString24",
        "Earliest trade time in the cluster.",
        FieldType::String
    ),
    field!(
        "LastComparibleTradeClusterDate",
        "Last date when a comparable cluster was seen.",
        FieldType::Date
    ),
];

pub(super) const LEVEL_FIELDS: &[FieldMetadata] = &[
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Price", "Significant level price.", FieldType::Number),
    field!("Dollars", "Level notional value.", FieldType::Number),
    field!("Volume", "Total shares at the level.", FieldType::Number),
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
    field!("Dates", "Dates when the level existed.", FieldType::String),
];

pub(super) const BOMB_FIELDS: &[FieldMetadata] = &[
    field!("Date", "Cluster-bomb date.", FieldType::Date),
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!(
        "MinFullTimeString24",
        "Earliest trade time in the cluster bomb.",
        FieldType::String
    ),
    field!("Volume", "Cluster-bomb share volume.", FieldType::Number),
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
        "LastComparableTradeClusterBombDate",
        "Last date when a comparable cluster bomb was seen.",
        FieldType::Date
    ),
];

pub(super) const TRADE_ALERT_FIELDS: &[FieldMetadata] = &[
    field!("Ticker", "Ticker symbol.", FieldType::String),
    field!("Date", "Alert date.", FieldType::Date),
    field!("FullTimeString24", "Alert trade time.", FieldType::String),
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
        "DollarsMultiplier",
        "Alert trade relative dollar-size multiplier.",
        FieldType::Number
    ),
    field!(
        "TradeRank",
        "Trade rank from VolumeLeaders.",
        FieldType::Number
    ),
    field!(
        "LastComparibleTradeDate",
        "Last comparable trade date.",
        FieldType::Date
    ),
    field!(
        "DarkPool",
        "Whether the alert trade was dark-pool volume.",
        FieldType::Boolean
    ),
    field!(
        "Sweep",
        "Whether the alert trade was a sweep.",
        FieldType::Boolean
    ),
];

pub(super) const VOLUME_FIELDS: &[FieldMetadata] = &[
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
        "OpeningTrade",
        "Whether the row is opening flow.",
        FieldType::Boolean
    ),
    field!(
        "ClosingTrade",
        "Whether the row is closing flow.",
        FieldType::Boolean
    ),
    field!(
        "DarkPool",
        "Whether the row is dark-pool volume.",
        FieldType::Boolean
    ),
    field!("Sweep", "Whether the row is a sweep.", FieldType::Boolean),
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
];

pub(super) const EARNINGS_FIELDS: &[FieldMetadata] = &[
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

pub(super) const WATCHLIST_CONFIG_FIELDS: &[FieldMetadata] = &[
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

pub(super) const WATCHLIST_TICKER_FIELDS: &[FieldMetadata] = &[
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

pub(super) const ALERT_CONFIG_FIELDS: &[FieldMetadata] = &[
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
