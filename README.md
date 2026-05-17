# volumeleaders-rs

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
├── .github/workflows/       # CI, audit, release-plz
├── AGENTS.md                # Workspace knowledge base for coding agents
├── Makefile                 # Local development commands
├── cliff.toml               # Changelog grouping
└── release-plz.toml         # Release tag automation
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

The CLI reads browser cookies automatically. If auth fails, log in to VolumeLeaders in the browser and retry. Command output goes to stdout as compact JSON by default. Use `--pretty` for indented JSON. Errors and logs go to stderr.

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

## Documentation freshness

Keep `README.md` and relevant `AGENTS.md` files updated in the same change as code. Update docs when commands, public APIs, auth/session behavior, fixtures, CI, release flow, or project layout change. Inaccurate docs are worse than no docs.
