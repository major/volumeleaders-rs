# PROJECT KNOWLEDGE BASE

## OVERVIEW

Rust 2024 workspace for VolumeLeaders access. `client` owns the browser-session API client and wire contracts. `agent` owns the CLI, command routing, and stdout formatting.

## DOC FRESHNESS

- Keep `AGENTS.md` and `README.md` current in the same change that modifies commands, public APIs, auth/session behavior, fixtures, CI, release flow, or project layout.
- Stale docs are worse than missing docs here. If code and docs disagree, update docs or remove the inaccurate claim.
- Keep AGENTS files short and progressively disclosed. Root covers workspace rules only. Crate AGENTS files cover crate-specific hazards only. Do not duplicate parent guidance.

## STRUCTURE

```text
volumeleaders-rs/
├── Cargo.toml                 # Workspace: client, agent
├── Makefile                   # Local fmt, clippy, test, doc, coverage, audit
├── README.md                  # Human project overview and commands
├── AGENTS.md                  # Workspace rules for agents
├── client/                    # API client library, browser-session auth, fixtures
├── agent/                     # CLI binary and command handlers
├── .github/workflows/         # CI, audit, release-plz
├── cliff.toml                 # Conventional-commit changelog grouping
└── release-plz.toml           # Release tag automation
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Workspace membership | `Cargo.toml` | Manifest-only root with resolver `2` |
| Local commands | `Makefile` | `make check` runs fmt, clippy, test, doc |
| CI behavior | `.github/workflows/ci.yml` | Linux, macOS, Windows test and clippy matrix |
| Release behavior | `release-plz.toml`, `cliff.toml` | Tags enabled, GitHub releases disabled |
| API client work | `client/` | Read `client/AGENTS.md` first |
| CLI work | `agent/` | Read `agent/AGENTS.md` first |
| Planning artifacts | `.sisyphus/` | Historical plans and evidence, not source of truth |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `volumeleaders_agent::run` | Function | `agent/src/lib.rs` | CLI parse and dispatch entry |
| `Cli`, `Commands` | clap types | `agent/src/cli.rs` | Top-level command tree |
| `volumeleaders_client` | Crate | `client/src/lib.rs` | API client public boundary |

## CONVENTIONS

- Rust edition `2024`, MSRV `1.95.0` in both member crates.
- Dependency direction is one-way: `agent` depends on `volumeleaders-client`; `client` must not depend on `agent`.
- Formatting follows `cargo fmt --all` and `.editorconfig`: UTF-8, 4-space indent, final newline, trim trailing whitespace except Markdown.
- Local clippy command is stricter than CI: `cargo clippy --workspace -- -D clippy::all`.
- CI clippy uses `--all-targets` and allows `clippy::needless_borrow` and `clippy::large_enum_variant`.
- Conventional commits feed `cliff.toml` changelog groups and release-plz tags.

## ANTI-PATTERNS

- Do not put crate-specific wire, fixture, or CLI details in root docs when a crate AGENTS file can own them.
- Do not add generated or historical `.sisyphus/` claims to docs unless current source files still prove them.
- Do not create deep AGENTS files unless a subdirectory has distinct rules. Parent coverage is preferred.

## COMMANDS

```bash
make fmt
make clippy
make test
make doc
make check
make coverage
make audit
cargo test --workspace
cargo doc --workspace --no-deps
```

## NOTES

- No dedicated build or benchmark target is codified. Use Cargo defaults only when needed, and document any new command if it becomes canonical.
- Coverage target requires `cargo llvm-cov` and enforces 90 percent line coverage.
- Audit is a separate workflow and also runs on manifest changes plus a daily schedule.
- If a code change modifies public CLI behavior, update README examples and the matching `agent/AGENTS.md` note.
- If a code change modifies request fields, response models, auth, fixtures, or pagination, update README scope notes and `client/AGENTS.md`.
