# PROJECT KNOWLEDGE BASE

## OVERVIEW

Rust 2024 single-crate project for VolumeLeaders access. The package is `rusty-volumeleaders`; it exposes the browser-session API client as the library and the `volumeleaders-agent` binary behind the default `cli` feature.

## DOC FRESHNESS

- Keep `AGENTS.md` and `README.md` current in the same change that modifies commands, public APIs, auth/session behavior, fixtures, CI, release flow, or project layout.
- Stale docs are worse than missing docs here. If code and docs disagree, update docs or remove the inaccurate claim.
- Keep AGENTS files short and progressively disclosed. Parent coverage is preferred; add deeper files only when a subdirectory has distinct rules.

## STRUCTURE

```text
volumeleaders-rs/
├── Cargo.toml                 # Single package: rusty-volumeleaders
├── Makefile                   # Local fmt, clippy, test, doc, coverage, audit
├── rust-toolchain.toml         # Local Rust toolchain pin matching CI MSRV
├── codecov.yml                # Codecov project and patch coverage gates
├── README.md                  # Human project overview and commands
├── AGENTS.md                  # Project rules for agents
├── src/                       # API client library modules
├── src/cli/                   # CLI parser, commands, output, and helpers
├── tests/fixtures/            # JSON payload fixtures
├── examples/                  # Client examples
├── .github/workflows/         # CI, audit, release-plz, cargo-dist releases
├── .github/instructions/      # GitHub Copilot review guidance for release automation
├── dist-workspace.toml        # cargo-dist release artifact configuration
├── cliff.toml                 # Conventional-commit changelog grouping
├── LICENSE                    # Apache-2.0 license
└── release-plz.toml           # Release PR and tag automation
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Package metadata and feature gates | `Cargo.toml` | Single package with default `cli` feature and `volumeleaders-agent` binary |
| API client work | `src/` | Library modules export client, session, request builders, response models, errors, and pagination |
| CLI work | `src/cli/` | Clap args, command routing, output formatting, and CLI helper modules |
| CLI runtime errors | `src/cli/error.rs` | Structured JSON stderr envelope and semantic exit-code mapping |
| CLI schema | `src/cli/schema.rs` | Machine-readable command metadata generated from the live clap tree |
| Fixtures | `tests/fixtures/` | JSON payload contracts used by tests |
| Local commands | `Makefile` | `make check` runs fmt, clippy, test, doc; `make patch-coverage` checks changed-line coverage; `make machete` checks unused dependencies |
| CI behavior | `.github/workflows/ci.yml` | Linux, macOS, Windows test and clippy matrix |
| Codecov policy | `codecov.yml` | Project coverage floor is 90 percent; patch coverage floor is 100 percent |
| Release PR and tag behavior | `.github/workflows/release-plz.yml`, `release-plz.toml`, `cliff.toml` | Release PRs, changelog updates, and tags |
| Release artifacts and crates.io publish | `dist-workspace.toml`, `.github/workflows/release.yml` | cargo-dist installers, GitHub Releases, and OIDC crates.io publish |
| Planning artifacts | `.sisyphus/` | Historical plans and evidence, not source of truth |

## CODE MAP

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `rusty_volumeleaders::Client` | Type | `src/lib.rs` | API client public boundary |
| `rusty_volumeleaders::Session` | Type | `src/lib.rs` | Browser-session auth state |
| `rusty_volumeleaders::cli::run` | Function | `src/cli/mod.rs` | CLI parse and dispatch entry |
| `Cli`, `Commands` | clap types | `src/cli/args.rs` | Top-level command tree |

## CONVENTIONS

- Rust edition `2024`, MSRV `1.95.0`.
- `rust-toolchain.toml` pins Rust 1.95 locally for consistency with the MSRV workflow.
- The package is publishable as `rusty-volumeleaders`; keep crate metadata valid for crates.io.
- The CLI binary remains `volumeleaders-agent` and is built only when the `cli` feature is enabled. `cli` is enabled by default.
- Runtime CLI errors are emitted to stderr as `{"ok":false,"error":{"kind":"...","message":"..."}}`; stdout remains compact JSON for successful commands.
- `volumeleaders-agent schema` emits machine-readable discovery metadata from `Cli::command()` so command paths, help text, aliases, auth requirements, and arguments cannot drift from clap definitions.
- Semantic CLI exit codes are `0` success, `2` usage error, `3` auth error, `4` HTTP transport error, `5` API error, `6` JSON parse or output transformation error, and `7` reserved for strict empty results.
- Library consumers that do not need the CLI should use `rusty-volumeleaders = { version = "0.4.0", default-features = false }` to avoid clap and CLI-only dependencies.
- Formatting follows `cargo fmt --all` and `.editorconfig`: UTF-8, 4-space indent, final newline, trim trailing whitespace except Markdown.
- Local and CI checks cover both supported feature shapes: the default CLI build with `--all-features` and the library-only build with `--no-default-features`.
- Clippy uses `cargo clippy --all-targets --all-features -- -D warnings` and `cargo clippy --lib --no-default-features -- -D warnings` on Linux, macOS, and Windows.
- `Cargo.toml` denies Rust `unused` lints so unused code and imports fail outside clippy-only workflows too.
- Conventional commits feed `cliff.toml` changelog groups and release-plz release PRs.

## ANTI-PATTERNS

- Do not recreate `client/` or `agent/` crates. This repository is now a single package.
- Do not add generated or historical `.sisyphus/` claims to docs unless current source files still prove them.
- Do not move CLI-only dependencies back into unconditional dependencies. Keep them behind the additive `cli` feature.
- Do not add broad `#[allow(...)]` suppressions without a rationale in this file or next to the item.

## COMMANDS

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
cargo test --all-features
cargo test --lib --no-default-features
cargo doc --all-features --no-deps
cargo doc --no-default-features --no-deps
```

## NOTES

- No dedicated build or benchmark target is codified. Use Cargo defaults only when needed, and document any new command if it becomes canonical.
- Coverage target requires `cargo llvm-cov` and enforces 90 percent line coverage with `--all-features` locally (`make coverage`) and in CI (`ci.yml` coverage job). Codecov status checks use `codecov.yml` for a 90 percent project floor and 100 percent patch floor. Run `make patch-coverage` before opening a PR to approximate the Codecov patch gate locally with `diff-cover` against `main`; override with `PATCH_COVERAGE_BASE=<branch>` or `DIFF_COVER='uvx diff-cover'` when needed.
- The crate root sets `#![deny(missing_docs)]`. Wire-type models use a module-level allow, clap arg structs and request builders use item-level `#[allow(missing_docs)]`, `Commands` allows `clippy::large_enum_variant` for generated parser shape, and alert construction allows `clippy::too_many_arguments` for request fidelity. New public items need doc comments or an explicit allow with rationale.
- Audit is a separate workflow and also runs on manifest changes plus a daily schedule.
- CodeRabbit uses `.coderabbit.yaml`; keep its path instructions aligned with the single-crate `src/**` layout, library-only feature support, machine-readable CLI output, and release automation policy.
- `release-plz.yml` uses `RELEASE_PLZ_TOKEN`; the token is needed so release PR branch updates and pushed release tags trigger normal workflows.
- `release-plz.toml` creates `v{{ version }}` tags but does not publish crates.io packages or GitHub Releases.
- `dist-workspace.toml` configures cargo-dist for the `volumeleaders-agent` binary installers. Regenerate `.github/workflows/release.yml` after changing dist settings, then reapply the Rust toolchain update and OIDC publish job if cargo-dist drops them.
- `.github/workflows/release.yml` publishes the single `rusty-volumeleaders` crate through OIDC trusted publishing after cargo-dist creates the GitHub Release.
- The first crates.io release of `rusty-volumeleaders` must be published manually with a crates.io API token. After that, configure crates.io Trusted Publishing for repo `major/volumeleaders-rs`, workflow file `release.yml`, and package `rusty-volumeleaders`.
- If a code change modifies public CLI behavior, update README examples and this file.
- If a code change modifies request fields, response models, auth, fixtures, or pagination, update README scope notes and this file.
