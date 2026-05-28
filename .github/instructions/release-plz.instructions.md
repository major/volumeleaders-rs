---
applyTo: ".github/workflows/release-plz.yml"
---

# release-plz review instructions

- This workflow is the only entry point for releases. It runs on every push to `main` and has two jobs:
  - `release-plz-pr` (`command: release-pr`) opens or updates the release PR. It needs `contents: write` and `pull-requests: write`, plus the concurrency group `release-plz-${{ github.ref }}` to avoid duplicate PR runs.
  - `release-plz-release` (`command: release`) runs when release commits land on `main` and pushes the `v<version>` tag that triggers `release.yml`.
- Both jobs are gated by `if: ${{ github.repository_owner == 'major' }}` so forks never attempt to release.
- Both jobs use `RELEASE_PLZ_TOKEN` (a fine-grained PAT) instead of `GITHUB_TOKEN`. The default `GITHUB_TOKEN` cannot trigger downstream workflows, so the cargo-dist release pipeline would never run if we used it.
- `actions/checkout` uses `persist-credentials: false`. release-plz pushes via the GitHub API, not over the local git remote, so leaving credentials in the checkout would add risk without benefit.
- The workflow MUST NOT set `CARGO_REGISTRY_TOKEN` or otherwise enable `cargo publish`. `release-plz.toml` keeps `publish = false` and cargo-dist publishes via OIDC trusted publishing from `release.yml`.
- The workflow MUST NOT enable `git_release_enable`. cargo-dist creates the GitHub Release with binary artifacts; enabling it here would race.
- Pin third-party actions to either a release-plz-blessed tag (`release-plz/action@v0.5`) or a 40-character commit SHA (`actions/checkout`, `dtolnay/rust-toolchain`).
- When reviewing changes here, confirm `release-plz.toml` still keeps `publish = false`, `git_release_enable = false`, and `changelog_config = "cliff.toml"`. Those three settings are the contract between release-plz and cargo-dist.
