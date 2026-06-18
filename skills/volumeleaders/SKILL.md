---
name: volumeleaders
description: Operate the volumeleaders-agent CLI and interpret its output. Covers authentication, command invocation, self-discovery, field glossary, significance thresholds, and analysis patterns for institutional trade data (trades, clusters, levels, cluster bombs). Use when an agent needs to call the CLI, understand the output fields, identify notable activity, or summarize findings for users.
---

# VolumeLeaders Agent Skill

Complete guide to operating the `volumeleaders-agent` CLI and interpreting its institutional trade data output.

## CLI Invocation

### Authentication

The CLI authenticates with a VolumeLeaders account. Auth sources are checked in order:

1. **Cached session** — reuses cookies from a prior login (stored in XDG cache)
2. **Environment variables** — `VL_USERNAME` and `VL_PASSWORD` (both must be set and non-empty; if either is set, config file is skipped)
3. **Config file** — `~/.config/volumeleaders-agent/config.json` with `username` and `password` fields

Local discovery commands (`doctor`, `schema`, `commands`, `fields`, `help`, `completions`) do not require authentication. All live data commands do.

### Command Catalog

| Group | Leaf commands |
|---|---|
| `alert` | `configs`, `create`, `delete`, `edit` |
| `market` | `earnings`, `exhaustion` |
| `report` | `dark-pool-20x`, `dark-pool-sweeps`, `disproportionately-large`, `leveraged-etfs`, `list`, `offsetting-trades`, `phantom-trades`, `rsi-overbought`, `rsi-oversold`, `top-10-rank`, `top-100-rank`, `top-30-rank-10x-99th` |
| `trade` | `alerts`, `cluster-alerts`, `cluster-bombs`, `clusters`, `dashboard`, `level-touches`, `levels`, `list`, `sentiment` |
| `volume` | `ah-institutional`, `institutional`, `total` |
| `watchlist` | `add-ticker`, `configs`, `create`, `delete`, `edit`, `tickers` |

Top-level aliases: `trades` = `trade list`, `dashboard` = `trade dashboard`, `levels` = `trade levels`.

### Global Flags

| Flag | What It Does |
|---|---|
| `-v` / `-vv` / `-vvv` | Enable info/debug/trace diagnostics on stderr. Stdout stays parseable. |
| `--fields Field1,Field2` | Project specific fields in the output (PascalCase, exact API names). |
| `--all-fields` | Return all available fields instead of compact defaults. |
| `--strict-empty` | Exit code 7 with stderr JSON when a record-array result is empty. |

### Self-Discovery Commands

These run locally without authentication:

```bash
volumeleaders-agent doctor              # Auth and environment readiness (JSON)
volumeleaders-agent doctor --live        # + authenticated connectivity check
volumeleaders-agent schema              # Machine-readable command metadata (JSON)
volumeleaders-agent commands             # List all leaf command paths
volumeleaders-agent commands --grouped   # Grouped with descriptions
volumeleaders-agent fields trade list    # Output field metadata for --fields projection
volumeleaders-agent help agent           # Operational guidance for agent automation
volumeleaders-agent help auth            # Auth configuration details
volumeleaders-agent help exit-codes      # Exit code reference
volumeleaders-agent help examples        # Common invocation examples
volumeleaders-agent help workflows       # Workflow-oriented agent tasks
volumeleaders-agent trade list --help    # Command-specific clap help
```

### Output Contract

- Successful data commands write **compact JSON to stdout** (one JSON array or object).
- Runtime errors write one compact JSON object to **stderr**: `{"ok":false,"error":{"kind":"...","message":"..."}}`.
- Diagnostic logs from `-v`/`-vv`/`-vvv` go to stderr only.
- Exit codes: `0` success, `2` usage error, `3` auth error, `4` HTTP transport error, `5` API error, `6` JSON parse/output error, `7` strict empty result.

### Invocation Examples

```bash
volumeleaders-agent trades NVDA
volumeleaders-agent trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27
volumeleaders-agent trade list NVDA --fields FullTimeString24,Dollars | jq '.[] | select(.Dollars > 1000000)'
volumeleaders-agent dashboard NVDA
volumeleaders-agent levels NVDA
volumeleaders-agent report top-10-rank --fields FullTimeString24,DollarsMultiplier,Dollars
volumeleaders-agent volume institutional --date 2026-05-27 --tickers AAPL
volumeleaders-agent market earnings --start-date 2026-05-01 --end-date 2026-05-27
volumeleaders-agent watchlist tickers --watchlist-key 123
```

## Data Model

### What VolumeLeaders Tracks

VolumeLeaders detects **institutional block trades** on US equities. These are large trades (typically $1M+) that indicate hedge fund, pension fund, or market-maker activity. The data comes in four record types, each representing a different aggregation level:

| Record Type | What It Is | Why It Matters |
|---|---|---|
| **Trade** | A single large block print | The atomic unit. One institution moved a big position at one price. |
| **Cluster** | Multiple trades at the same price in a short window | Several institutions (or the same one) all traded at one price. Stronger signal than a single trade. |
| **Cluster Bomb** | Multiple clusters at the same price | Repeated institutional interest at a specific price across sessions. Heavy accumulation or distribution. |
| **Level** | A price where significant institutional volume has concentrated over time | The summary view. Shows where institutions have built positions historically. |

The hierarchy is: trades build into clusters, clusters build into cluster bombs, and all of them feed levels.

## Field Glossary

All field names are PascalCase and match the raw API wire format exactly. The CLI `--fields` flag uses these names for projection.

### Shared Fields

These appear across multiple record types with the same meaning:

| Field | Type | What It Means |
|---|---|---|
| `Ticker` | string | Stock ticker symbol (e.g., `AAPL`, `MSFT`) |
| `Date` | date | The trading date when the activity occurred |
| `Price` | number | The price at which the trade/cluster/level occurred |
| `Dollars` | number | Total notional value (price x shares). Raw dollar amount. |
| `Volume` | number | Total shares traded |
| `DollarsMultiplier` | number | **How many times larger this is than the stock's average institutional block trade, by dollar value.** A value of 5.0 means this trade was 5x the typical block. This is the single most useful "how big is this?" indicator. |
| `CumulativeDistribution` | number | **Percentile rank in the historical distribution (0.0-1.0).** Values near 1.0 mean historically extreme size. Near 0.5 is typical. Near 0.0 is historically small. |
| `Sector` | string | Issuer's market sector (e.g., "Technology") |
| `Industry` | string | Issuer's specific industry (e.g., "Semiconductors") |

### Trade Fields

Default compact output: `FullTimeString24`, `Volume`, `Price`, `Dollars`, `DollarsMultiplier`, `TradeRank`, `LastComparibleTradeDate`

| Field | Type | What It Means |
|---|---|---|
| `FullTimeString24` | string | Trade time in 24h format within the market session (e.g., `16:20:51`) |
| `FullDateTime` | datetime | Full timestamp (e.g., `2026-05-01T16:20:51`) |
| `TradeRank` | number | **Rank among all trades ever seen for this ticker.** Lower = rarer. Rank 1 = the single largest trade on record. |
| `LastComparibleTradeDate` | date | **Last date a trade this large was seen.** If this is months or years ago, the trade is rare. Note: the API misspells "Comparable" as "Comparible" for trades and clusters. |
| `DarkPool` | boolean | Trade executed in a dark pool (off-exchange). Institutions use dark pools to hide large orders. |
| `Sweep` | boolean | Trade was part of an intermarket sweep order. Indicates urgency -- the institution swept multiple exchanges simultaneously. |
| `OpeningTrade` | boolean | Trade occurred during the opening session |
| `ClosingTrade` | boolean | Trade occurred during the closing session |
| `SignaturePrint` | boolean | VolumeLeaders proprietary flag for trades with distinctive institutional characteristics |
| `PhantomPrint` | boolean | A trade that appeared and then had characteristics suggesting it may relate to derivative hedging |
| `LatePrint` | boolean | Trade was reported late (after normal tape time) |
| `InsideBar` | boolean | Price was contained within the prior bar's range |
| `DoubleInsideBar` | boolean | Price was contained within the prior two bars' ranges |
| `NewPosition` | boolean | Indicates the trade likely represents a new institutional position rather than adding to an existing one |
| `Bid` | number | Bid price at time of trade |
| `Ask` | number | Ask price at time of trade |
| `AverageBlockSizeDollars` | number | Historical average institutional block size in dollars for this ticker |
| `AverageBlockSizeShares` | number | Historical average institutional block size in shares for this ticker |
| `AverageDailyVolume` | number | Average daily volume for this ticker |
| `PercentDailyVolume` | number | This trade as a percentage of average daily volume |
| `RelativeSize` | number | Size relative to the stock's typical institutional activity |
| `TradeRankSnapshot` | number | Rank at the time the trade was recorded (rank can shift as new trades come in) |
| `TradeCount` | number | Number of trades at this price (usually 1 for individual trades) |
| `ClosePrice` | number | Stock's closing price on the trade date |
| `RSIHour` | number | Hourly RSI at time of trade (-1.0 if unavailable) |
| `RSIDay` | number | Daily RSI at time of trade (-1.0 if unavailable) |
| `Name` | string | Company name |
| `OffsettingTradeDate` | date | Date of a trade in the opposite direction at the same price (if any) |
| `PhantomPrintFulfillmentDate` | date | Date when a phantom print was confirmed/fulfilled |
| `PhantomPrintFulfillmentDays` | number | Days between phantom print and fulfillment |
| `FrequencyLast30TD` | number | Count of institutional trades in the last 30 trading days |
| `FrequencyLast90TD` | number | Count of institutional trades in the last 90 trading days |
| `FrequencyLast1CY` | number | Count of institutional trades in the last calendar year |
| `Cancelled` | boolean | Trade was cancelled/corrected |

**Calendar flags** (present on trades, clusters, and cluster bombs):

| Field | What It Means |
|---|---|
| `EOM` | End of month |
| `EOQ` | End of quarter |
| `EOY` | End of year |
| `OPEX` | Options expiration day |
| `VOLEX` | Volatility expiration day |

These calendar flags matter because institutional activity often spikes at these boundaries (portfolio rebalancing, options expiration hedging).

### Cluster Fields

Default compact output: `MinFullTimeString24`, `TradeCount`, `Price`, `Dollars`, `DollarsMultiplier`, `TradeClusterRank`, `LastComparibleTradeClusterDate`

| Field | Type | What It Means |
|---|---|---|
| `MinFullTimeString24` | string | **Earliest** trade time in the cluster. The cluster may span several minutes. |
| `TradeCount` | number | Number of individual trades that form this cluster. More trades = more institutional interest. |
| `TradeClusterRank` | number | Rank among all clusters for this ticker. Lower = rarer. |
| `LastComparibleTradeClusterDate` | date | Last date a cluster this large was seen. Same "Comparible" misspelling. |

Clusters share `Ticker`, `Date`, `Price`, `Dollars`, `DollarsMultiplier`, `CumulativeDistribution`, `Sector`, `Industry`, and calendar flags with trades.

### Cluster Bomb Fields

Default compact output: `MinFullTimeString24`, `TradeCount`, `Volume`, `Dollars`, `DollarsMultiplier`, `CumulativeDistribution`, `TradeClusterBombRank`, `LastComparableTradeClusterBombDate`

| Field | Type | What It Means |
|---|---|---|
| `MinFullTimeString24` | string | Earliest trade time in the cluster bomb |
| `TradeCount` | number | Total individual trades across all clusters in the bomb |
| `TradeClusterBombRank` | number | Rank among all cluster bombs for this ticker. Lower = rarer. |
| `LastComparableTradeClusterBombDate` | date | Last date a cluster bomb this large was seen. Note: cluster bombs use the **correct** "Comparable" spelling, unlike trades and clusters. |

### Level Fields

Default compact output: `Price`, `Dollars`, `Volume`, `Trades`, `RelativeSize`, `CumulativeDistribution`, `TradeLevelRank`, `Dates`

| Field | Type | What It Means |
|---|---|---|
| `Trades` | number | Total number of institutional trades at this price level |
| `RelativeSize` | number | Size of activity at this level compared to other levels for the same ticker |
| `TradeLevelRank` | number | Rank among all levels for this ticker. Lower = more significant. |
| `TradeLevelTouches` | number | Number of separate sessions where institutional activity returned to this price |
| `Dates` | string | Comma-separated dates when institutional activity occurred at this level |
| `MinDate` | date | Earliest date of activity at this level |
| `MaxDate` | date | Most recent date of activity at this level |

### Dashboard Section-Qualified Fields

The `trade dashboard` command returns all four record types in one response. Fields are prefixed with their section name: `trades.TradeRank`, `clusters.MinFullTimeString24`, `levels.TradeLevelRank`, `cluster_bombs.TradeCount`. The response includes `sections` metadata with per-section `count` and `empty` values.

### Alert Fields

Default compact output: `FullTimeString24`, `AlertType`, `TradeID`, `Price`, `Volume`, `Dollars`, `DollarsMultiplier`, `TradeRank`, `LastComparibleTradeDate`, `DarkPool`, `Sweep`

| Field | Type | What It Means |
|---|---|---|
| `AlertType` | string | Type of alert that triggered (configured by the user) |
| `TradeID` | number | Unique trade identifier for deduplication |

Alert rows are trade-shaped and share all trade fields.

### Volume Report Fields

Default compact output: `Date`, `FullDateTime`, `Ticker`, `Sector`, `Industry`, `Price`, `Dollars`, `DollarsMultiplier`, `CumulativeDistribution`, `TradeRank`, `OpeningTrade`, `ClosingTrade`, `DarkPool`, `Sweep`, `LatePrint`, `SignaturePrint`, `PhantomPrint`

Volume reports (`volume institutional`, `volume ah-institutional`, `volume total`) show trades across all tickers for a date range. Same fields as trades but always include `Ticker` since results span multiple stocks.

## Interpreting Significance

### What to Highlight

When summarizing VolumeLeaders output for a user, flag these conditions:

| Condition | Why It's Notable |
|---|---|
| `DollarsMultiplier` > 5 | Trade is 5x the stock's average institutional block. Unusually large. |
| `DollarsMultiplier` > 10 | Exceptionally large. Rare event worth calling out explicitly. |
| `TradeRank` (or variant) < 100 | Among the top 100 trades/clusters/levels ever recorded for this ticker. |
| `TradeRank` = 1 | The single largest on record. Always highlight. |
| `CumulativeDistribution` > 0.95 | In the top 5% historically by size. |
| `LastComparibleTradeDate` is months/years ago | A trade this size hasn't happened in a long time. The rarer the date, the more notable. |
| `DarkPool` = true | Institutional effort to trade without moving the visible market. |
| `Sweep` = true | Aggressive, urgent execution across exchanges. |
| `SignaturePrint` = true | VolumeLeaders identified distinctive institutional characteristics. |
| Multiple clusters or a cluster bomb at one price | Repeated institutional interest. Stronger signal than any single trade. |
| Level with many `Trades` and recent `Dates` | Active, ongoing institutional price level. Potential support/resistance. |

### What's Normal (Skip It)

- `DollarsMultiplier` between 1-3: typical institutional block
- `TradeRank` > 5000: common, not distinctive
- `CumulativeDistribution` between 0.2-0.8: middle of the historical range
- `LastComparibleTradeDate` within the last few days: similar trades happen frequently
- A single trade at a price with no clusters or levels: isolated event, less meaningful alone

### Reading a Dashboard

The dashboard (`trade dashboard`) returns four sections. A useful summary pattern:

1. Check `sections` metadata for which sections have data (`empty: false`)
2. Start with **levels** -- these are the big picture (where has institutional money concentrated?)
3. Then **cluster bombs** -- any repeated heavy activity today?
4. Then **clusters** -- groups of trades at the same price
5. Finally **trades** -- individual notable prints

For each section, lead with the top-ranked entry (lowest rank number) and its `DollarsMultiplier`.

## Known Quirks

### Spelling Inconsistency

The API misspells "Comparable" as "Comparible" in trade and cluster fields (`LastComparibleTradeDate`, `LastComparibleTradeClusterDate`) but uses the correct spelling for cluster bombs (`LastComparableTradeClusterBombDate`). This is not a bug in the CLI -- it matches the upstream API exactly.

### FlexBool Fields

Boolean fields like `DarkPool`, `Sweep`, `OpeningTrade`, etc. can appear as JSON `true`/`false` OR as `1`/`0` integers from the API. The CLI normalizes these to `true`/`false` in output. In raw API fixtures you may see either form.

### ASP.NET Date Format

Raw API dates use the `/Date(epoch_ms)/` format (e.g., `/Date(1777593600000)/`). The CLI converts these to RFC 3339 strings. Sentinel values (.NET `DateTime.MinValue`, year-1900 placeholder) are serialized as `null`.

### Compact Defaults

Without `--fields`, the CLI returns a curated subset of fields for each record type (listed above as "Default compact output"). To see all available fields, use `volumeleaders-agent fields <command>`. To project specific fields, pass `--fields Field1,Field2`.

### RSI Values

`RSIHour` and `RSIDay` use `-1.0` as a sentinel for "not available." Do not interpret -1.0 as an actual RSI reading.

### Rank Semantics

All rank fields (TradeRank, TradeClusterRank, TradeClusterBombRank, TradeLevelRank) use **lower = more significant**. Rank 1 is the most extreme on record. This is the opposite of how some ranking systems work (where #1 is best/highest), but here #1 means "largest/rarest ever."

### CumulativeDistribution Scale

`CumulativeDistribution` is a decimal between 0.0 and 1.0, not 0-100. A value of `0.9918` means the 99.18th percentile.

## Common Agent Patterns

### "What happened with AAPL today?"

```bash
volumeleaders-agent trade dashboard AAPL
```

Parse the response, check `sections` for non-empty sections, and summarize the top-ranked entries from each section. Lead with levels and cluster bombs if present -- those are the strongest signals.

### "Is there unusual activity?"

Look for:
- `DollarsMultiplier` > 5 on any record type
- `TradeRank` (or variant) in the low hundreds or less
- `LastComparibleTradeDate` that's weeks or months old
- Cluster bombs (their existence alone signals repeated institutional interest)

### "What are the key price levels?"

```bash
volumeleaders-agent trade levels AAPL
```

Sort by `TradeLevelRank` (ascending). The top-ranked levels are where institutions have concentrated the most capital. Cross-reference `Dates` to see if the level is recent or historical.

### "Show me the biggest trades"

```bash
volumeleaders-agent trade list AAPL --fields FullTimeString24,Price,Dollars,DollarsMultiplier,TradeRank,DarkPool,Sweep,SignaturePrint
```

Sort by `TradeRank` (ascending) or `DollarsMultiplier` (descending). Flag any with `DarkPool`, `Sweep`, or `SignaturePrint` = true.

### "Summarize for a non-technical user"

Translate the data:
- "DollarsMultiplier: 17.0" becomes "This trade was 17 times larger than the average institutional trade in this stock"
- "TradeRank: 42" becomes "This is the 42nd largest institutional trade ever recorded for this stock"
- "LastComparibleTradeDate: 2025-01-15" becomes "The last time a trade this large happened was January 2025 -- over a year ago"
- "DarkPool: true" becomes "This trade was executed off-exchange to avoid impacting the visible market"
