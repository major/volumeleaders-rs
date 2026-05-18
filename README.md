# volumeleaders-rs

> **Disclaimer:** This project is unofficial and is not affiliated with, endorsed by, or sponsored by [volumeleaders.com](https://www.volumeleaders.com).

Rust workspace for working with VolumeLeaders data from an authenticated browser session. The workspace has a reusable API client crate and a CLI agent crate built on top of it.

## Crates

| Crate | Purpose |
|-------|---------|
| `volumeleaders-client` | Browser-session API client, request builders, response models, fixtures, and auth/session handling |
| `volumeleaders-agent` | CLI for reports, trades, volume, market data, alerts, watchlists, and shell completions |

## Repository layout

```text
.
├── client/                  # Library crate
├── agent/                   # CLI crate
├── .github/workflows/       # CI, audit, release-plz, cargo-dist releases
├── AGENTS.md                # Workspace knowledge base for coding agents
├── dist-workspace.toml      # cargo-dist release artifact configuration
├── Makefile                 # Local development commands
├── cliff.toml               # Changelog grouping
├── LICENSE                  # Apache-2.0 license
└── release-plz.toml         # Release PR, crates.io publish, and tag automation
```

## Requirements

- Rust 1.95.0 or newer
- Browser login at `https://www.volumeleaders.com` for commands that need live authenticated data
- Optional tools for local maintenance: `cargo llvm-cov`, `cargo audit`

## Development commands

```bash
make fmt
make clippy
make test
make doc
make check
make coverage
make audit
```

Equivalent core Cargo commands:

```bash
cargo fmt --all
cargo clippy --workspace -- -D clippy::all
cargo test --workspace
cargo doc --workspace --no-deps
```

## CLI usage

```bash
cargo run -p volumeleaders-agent -- --help
cargo run -p volumeleaders-agent -- report list
cargo run -p volumeleaders-agent -- completions bash
```

The CLI reads browser cookies automatically. If auth fails, log in to VolumeLeaders in the browser and retry. Command output goes to stdout as compact JSON by default. Use `--json-table` for a token-efficient array-of-arrays format where keys appear once as a header row. Pipe through `jq` for pretty-printed output. Errors and logs go to stderr.

Trade-shaped outputs intentionally omit the upstream `PercentDailyVolume` field. Live report data returns that value as `0.0` for current and prior trading days, so returning it would suggest a meaningful percentage where the source data does not provide one. Compact defaults also omit `TradeConditions`, `RelativeSize`, `Name`, and `Volume` on trade-shaped rows to avoid surfacing fields that are consistently null or misleadingly sparse in those surfaces. `RelativeSize` remains in full output and is still surfaced in level-centric data when requested.

## Client example

```bash
cargo run -p volumeleaders-client --example rookie_spike
```

The `rookie_spike` example checks whether required VolumeLeaders cookies can be extracted from Chrome, then Firefox, and prints manual fallback guidance if needed.

## Tests and fixtures

- Most tests are inline `#[cfg(test)]` modules in `client/src/**` and `agent/src/**`.
- Client fixtures live in `client/tests/fixtures/*.json` and represent server payload contracts.
- Client HTTP tests use `mockito`.
- There are no standalone Rust integration test files or benchmarks today.

## Release automation

- `cd.yml` runs release-plz on pushes to `main` and on manual dispatch. It uses the `RELEASE_PLZ_TOKEN` secret so release PR branch updates and release tags can trigger normal GitHub Actions workflows.
- `release-plz.toml` keeps the changelog current, opens release PRs, publishes publishable workspace crates to crates.io, and creates git tags. GitHub Releases are disabled there because cargo-dist owns artifact releases.
- `dist-workspace.toml` configures cargo-dist for the `volumeleaders-agent` binary installers and generated GitHub Release workflow.
- The cargo-dist release workflow updates Rust to the current stable toolchain before artifact builds so runner image defaults do not fall below the workspace MSRV.
- The first crates.io release for each crate must be published manually with a crates.io API token. After that, configure crates.io Trusted Publishing for `major/volumeleaders-rs` with workflow file `cd.yml`; release-plz can then publish through GitHub OIDC without storing a `CARGO_REGISTRY_TOKEN` secret.
- Publish workspace crates in dependency order: `volumeleaders-client` first, then `volumeleaders-agent` after the client version is available in the crates.io index.

## Documentation freshness

Keep `README.md` and relevant `AGENTS.md` files updated in the same change as code. Update docs when commands, public APIs, auth/session behavior, fixtures, CI, release flow, or project layout change. Inaccurate docs are worse than no docs.
