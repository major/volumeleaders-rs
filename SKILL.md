# VolumeLeaders CLI Skill

Use this skill when an agent needs to operate `volumeleaders-agent`, discover its command surface, diagnose auth, or modify the CLI implementation. The CLI reads an authenticated browser session for live VolumeLeaders data and is optimized for machine-readable automation.

## Self-discovery

Run these commands before guessing command shapes:

```bash
volumeleaders-agent doctor
volumeleaders-agent schema
volumeleaders-agent commands
volumeleaders-agent commands --grouped
volumeleaders-agent fields trade list
volumeleaders-agent help agent
volumeleaders-agent help auth
volumeleaders-agent help environment
volumeleaders-agent help exit-codes
volumeleaders-agent help schema
volumeleaders-agent help examples
volumeleaders-agent trade list --help
```

- `doctor` is local-only by default and reports browser-cookie readiness as compact JSON.
- `schema` is the authoritative machine-readable command contract generated from the live clap tree. Command entries include `path`, `preferred_path`, `is_alias`, optional `alias_for`, `aliases`, auth requirements, help text, argument metadata with stable `name` identifiers and semantic types, boolean flag versus value-taking option shape, and structured `examples` arrays.
- `commands` is the lightweight plain-text leaf command list. Use `--grouped` for descriptions.
- `fields <command path>` emits exact case-sensitive output field names, descriptions, and type hints for commands that support `--fields` without requiring live rows.
- `help <topic>` gives operational guidance when README access is unavailable.
- `help agent` summarizes the recommended non-interactive automation flow.
- Command-specific `--help` includes descriptions for public options and an `Examples:` section for every visible leaf command; schema also exposes structured examples for machine-readable access.

## Invocation contract

- Successful data commands write compact JSON to stdout.
- Plain discovery/help outputs (`commands`, `help`, shell `completions`, clap `--help`) write plain text to stdout.
- Runtime errors write one compact JSON object to stderr: `{"ok":false,"error":{"kind":"...","message":"..."}}`.
- Diagnostic logs from `-v`, `-vv`, and `-vvv` go to stderr only. stdout must remain parseable command output.
- Exit codes: `0` success, `2` usage error, `3` auth error, `4` HTTP transport error, `5` API error, `6` JSON parse or output transformation error, `7` strict empty result.
- Commands that need live data require browser cookies. Local discovery commands (`doctor`, `schema`, `commands`, `fields`, `help`, `completions`) do not require live API access.

## Auth model

The CLI extracts browser cookies for `volumeleaders.com` from local Chrome first, then Firefox. Required session material includes the ASP.NET session cookie, forms auth cookie, and request verification token. Cookie values and XSRF tokens must never be printed or logged.

Use this recovery flow:

```bash
volumeleaders-agent doctor
volumeleaders-agent -vv doctor
volumeleaders-agent help auth
```

Common failures:

- No browser login: log in at `https://www.volumeleaders.com`, then rerun `doctor`.
- Expired cookies: refresh the browser session and retry.
- Missing XSRF token: reload the site in the browser, then retry.
- Browser profile mismatch: run the CLI as the same OS user that owns the logged-in browser profile.

## Global flags and reusable argument shapes

Global flags:

| Flag | Contract |
|------|----------|
| `--fields a,b,c` | Select compact JSON object fields for record-array outputs. |
| `--all-fields` | Emit full available fields for record-array outputs. |
| `--strict-empty` | Return exit `7` with `empty_result` stderr JSON when a record-array output would be empty. |
| `-v`, `-vv`, `-vvv` | Enable info, debug, or trace diagnostics on stderr. |

Reusable shapes:

| Shape | Common flags or args | Notes |
|-------|----------------------|-------|
| `TickerOnly` | Positional ticker such as `NVDA`, or `--ticker` where command help says so | Prefer reading `schema` before choosing positional vs flag form. |
| `DateRange` | `--start-date YYYY-MM-DD --end-date YYYY-MM-DD` | Used by many trade, market, and report commands. |
| `TickerDateRange` | Ticker plus `--start-date` and `--end-date` | Common for trade list, dashboard, clusters, levels, and report filters. |
| `Paged` | `--start N --length N`, or command-specific `--limit N` | DataTables-style commands use `start` and `length`; report and volume commands often use `limit`. |
| `FieldsSelectable` | `--fields Ticker,Date,Price` or `--all-fields` | Discover exact field names with `fields <command path>` before filtering. |
| `BooleanFilter` | Bare flags such as `--sweep`, or value-taking booleans such as `--normal-prints false` where help shows a value | Read command help or schema `kind` before passing `true` or `false`; bare flags do not take values. |

## Command catalog

Aliases:

| Alias | Canonical path |
|-------|----------------|
| `trades` | `trade list` |
| `dashboard` | `trade dashboard` |
| `levels` | `trade levels` |

Local discovery and setup:

| Path | Description |
|------|-------------|
| `commands` | List available leaf command paths. |
| `doctor` | Check local auth and environment readiness as JSON. |
| `fields` | Show output field metadata for commands that support `--fields`. |
| `help` | Show built-in operational help topics. |
| `schema` | Emit machine-readable command metadata as JSON. |
| `completions` | Generate shell completions. |

Live data and configuration:

| Group | Leaf commands |
|-------|---------------|
| `alert` | `configs`, `create`, `delete`, `edit` |
| `market` | `earnings`, `exhaustion` |
| `report` | `dark-pool-20x`, `dark-pool-sweeps`, `disproportionately-large`, `leveraged-etfs`, `list`, `offsetting-trades`, `phantom-trades`, `rsi-overbought`, `rsi-oversold`, `top-10-rank`, `top-100-rank`, `top-30-rank-10x-99th` |
| `trade` | `alerts`, `cluster-alerts`, `cluster-bombs`, `clusters`, `dashboard`, `level-touches`, `levels`, `list`, `sentiment` |
| `volume` | `ah-institutional`, `institutional`, `total` |
| `watchlist` | `add-ticker`, `configs`, `create`, `delete`, `edit`, `tickers` |

## Examples and recovery flows

Discovery:

```bash
volumeleaders-agent commands --grouped
volumeleaders-agent fields trade list
volumeleaders-agent fields volume institutional | jq '.fields[].name'
volumeleaders-agent help agent
volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == "trade list")'
volumeleaders-agent help examples
```

Auth recovery:

```bash
volumeleaders-agent doctor
volumeleaders-agent -vv doctor
volumeleaders-agent help auth
```

Common data workflows:

```bash
volumeleaders-agent trades NVDA
volumeleaders-agent trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields Ticker,DateTime,Price,Dollars
volumeleaders-agent dashboard NVDA
volumeleaders-agent levels NVDA
volumeleaders-agent volume institutional --date 2026-05-27 --tickers AAPL
volumeleaders-agent market earnings --start-date 2026-05-01 --end-date 2026-05-27
volumeleaders-agent watchlist tickers --watchlist-key 123
```

Automation-safe empty handling:

```bash
volumeleaders-agent --strict-empty trades NVDA
volumeleaders-agent --strict-empty report dark-pool-sweeps --days 30
```

## Development

CLI code lives under `src/cli/`:

| Area | File |
|------|------|
| Clap parser and aliases | `src/cli/args.rs` |
| Dispatch entry point | `src/cli/mod.rs` |
| Runtime error JSON and exit codes | `src/cli/error.rs` |
| Output formatting and strict-empty handling | `src/cli/output.rs` |
| Machine-readable schema | `src/cli/schema.rs` |
| Plain command discovery | `src/cli/command_list.rs` |
| Local auth diagnostics | `src/cli/doctor.rs` |
| Operational help topics | `src/cli/help.rs` |
| Stderr tracing setup | `src/cli/logging.rs` |
| Live command handlers | `src/cli/commands/` |

Run these before opening CLI PRs:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --lib --no-default-features
cargo clippy --lib --no-default-features -- -D warnings
cargo doc --all-features --no-deps
cargo doc --no-default-features --no-deps
DIFF_COVER='uvx diff-cover' make patch-coverage
```

When changing public CLI behavior, update `README.md`, `AGENTS.md`, this `SKILL.md`, schema or command-list tests, and any relevant integration tests in `tests/cli_*.rs`.
