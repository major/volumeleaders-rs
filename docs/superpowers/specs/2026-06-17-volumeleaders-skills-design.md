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

| Concept | Draft interpretation (to be verified) |
|---|---|
| `TradeRank` | Ranks trades by institutional significance for that ticker on that day. Rank 1 = the single largest/most significant institutional trade. Lower numbers = higher significance. |
| `Dollars` | Total dollar volume of the trade. Larger values indicate more significant institutional capital deployment. |
| `clusters` | Groups of trades at similar price levels over a time window, indicating institutional accumulation (buying cluster) or distribution (selling cluster) at a price zone. |
| `cluster_bombs` | An unusually large or highly concentrated cluster — a strong signal of aggressive institutional positioning at a level. |
| `levels` | Key price levels where institutions have historically traded heavily. These tend to act as support or resistance on revisit. |
| `level_touches` | Instances where price has revisited an institutional level. High touch count = level is being respected by the market. |
| `vcd` (volume concentration delta) | Score measuring how concentrated volume is at a price level relative to surrounding levels. Higher VCD = more institutional conviction at that price. |

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
