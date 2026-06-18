# volumeleaders-rs

> **Disclaimer:** This project is unofficial and is not affiliated with, endorsed by, or sponsored by [volumeleaders.com](https://www.volumeleaders.com).

Rust crate for working with VolumeLeaders data from an authenticated session. The package is published as `rusty-volumeleaders` and includes both a reusable API client library and the `volumeleaders-agent` CLI. Authentication uses an XDG cached session first, then `VL_USERNAME` and `VL_PASSWORD`, then an optional XDG config file.

## Install

Install the CLI from crates.io with Cargo after the first `rusty-volumeleaders` release is published:

```bash
cargo install rusty-volumeleaders --locked
```

GitHub releases also provide cargo-dist archives and shell or PowerShell installers for supported platforms.

## Repository layout

```text
.
├── src/                      # API client library modules
├── src/cli/                  # CLI parser, commands, output, and shared helpers
├── tests/fixtures/           # Server payload fixtures
├── examples/                 # Standalone client examples
├── .github/workflows/        # CI, audit, release-plz, cargo-dist releases
├── .github/instructions/     # GitHub Copilot review guidance for release automation
├── AGENTS.md                 # Project knowledge base for coding agents
├── skills/volumeleaders/     # Installable agent skill (npx skills add)
├── codecov.yml               # Codecov project and patch coverage gates
├── dist-workspace.toml       # cargo-dist release artifact configuration
├── Makefile                  # Local development commands
├── cliff.toml                # Conventional-commit changelog grouping
├── LICENSE                   # Apache-2.0 license
└── release-plz.toml          # Release PR and tag automation
```

## Requirements

- Rust 1.96.0 or newer
- `rust-toolchain.toml` pins local builds to the CI MSRV by default
- Configure credentials for commands that need live authenticated data with either `VL_USERNAME` and `VL_PASSWORD` or `~/.config/volumeleaders-agent/config.json`
- Optional tools for local maintenance: `cargo llvm-cov`, `cargo audit`
- Optional tool for local patch coverage checks: `diff-cover` or `uvx diff-cover`

## CLI usage

The CLI authenticates with VolumeLeaders by trying sources in this order: a valid cached session at `~/.cache/volumeleaders-agent/cookies.json`, non-empty `VL_USERNAME` and `VL_PASSWORD` environment variables, then `~/.config/volumeleaders-agent/config.json` with `{"username":"YOUR_EMAIL","password":"YOUR_PASSWORD"}`. Environment variables override the config file. If either auth environment variable is set, both must be set and non-empty, and config fallback is skipped. The config file contains plaintext credentials, so keep it readable only by your user, for example with `chmod 600 ~/.config/volumeleaders-agent/config.json`. Command output goes to stdout as compact JSON by default. Use command-specific `--fields` for built-in projection and pipe through external `jq` for filtering, reshaping, sorting, or pretty-printing. Runtime errors are written to stderr as one compact JSON line such as `{"ok":false,"error":{"kind":"auth_error","message":"set VL_USERNAME and VL_PASSWORD environment variables or create ~/.config/volumeleaders-agent/config.json with username and password fields"}}`.

Semantic exit codes are stable for automation: `0` means success, `2` is clap usage or argument validation, `3` is auth failure, `4` is HTTP transport failure, `5` is a VolumeLeaders API error response, `6` is JSON parsing or output transformation failure, and `7` is strict empty-result handling.

Use global `--strict-empty` when an empty record array should fail automation instead of returning `[]` on stdout. Data commands that would emit an empty array return exit code `7` and write a structured `empty_result` error to stderr with command-specific recovery guidance. Object outputs, discovery commands, help topics, completions, and local diagnostics are not treated as empty arrays.

Use global `-v`, `-vv`, or `-vvv` to enable info, debug, or trace diagnostics on stderr. Without `-v`, the CLI logs warnings and errors only. stdout remains reserved for command output so JSON pipelines stay clean, and sensitive cookie material is never logged.

Use `schema` for machine-readable CLI discovery. It emits compact JSON generated from the live clap command tree with the binary version, credential-based auth model, leaf command paths, help text, explicit alias metadata, auth requirements, mutating and dry-run safety metadata, argument metadata with stable names and semantic types, boolean flag versus value-taking option shape, and structured command examples parsed from command help.

Schema argument metadata includes known custom validation constraints, such as `trade levels --trade-level-count` accepting only `5`, `10`, `20`, or `50`. Invalid values still fail with structured `usage_error` JSON on stderr and exit code `2`.

Mutating alert and watchlist commands support `--dry-run` so automation can inspect the planned request without sending it. Delete commands also require `--yes` for live deletion; use `--dry-run` first to inspect the delete request.

Use `fields <command path>` for machine-readable output field discovery before using `--fields`. It emits compact JSON with the preferred command path, exact case-sensitive raw API field names accepted by `--fields`, short descriptions, and type hints. Nested dashboard fields are section-qualified, such as `trades.TradeRank`, `clusters.MinFullTimeString24`, `levels.TradeLevelRank`, and `cluster_bombs.TradeCount`. It does not need a live API response or non-empty result rows. Unknown projected fields fail with exit code `2` and structured `usage_error` JSON on stderr.

`trade dashboard` returns `sections` metadata with per-section `count` and `empty` values for `trades`, `clusters`, `levels`, and `cluster_bombs`. Use it to distinguish a genuinely empty dashboard section from a populated sibling section without treating the whole object as an empty result.

The CLI intentionally does not embed a jq expression engine. Built-in output shaping stays focused on `--fields` and `--all-fields`; use external `jq` after projection when automation needs filters or derived objects. For example: `volumeleaders-agent trade list NVDA --fields FullTimeString24,Dollars | jq '.[] | select(.Dollars > 1000000)'`.

Top-level aliases are available for the highest-frequency trade commands: `trades` for `trade list`, `dashboard` for `trade dashboard`, and `levels` for `trade levels`. The schema keeps the canonical `trade ...` preferred paths, marks alias entries with `is_alias` and `alias_for`, and lists each alias on its canonical command so automation can normalize either form.

Use `commands` for lightweight CLI discovery. It emits a sorted plain-text list of leaf command paths, or grouped command names with short descriptions when run with `--grouped`.

Use `doctor` for a safe local readiness check before running live data commands. It emits compact JSON with the CLI version, credential-based auth status, credential source, config path when relevant, and an `auth.actions` array with exact recovery steps for LLM callers. It does not make a network request by default. Add `--live` to perform a low-cost authenticated connectivity check and create or refresh the cached session; live auth, HTTP transport, and API failures use exit codes `3`, `4`, and `5`.

Use `help <topic>` for built-in operational guidance when README access is unavailable. Topics are `agent`, `auth`, `environment`, `exit-codes`, `schema`, `examples`, and `workflows`; regular clap help remains available with `--help` on the root or any subcommand. The `agent` topic summarizes the recommended non-interactive automation flow, and `workflows` gives copy-paste starts for common agent tasks.

Install the agent skill with `npx skills add major/volumeleaders-rs` for CLI invocation guidance, field glossary, significance thresholds, and analysis patterns. The skill lives in `skills/volumeleaders/SKILL.md`.

Every leaf command also includes an `Examples:` section in its command-specific `--help` output, and schema command entries expose those examples as structured machine-readable discovery. Use those examples for copy-pasteable minimal and filtered invocations.

```bash
cargo run -- --help
cargo run -- commands
cargo run -- commands --grouped
cargo run -- doctor
cargo run -- doctor --live
cargo run -- fields trade list
cargo run -- help agent
cargo run -- help workflows
cargo run -- help auth
cargo run -- help exit-codes
cargo run -- schema
cargo run -- -vv doctor
cargo run -- --strict-empty trade list NVDA
cargo run -- trades NVDA
cargo run -- dashboard NVDA
cargo run -- levels NVDA
cargo run -- report list
cargo run -- trade list
cargo run -- trade list --help
cargo run -- completions bash
```

After building or installing, run the binary as `volumeleaders-agent`:

```bash
volumeleaders-agent doctor
volumeleaders-agent doctor --live
volumeleaders-agent commands
volumeleaders-agent fields trade list
volumeleaders-agent fields volume institutional | jq '.fields[].name'
volumeleaders-agent trade list NVDA --fields FullTimeString24,Dollars | jq '.[] | select(.Dollars > 1000000)'
volumeleaders-agent help agent
volumeleaders-agent help examples
volumeleaders-agent alert create --name BigTechSweeps --tickers AAPL,MSFT --dry-run
volumeleaders-agent watchlist delete --key 123 --dry-run
volumeleaders-agent -vv trade list NVDA
volumeleaders-agent trades NVDA
volumeleaders-agent dashboard NVDA
volumeleaders-agent levels NVDA
volumeleaders-agent report list
volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == "trade list")'
volumeleaders-agent trade list
volumeleaders-agent completions zsh > _volumeleaders-agent
```

`trade list` defaults mirror the browser `/Trades/GetTrades` request captured from the VolumeLeaders trades page: today's trades, 1000 requested rows, empty table search, `FullTimeString24` descending order, `MinVolume=10000`, `MaxVolume=2000000000`, `MinPrice=0`, `MaxPrice=100000`, `MinDollars=500000`, `MaxDollars=100000000000`, `Conditions=0`, `VCD=0`, `SecurityTypeKey=-1`, `RelativeSize=0`, `DarkPools=-1`, `Sweeps=-1`, `LatePrints=-1`, `SignaturePrints=-1`, `EvenShared=-1`, `TradeRank=100`, `TradeRankSnapshot=-1`, `MarketCap=0`, and all session toggles enabled. Pass date, range, or filter flags to override those browser defaults.

Trade-shaped CLI outputs use raw VolumeLeaders field names. When `--fields` is omitted, compact defaults are returned: trades/reports use `FullTimeString24`, `Volume`, `Price`, `Dollars`, `DollarsMultiplier`, `TradeRank`, and `LastComparibleTradeDate`; clusters use `MinFullTimeString24`, `TradeCount`, `Price`, `Dollars`, `DollarsMultiplier`, `TradeClusterRank`, and `LastComparibleTradeClusterDate`; cluster bombs use `MinFullTimeString24`, `TradeCount`, `Volume`, `Dollars`, `DollarsMultiplier`, `CumulativeDistribution`, `TradeClusterBombRank`, and `LastComparableTradeClusterBombDate`; levels use `Price`, `Dollars`, `Volume`, `Trades`, `RelativeSize`, `CumulativeDistribution`, `TradeLevelRank`, and `Dates`. Use `--all-fields` to emit every raw serialized field.

## Using as a library

Other Rust projects can depend on `rusty-volumeleaders` as an API client without pulling in the CLI by disabling default features:

```toml
[dependencies]
rusty-volumeleaders = { version = "0.4.0", default-features = false }
```

This excludes `clap` and `clap_complete` and exposes `Client`, `Session`, request builders, response models, `ClientError`, and `Result`. The `cli` feature (enabled by default) adds the `Cli` parser and `run` entry point used by the `volumeleaders-agent` binary.

## Client usage

Library consumers can call `login(username, password).await?` to create a `Session`, pass that session to `Client::new(session)?`, and optionally persist it with `save_session(&session)?` for reuse by the CLI-compatible XDG cache.

## Development

```bash
make fmt
make clippy
make test
make doc
make check
make coverage
make patch-coverage
make audit
make machete
```

Equivalent core Cargo commands:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --lib --no-default-features -- -D warnings
cargo test --all-features
cargo test --lib --no-default-features
cargo doc --all-features --no-deps
cargo doc --no-default-features --no-deps
```

`make check` runs formatting, clippy, tests, and docs for both supported feature shapes: the default CLI build and the library-only `--no-default-features` build. The GitHub CI workflow mirrors those checks across Linux, macOS, and Windows, with an MSRV job pinned to Rust 1.96. The separate audit workflow runs `actions-rust-lang/audit` on manifest changes and a daily schedule.

Most tests are inline `#[cfg(test)]` modules in `src/**`. Fixtures live in `tests/fixtures/*.json` and represent server payload contracts. HTTP tests use `mockito`.

CLI drift tests assert that every visible clap leaf appears in `commands` and `schema`, every public option has help text, every leaf has command-specific examples, structured schema examples stay valid, aliases keep canonical preferred paths, and global flags plus semantic argument metadata stay present in schema metadata.

`make coverage` and CI enforce 90 percent line coverage with `cargo llvm-cov --all-features`; Codecov also requires 90 percent project coverage and 100 percent patch coverage for changed lines. Run `make patch-coverage` before opening a PR to generate `lcov.info` and check changed-line coverage against `main`. Override the base branch with `PATCH_COVERAGE_BASE=<branch>` or use `DIFF_COVER='uvx diff-cover'` if `diff-cover` is not installed as a standalone command.

## Release automation

- `release-plz.yml` runs on pushes to `main` and on manual dispatch. It keeps a release PR open with the `Cargo.toml` version bump and `CHANGELOG.md` updates from conventional commits via `cliff.toml`.
- Merging the release PR runs release-plz in release mode. It pushes a `v<version>` tag using `RELEASE_PLZ_TOKEN` so the downstream cargo-dist workflow can run.
- `release-plz.toml` sets `publish = false` and `git_release_enable = false`. release-plz opens release PRs and pushes tags only.
- `release.yml` is the cargo-dist release workflow. It builds multi-platform artifacts, creates the GitHub Release, and publishes `rusty-volumeleaders` to crates.io through OIDC trusted publishing with a release-tagged `rust-lang/crates-io-auth-action` pin.
- The first crates.io release of `rusty-volumeleaders` must be published manually with a crates.io API token. After that, configure crates.io Trusted Publishing for owner `major`, repo `volumeleaders-rs`, workflow file `release.yml`, and package `rusty-volumeleaders`. No stored `CARGO_REGISTRY_TOKEN` secret is used after the one-time setup.

## Documentation freshness

Keep `AGENTS.md`, `README.md`, and `skills/volumeleaders/SKILL.md` updated in the same change as code. Update docs when commands, public APIs, auth/session behavior, fixtures, CI, release flow, or project layout change. Inaccurate docs are worse than no docs.
