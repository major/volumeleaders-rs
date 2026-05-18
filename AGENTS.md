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
├── .github/workflows/         # CI, audit, release-plz, cargo-dist releases
├── dist-workspace.toml        # cargo-dist release artifact configuration
├── cliff.toml                 # Conventional-commit changelog grouping
├── LICENSE                    # Apache-2.0 license
└── release-plz.toml           # Release PR, crates.io publish, and tag automation
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Workspace membership | `Cargo.toml` | Manifest-only root with resolver `2` |
| Local commands | `Makefile` | `make check` runs fmt, clippy, test, doc |
| CI behavior | `.github/workflows/ci.yml` | Linux, macOS, Windows test and clippy matrix |
| Release behavior | `.github/workflows/cd.yml`, `release-plz.toml`, `cliff.toml` | Release PRs, crates.io publish, and tags |
| Release artifacts | `dist-workspace.toml`, `.github/workflows/release.yml` | cargo-dist installers and GitHub Releases |
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
- Conventional commits feed `cliff.toml` changelog groups and release-plz release PRs.
- Workspace crates are publishable. Keep crate package metadata valid for crates.io.
- Publish order matters: `volumeleaders-client` before `volumeleaders-agent` because the agent depends on the client crate.

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
- Coverage target requires `cargo llvm-cov` and enforces 90 percent line coverage locally (`make coverage`) and in CI (`ci.yml` coverage job).
- Both crate roots set `#![deny(missing_docs)]`. Wire-type models use a module-level allow; clap arg structs and request builders use item-level `#[allow(missing_docs)]`. New public items need doc comments or an explicit allow with rationale.
- Audit is a separate workflow and also runs on manifest changes plus a daily schedule.
- `cd.yml` runs release-plz with `RELEASE_PLZ_TOKEN`; the token is needed so release PR branch updates and release tags trigger normal workflows.
- `release-plz.toml` publishes crates.io packages and creates git tags, but GitHub Releases are disabled because cargo-dist owns artifact releases.
- `dist-workspace.toml` configures cargo-dist for the `volumeleaders-agent` binary installers. Regenerate `.github/workflows/release.yml` after changing dist settings.
- Release artifact jobs in `.github/workflows/release.yml` explicitly ensure `rustup` exists and update Rust to stable before `dist build`; keep those steps if regenerating cargo-dist CI so hosted runner images cannot fall below the workspace MSRV.
- After the first manual crates.io publish, configure crates.io Trusted Publishing for repo `major/volumeleaders-rs` and workflow file `cd.yml` so future release-plz publishes use GitHub OIDC.
- If a code change modifies public CLI behavior, update README examples and the matching `agent/AGENTS.md` note.
- If a code change modifies request fields, response models, auth, fixtures, or pagination, update README scope notes and `client/AGENTS.md`.
