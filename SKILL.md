# VolumeLeaders CLI Skill

Use this skill when an agent needs to operate `volumeleaders-agent`, discover its command surface, diagnose auth, or modify the CLI implementation. The CLI authenticates with an XDG cached session first, then `VL_USERNAME` and `VL_PASSWORD`, then optional XDG config credentials.

## Self-discovery

Run these commands before guessing command shapes:

```bash
volumeleaders-agent doctor
volumeleaders-agent doctor --live
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
volumeleaders-agent help workflows
volumeleaders-agent trade list --help
```

- `doctor` is local-only by default and reports credential-based readiness as compact JSON. If auth is missing or invalid, parse `auth.actions` and apply each recovery step. Use `doctor --live` for an authenticated connectivity check that tries the cached session first, then falls back to a live login if credentials are configured.
- `schema` is the authoritative machine-readable command contract generated from the live clap tree. Command entries include `path`, `preferred_path`, `is_alias`, optional `alias_for`, `aliases`, auth requirements, mutating and dry-run safety metadata, help text, argument metadata with stable `name` identifiers and semantic types, known `possible_values` validation constraints, boolean flag versus value-taking option shape, and structured `examples` arrays parsed from command help.
- `commands` is the lightweight plain-text leaf command list. Use `--grouped` for descriptions.
- `fields <command path>` emits exact case-sensitive raw API output field names, descriptions, and type hints for commands that support `--fields` without requiring live rows. `fields trade dashboard` uses section-qualified nested names such as `trades.TradeRank`, `clusters.MinFullTimeString24`, `levels.TradeLevelRank`, and `cluster_bombs.TradeCount`. Unknown projected fields fail with exit code `2` and structured `usage_error` JSON on stderr.
- Trade-shaped outputs use raw VolumeLeaders field names. Omitted `--fields` returns compact defaults: trades/reports (`FullTimeString24`, `Volume`, `Price`, `Dollars`, `DollarsMultiplier`, `TradeRank`, `LastComparibleTradeDate`), clusters (`MinFullTimeString24`, `TradeCount`, `Price`, `Dollars`, `DollarsMultiplier`, `TradeClusterRank`, `LastComparibleTradeClusterDate`), cluster bombs (`MinFullTimeString24`, `TradeCount`, `Volume`, `Dollars`, `DollarsMultiplier`, `CumulativeDistribution`, `TradeClusterBombRank`, `LastComparableTradeClusterBombDate`), and levels (`Price`, `Dollars`, `Volume`, `Trades`, `RelativeSize`, `CumulativeDistribution`, `TradeLevelRank`, `Dates`).
- `help <topic>` gives operational guidance when README access is unavailable. Use `help workflows` for common agent workflows with recommended first commands and copy-paste examples.
- `help agent` summarizes the recommended non-interactive automation flow.
- Command-specific `--help` includes descriptions for public options and an `Examples:` section for every visible leaf command; schema exposes the same examples for machine-readable access.

## Invocation contract

- Successful data commands write compact JSON to stdout.
- `trade dashboard` includes `sections.<name>.count` and `sections.<name>.empty` metadata for `trades`, `clusters`, `levels`, and `cluster_bombs` so empty sibling sections are explicit.
- Plain discovery/help outputs (`commands`, `help`, shell `completions`, clap `--help`) write plain text to stdout.
- Runtime errors write one compact JSON object to stderr: `{"ok":false,"error":{"kind":"...","message":"..."}}`.
- Diagnostic logs from `-v`, `-vv`, and `-vvv` go to stderr only. stdout must remain parseable command output.
- Exit codes: `0` success, `2` usage error, `3` auth error, `4` HTTP transport error, `5` API error, `6` JSON parse or output transformation error, `7` strict empty result.
- Commands that need live data require a valid cached session, `VL_USERNAME` and `VL_PASSWORD`, or `~/.config/volumeleaders-agent/config.json`. Local discovery commands (`doctor`, `schema`, `commands`, `fields`, `help`, `completions`) do not require live API access. `doctor --live` is the explicit exception for checking authenticated connectivity.

## Auth model

The CLI authenticates with VolumeLeaders using this source order:

1. Cached session at `~/.cache/volumeleaders-agent/cookies.json`, if it exists and XSRF refresh succeeds.
2. Non-empty `VL_USERNAME` and `VL_PASSWORD` environment variables.
3. Config file at `~/.config/volumeleaders-agent/config.json` containing `{"username":"YOUR_EMAIL","password":"YOUR_PASSWORD"}`.
4. Structured auth error with exit code `3`.

Environment variables take precedence over the config file. If either `VL_USERNAME` or `VL_PASSWORD` is set, both must be set and non-empty; partial or empty env credentials are an auth error and the config file is not used. On credential login, the CLI POSTs to `/Login/Login`, extracts session cookies and the XSRF token, and caches the session. Cookie values, passwords, and XSRF tokens must never be printed or logged.

Use this recovery flow:

```bash
volumeleaders-agent doctor
volumeleaders-agent doctor --live
volumeleaders-agent -vv doctor
volumeleaders-agent help auth
```

Common failures:

- Missing credentials: set both `VL_USERNAME` and `VL_PASSWORD`, or create `~/.config/volumeleaders-agent/config.json`, then rerun `doctor`.
- Invalid credentials: check the username and password, then retry.
- Expired session: the cached session is automatically cleared; the next live request will re-authenticate.
- Cache unavailable: the XDG cache directory may not be available; check filesystem permissions.
- Ambiguous setup: do not set only one auth environment variable. Env vars override config fallback.

## Global flags and reusable argument shapes

Global flags:

| Flag | Contract |
|------|----------|
| `--fields a,b,c` | Select exact, case-sensitive compact JSON object fields for record-array outputs. Discover names with `fields <command path>`. |
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
| `FieldsSelectable` | `--fields Ticker,Date,Price` or `--all-fields` | Discover exact case-sensitive field names with `fields <command path>` before filtering. Pipe stdout to external `jq` for jq-style filtering or object construction after built-in projection. |
| `ConstrainedInteger` | Example: `trade levels --trade-level-count 10` | Check schema `possible_values` before choosing values. `trade-level-count` accepts only `5`, `10`, `20`, or `50` and invalid values fail with structured `usage_error` JSON. |
| `BooleanFilter` | Bare flags such as `--sweep`, or value-taking booleans such as `--normal-prints false` where help shows a value | Read command help or schema `kind` before passing `true` or `false`; bare flags do not take values. |
| `DryRunMutation` | Mutating alert/watchlist create, edit, delete, and add-ticker commands accept `--dry-run` | Inspect the JSON plan before live mutation; delete commands require `--yes` when not using `--dry-run`. |

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
| `doctor` | Check local auth and environment readiness as JSON; add `--live` for authenticated connectivity. |
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
volumeleaders-agent schema | jq '.commands[] | select(.mutating == true) | {path, supports_dry_run, requires_confirmation}'
volumeleaders-agent trade list NVDA --fields FullTimeString24,Dollars | jq '.[] | select(.Dollars > 1000000)'
volumeleaders-agent alert create --name BigTechSweeps --tickers AAPL,MSFT --dry-run
volumeleaders-agent help examples
```

Auth recovery:

```bash
volumeleaders-agent doctor
volumeleaders-agent doctor --live
volumeleaders-agent -vv doctor
volumeleaders-agent help auth
```

Common data workflows:

```bash
volumeleaders-agent trades NVDA
volumeleaders-agent trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields FullTimeString24,Volume,Price,Dollars
volumeleaders-agent trade list NVDA --fields FullTimeString24,Dollars | jq '.[] | select(.Dollars > 1000000)'
volumeleaders-agent report top-10-rank --fields FullTimeString24,DollarsMultiplier,Dollars
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

Dependency and workflow maintenance notes:

- Security audit CI uses `actions-rust-lang/audit`; keep `make audit` for local checks.
- The cargo-dist release workflow uses OIDC trusted publishing through a release-tagged `rust-lang/crates-io-auth-action` pin.
