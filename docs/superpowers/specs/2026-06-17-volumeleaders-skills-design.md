# Design: VolumeLeaders Agent Skills for AI Tools

**Date:** 2026-06-17  
**Status:** Approved for implementation  
**Scope:** Two installable skills to help AI tools (opencode, pi, etc.) use `volumeleaders-agent` effectively

---

## Problem

AI tools consuming `volumeleaders-agent` struggle with:

1. **Command selection** — given a user's trading question, which command(s) produce the right data?
2. **Output interpretation** — what do fields like `TradeRank`, `clusters`, `cluster_bombs`, and `levels` mean in trading terms?

The existing `SKILL.md` in the repo root is a developer/operator skill. It is not appropriate for end-user AI assistant contexts: it covers implementation internals, drift tests, and dev workflow commands that are irrelevant noise for a trading-focused LLM.

---

## Solution

Two lightweight, installable skills published as an npm package from this repo.

---

## Repository Layout

```
skills/
  volumeleaders/
    SKILL.md              # command-selection + workflow skill
  volumeleaders-interpret/
    SKILL.md              # domain interpretation glossary
  package.json            # npm package: `npx skills add volumeleaders-agent-skills`
```

The `skills/` directory lives at the root of `volumeleaders-rs`. No separate repo.

---

## Skill 1: `volumeleaders`

**Description trigger:** Use when an AI assistant needs to query institutional trade data via `volumeleaders-agent` — covers auth, command selection, and output contract.

**Token budget:** ~150–200 words (always loaded, must be slim)

**Sections:**

### Auth
One line: run `doctor` first if auth state is unknown; `doctor --live` for connectivity check.

### Question → Command Table
Core value of this skill. Maps natural-language trading intents to commands:

| What the user wants | Command |
|---|---|
| Recent institutional trades for a ticker | `trades NVDA` |
| Full dashboard (trades + clusters + levels + cluster bombs) | `dashboard NVDA` |
| Key price levels for a ticker | `levels NVDA` |
| Top institutional trades across the whole market | `report top-10-rank` or `report top-100-rank` |
| Specific report preset (dark pool, RSI, etc.) | `report list` to discover, then `report <name> --days N` |
| Institutional volume for a ticker on a date | `volume institutional --date 2026-06-17 --tickers NVDA` |
| After-hours institutional volume | `volume ah-institutional --date 2026-06-17 --tickers NVDA` |
| Earnings calendar | `market earnings --start-date YYYY-MM-DD --end-date YYYY-MM-DD` |
| Market exhaustion signals | `market exhaustion` |
| Trades that touched a known level | `trade level-touches NVDA` |

**Omissions:** `trade sentiment` is intentionally excluded — semantics are unclear without domain context.

### Output Contract
- Successful commands: compact JSON to stdout
- Errors: `{"ok":false,"error":{"kind":"...","message":"..."}}` to stderr
- Exit codes: `0` success, `2` usage, `3` auth, `4` HTTP, `5` API, `6` parse, `7` strict-empty

### Field Projection
Run `fields <command path>` to discover `--fields` names before filtering. Pipe to `jq` for reshaping.

### Self-Discovery Hook
When uncertain about a command: `commands --grouped` or `schema`.  
When results are in hand and need interpretation: load `volumeleaders-interpret`.

---

## Skill 2: `volumeleaders-interpret`

**Description trigger:** Use when an AI has `volumeleaders-agent` output in hand and needs to interpret what it means for a trading decision.

**Token budget:** ~350–450 words (on-demand, more content acceptable)

**Approach:** Draft content authored by agent, reviewed and corrected by project owner before implementation.

**Sections:**

### Core Field Concepts

| Concept | Status | Interpretation |
|---|---|---|
| `TradeRank` | ✅ confirmed | Current rank of the trade — shifts downward as larger trades arrive. Lower = higher current significance. Use for live/recent context. |
| `TradeRankSnapshot` | ✅ confirmed | Immutable rank at the moment the trade hit. Never changes. Use for historical comparisons. |
| `Dollars` | ✅ confirmed | Total dollar volume of the trade. Larger = more significant institutional capital deployment. |
| `CumulativeDistribution` | ✅ confirmed from data | 0–1 percentile rank. 1.0 = 100th percentile = most significant in the dataset. |
| `DollarsMultiplier` | ✅ confirmed from data | How many times larger this trade is vs. the average trade size for that security (e.g. 8.04 = 8.04× average). This is what the website labels as "relative size". Use this field for trade-level relative size. |
| `RelativeSize` | ✅ confirmed from data | Same concept as `DollarsMultiplier` but not populated in trade API responses. Use for levels (e.g. NVDA $211 level = 15.67× average). On trades, read `DollarsMultiplier` instead. |
| `FrequencyLast30TD` / `FrequencyLast90TD` / `FrequencyLast1CY` | ✅ confirmed from data | Count of institutional trades for that ticker in the last 30 trading days / 90 trading days / 1 calendar year. Values of 1/1/1 = rare or first-ever institutional activity for that ticker. |
| `InsideBar` | ✅ confirmed from data | Trade occurred on a day when the price bar was fully inside the prior day's high/low range — a consolidation/compression signal. |
| `DoubleInsideBar` | ✅ confirmed from data | Two consecutive inside bar days — stronger consolidation signal than a single inside bar. |
| `PhantomPrint` | ✅ confirmed from data | Trade appeared on the tape but has not yet been fulfilled. `PhantomPrintFulfillmentDate` is null until fulfilled; `TradeRankSnapshot` is 0 on unfulfilled phantom prints. |
| `OffsettingTradeDate` | ✅ confirmed from data | Date of a paired offsetting trade. Presence indicates this trade is part of a matched pair (hedging, rolling, or unwinding). |
| `EOM` | ✅ confirmed | Boolean flag: trade occurred at end of month. Only appears in output when relevant. |
| `EOQ` | ✅ confirmed | Boolean flag: trade occurred at end of quarter. Only appears in output when relevant. |
| `EOY` | ✅ inferred | Boolean flag: trade occurred at end of year. Only appears in output when relevant. |
| `OPEX` | ✅ confirmed | Boolean flag: trade occurred on options expiration date. Only appears in output when relevant. |
| `VOLEX` | ✅ confirmed | Boolean flag: trade occurred on VIX/volatility expiration date. Only appears in output when relevant. |
| `venue` | ✅ observed in data | Report-layer field (not in base trade model): `"dark_pool"` or `"dark_pool_sweep"`. Indicates which report type surfaced this trade. |
| `clusters` | ✅ confirmed | Groups of trades at similar price levels over a time window, indicating institutional accumulation or distribution at a price zone. |
| `cluster_bombs` | ✅ confirmed | An unusually large or highly concentrated cluster — a strong signal of aggressive institutional positioning at a level. |
| `levels` | ✅ confirmed | Key price levels where institutions have historically traded heavily. Tend to act as support or resistance on revisit. |
| `level_touches` | ✅ confirmed | Instances where price has revisited an institutional level. High touch count = level is being respected by the market. |
| `vcd` (volume concentration delta) | ⚠️ needs verification | Score measuring how concentrated volume is at a price level. Higher VCD = more institutional conviction at that price? |
| `ExhaustionScoreRank` (+ 30/90/365-day variants) | ❓ needs input | NVDA today: rank 337 overall, 16 for 30-day, 38 for 90-day, 90 for 365-day. Is rank 1 = most exhausted? What does exhaustion measure? |
| `SignaturePrint` | ❓ needs input | No examples found in live data. What makes a print "signature"? |
| `TD30` / `TD90` / `TD1CY` | ❓ needs input | Always null in responses. Reference dates 30/90 trading days and 1 calendar year ago? |
| `NewPosition` | ❓ needs input | True on new ETFs and on high-frequency tickers. Institution opening a brand-new position vs. adding to existing? |
| `PercentDailyVolume` | ❓ needs input | Always 0.0 in every response seen. Data availability issue or not currently populated? |

### Report Signal Meanings

| Report | Draft signal meaning (to be verified) |
|---|---|
| `dark-pool-sweeps` | Large off-exchange (dark pool) trades executed aggressively — typically institutional urgency to accumulate or distribute without moving the public market. |
| `dark-pool-20x` | Dark pool trades 20× the normal size for that ticker — extreme institutional positioning, often precedes significant price moves. |
| `disproportionately-large` | Trades anomalously large relative to that ticker's normal institutional activity — unusual conviction signal. |
| `top-10-rank` / `top-100-rank` | The highest-ranked institutional trades across all tickers for the period — useful for finding where the biggest money is moving. |
| `top-30-rank-10x-99th` | Top 30 ranked trades that are also 10× size and in the 99th percentile — highest-conviction institutional activity. |
| `phantom-trades` | Trades that appear briefly and disappear from the tape — may indicate wash activity or data anomalies; treat with caution. |
| `offsetting-trades` | Paired buy/sell institutional trades at similar sizes — typically hedging, unwinding, or rolling a position. |
| `rsi-overbought` / `rsi-oversold` | Institutional trades in tickers at RSI extremes — useful for spotting contra-trend institutional positioning. |
| `leveraged-etfs` | Institutional trades in leveraged ETFs — signals directional macro conviction with amplified exposure. |

### Synthesis Guidance

Draft guidance for LLMs combining signals:
- A cluster + cluster_bomb on the same ticker on the same day = strong institutional interest at that level; note direction (buy vs sell side).
- Levels with multiple recent touches = price is actively respecting an institutional zone; level-touches count is a proxy for zone strength.
- Dark pool sweep + top-rank appearance = institutional urgency; unusual in combination.
- Disproportionately large + high Dollars = size and relative significance both elevated; treat as high-conviction.
- Offsetting trades reduce net signal — when present alongside bullish indicators, temper the read.

### Caveats
- These signals describe institutional positioning, not guaranteed price direction.
- Dark pool trades are reported with a delay; timing context matters.
- `TradeRank` is relative to the ticker's own history — rank 1 for a low-volume stock differs from rank 1 for NVDA.
- Phantom and offsetting trades should generally be filtered out of bullish/bearish reads.

---

## npm Packaging

`skills/package.json` declares:
- `name`: `volumeleaders-agent-skills`
- `version`: mirrors the crate version (updated by release-plz in the same PR)
- `files`: `["volumeleaders/", "volumeleaders-interpret/"]`
- No runtime dependencies

Users install with: `npx skills add volumeleaders-agent-skills`

The `release-plz.toml` or release workflow bumps the npm version in lockstep with the crate version.

---

## AGENTS.md Update

Add to the "DOC FRESHNESS" section:

> When changing command paths, output fields, argument shapes, or data interpretation behavior, update the relevant skill in `skills/` in the same PR. Stale user-facing skills are treated the same as stale docs.

---

## Success Criteria

- [ ] `npx skills add volumeleaders-agent-skills` installs both skills
- [ ] An LLM with only `volumeleaders` loaded can correctly select commands for the 10 common intents in the table
- [ ] An LLM with `volumeleaders-interpret` loaded can produce a correct plain-English summary from `dashboard` output
- [ ] Both skills fit within their token budgets
- [ ] Version in `skills/package.json` stays in sync with crate version through the release pipeline
- [ ] AGENTS.md rule is present and enforced in PR review
