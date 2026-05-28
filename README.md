# volumeleaders-rs

> **Disclaimer:** This project is unofficial and is not affiliated with, endorsed by, or sponsored by [volumeleaders.com](https://www.volumeleaders.com).

Rust crate for working with VolumeLeaders data from an authenticated browser session. The package is published as `rusty-volumeleaders` and includes both a reusable API client library and the `volumeleaders-agent` CLI.

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
├── codecov.yml               # Codecov project and patch coverage gates
├── dist-workspace.toml       # cargo-dist release artifact configuration
├── Makefile                  # Local development commands
├── cliff.toml                # Conventional-commit changelog grouping
├── LICENSE                   # Apache-2.0 license
└── release-plz.toml          # Release PR and tag automation
```

## Requirements

- Rust 1.95.0 or newer
- Browser login at `https://www.volumeleaders.com` for commands that need live authenticated data
- Optional tools for local maintenance: `cargo llvm-cov`, `cargo audit`
- Optional tool for local patch coverage checks: `diff-cover` or `uvx diff-cover`

## CLI usage

The CLI reads browser cookies automatically. If auth fails, log in to VolumeLeaders in the browser and retry. Command output goes to stdout as compact JSON by default. Pipe through `jq` for pretty-printed output. Errors and logs go to stderr.

```bash
cargo run -- --help
cargo run -- report list
cargo run -- trade list
cargo run -- completions bash
```

After building or installing, run the binary as `volumeleaders-agent`:

```bash
volumeleaders-agent report list
volumeleaders-agent trade list
volumeleaders-agent completions zsh > _volumeleaders-agent
```

`trade list` defaults mirror the browser `/Trades/GetTrades` request captured from the VolumeLeaders trades page: today's trades, 1000 requested rows, empty table search, `FullTimeString24` descending order, `MinVolume=10000`, `MaxVolume=2000000000`, `MinPrice=0`, `MaxPrice=100000`, `MinDollars=500000`, `MaxDollars=100000000000`, `Conditions=0`, `VCD=0`, `SecurityTypeKey=-1`, `RelativeSize=0`, `DarkPools=-1`, `Sweeps=-1`, `LatePrints=-1`, `SignaturePrints=-1`, `EvenShared=-1`, `TradeRank=100`, `TradeRankSnapshot=-1`, `MarketCap=0`, and all session toggles enabled. Pass date, range, or filter flags to override those browser defaults.

Trade-shaped outputs intentionally omit the upstream `PercentDailyVolume` field. Live report data returns that value as `0.0` for current and prior trading days, so returning it would suggest a meaningful percentage where the source data does not provide one. Compact defaults also omit `TradeConditions`, `RelativeSize`, `Name`, and `Volume` on trade-shaped rows to avoid surfacing fields that are consistently null or misleadingly sparse in those surfaces. `RelativeSize` remains in full output and is still surfaced in level-centric data when requested.

## Using as a library

Other Rust projects can depend on `rusty-volumeleaders` as an API client without pulling in the CLI by disabling default features:

```toml
[dependencies]
rusty-volumeleaders = { version = "0.4.0", default-features = false }
```

This excludes `clap` and `clap_complete` and exposes `Client`, `Session`, request builders, response models, `ClientError`, and `Result`. The `cli` feature (enabled by default) adds the `Cli` parser and `run` entry point used by the `volumeleaders-agent` binary.

## Client example

```bash
cargo run --example rookie_spike
```

The `rookie_spike` example checks whether required VolumeLeaders cookies can be extracted from Chrome, then Firefox, and prints manual fallback guidance if needed.

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
```

Equivalent core Cargo commands:

```bash
cargo fmt --all
cargo clippy --workspace -- -D clippy::all
cargo test --workspace
cargo doc --workspace --no-deps
```

Most tests are inline `#[cfg(test)]` modules in `src/**`. Fixtures live in `tests/fixtures/*.json` and represent server payload contracts. HTTP tests use `mockito`.

`make coverage` and CI enforce 90 percent line coverage with `cargo llvm-cov`; Codecov also requires 90 percent project coverage and 100 percent patch coverage for changed lines. Run `make patch-coverage` before opening a PR to generate `lcov.info` and check changed-line coverage against `main`. Override the base branch with `PATCH_COVERAGE_BASE=<branch>` or use `DIFF_COVER='uvx diff-cover'` if `diff-cover` is not installed as a standalone command.

## Release automation

- `release-plz.yml` runs on pushes to `main` and on manual dispatch. It keeps a release PR open with the `Cargo.toml` version bump and `CHANGELOG.md` updates from conventional commits via `cliff.toml`.
- Merging the release PR runs release-plz in release mode. It pushes a `v<version>` tag using `RELEASE_PLZ_TOKEN` so the downstream cargo-dist workflow can run.
- `release-plz.toml` sets `publish = false` and `git_release_enable = false`. release-plz opens release PRs and pushes tags only.
- `release.yml` is the cargo-dist release workflow. It builds multi-platform artifacts, creates the GitHub Release, and publishes `rusty-volumeleaders` to crates.io through OIDC trusted publishing with `rust-lang/crates-io-auth-action`.
- The first crates.io release of `rusty-volumeleaders` must be published manually with a crates.io API token. After that, configure crates.io Trusted Publishing for owner `major`, repo `volumeleaders-rs`, workflow file `release.yml`, and package `rusty-volumeleaders`. No stored `CARGO_REGISTRY_TOKEN` secret is used after the one-time setup.

## Documentation freshness

Keep `README.md` and `AGENTS.md` updated in the same change as code. Update docs when commands, public APIs, auth/session behavior, fixtures, CI, release flow, or project layout change. Inaccurate docs are worse than no docs.
