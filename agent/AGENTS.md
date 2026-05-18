# AGENT CRATE NOTES

## OVERVIEW

`volumeleaders-agent` is the CLI boundary. It parses clap commands, bootstraps browser auth, translates arguments into `volumeleaders-client` requests, and writes results to stdout (TSV by default, JSON with `--json` or `--pretty`).

## DOC FRESHNESS

- Update this file and root `README.md` in the same change that modifies command names, arguments, default output fields, report presets, auth messages, exit behavior, or supported output formats.
- Keep client wire-contract details in `client/AGENTS.md`; this file only documents CLI behavior and command plumbing.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Binary entry | `src/main.rs` | Thin wrapper around `volumeleaders_agent::run()` |
| Dispatch | `src/lib.rs` | Parses CLI and routes to command handlers |
| Command tree | `src/cli.rs` | Top-level clap groups and global `--json`/`--pretty` |
| Command handlers | `src/commands/*.rs` | `handle(args, format) -> i32` per group |
| Shared CLI args/types | `src/common/` | Dates, tickers, order direction, summary groups, tri-state filters |
| Browser auth bridge | `src/common/auth.rs` | Builds client sessions from browser cookies |
| Output formatting | `src/output.rs` | TSV/JSON output, `OutputFormat` enum, field selection, validation |
| Trade output transforms | `src/common/trade_transforms.rs` | Shared semantic cleanup for trade-shaped rows before field selection |

## COMMAND SURFACE

- Top-level groups: `report`, `trade`, `volume`, `market`, `alert`, `watchlist`, `completions`.
- Default output is tab-separated values (TSV). `--json` switches to compact JSON, `--pretty` to indented JSON.
- Errors and logs go to stderr. Data goes to stdout.
- Auth failure text tells users to log in at `https://www.volumeleaders.com` and retry.
- Trade-shaped output intentionally omits the upstream `PercentDailyVolume` value because live report payloads return it as `0.0` for current and prior trading days.
- Compact defaults also omit `TradeConditions`, `RelativeSize`, `Name`, and `Volume` on trade-shaped rows. Keep `RelativeSize` available via `--fields` or `--all-fields`, and keep level-centric RelativeSize behavior intact.

## HOTSPOTS

- `src/commands/trade.rs` is the largest risk: CLI dispatch, request construction, dashboard projection, summary math, sentiment classification, trade-level validation, and preset-like constants live together.
- `src/commands/report.rs` owns report preset definitions and summary aggregation. Tests pin preset count, names, and filter contents.
- `src/commands/alert.rs` and `src/commands/watchlist.rs` are mapping-heavy create/edit flows. Keep CLI field names aligned with client request structs.
- `src/output.rs` validates requested fields against serialized record keys and controls compact vs all-fields output.
- `src/common/trade_transforms.rs` owns reusable trade-shaped row transforms. Use it for commands that emit trades, trade clusters, trade levels, cluster bombs, or institutional volume rows.

## TESTS

- Tests live inline in source modules.
- `src/cli.rs` uses `Cli::command().debug_assert()` to validate the clap tree.
- Command modules test defaults, parsing, output shaping, presets, summaries, and field selections.
- There is no `agent/tests/` fixture tree today.

## COMMANDS

```bash
cargo test -p volumeleaders-agent
cargo run -p volumeleaders-agent -- --help
cargo run -p volumeleaders-agent -- completions bash
```

## ANTI-PATTERNS

- Do not print diagnostics to stdout. stdout is machine-readable command output.
- Do not change compact default fields without updating tests and README examples.
- Do not add a command without wiring `cli.rs`, `commands/mod.rs`, `lib.rs`, tests, and README command notes.
- Do not duplicate client request-building rules here if they belong in `volumeleaders-client`.
