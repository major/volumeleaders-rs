# CLIENT CRATE NOTES

## OVERVIEW

`volumeleaders-client` is the library boundary for browser-session auth, HTTP requests, ASP.NET DataTables encoding, endpoint request builders, response models, fixtures, and redacted errors.

## DOC FRESHNESS

- Update this file and root `README.md` in the same change that modifies public exports, endpoint methods, request fields, response models, fixtures, auth/session handling, or test conventions.
- Keep this file crate-specific. Workspace commands and CLI behavior belong in parent `AGENTS.md` or `agent/AGENTS.md`.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Public exports | `src/lib.rs` | Reexports endpoint requests, models, `Client`, `Session`, errors |
| HTTP boundary | `src/client.rs` | Cookies, XSRF header, redirects, body limits, login detection |
| Session material | `src/session.rs` | Required cookie names and redacted debug behavior |
| Browser extraction | `src/browser_auth.rs` | Chrome then Firefox cookie extraction via `rookie` |
| DataTables wire format | `src/datatables.rs` | Bracketed form keys, pagination, `fetch_limit`, `fetch_all` |
| Response models | `src/models/` | API field names and custom serde helpers |
| Endpoint modules | `src/{alerts,clusters,earnings,executive_summary,levels,trades,volume,watchlists}.rs` | Request builders and client methods |
| Fixtures | `tests/fixtures/*.json` | Golden server payloads |
| Manual example | `examples/rookie_spike.rs` | Browser cookie extraction smoke helper |

## WIRE CONTRACT RULES

- Preserve exact server field names, column definitions, form keys, and DataTables bracketed encoding.
- Preserve cookie and XSRF handling in `Client`: callers supply cookies, POST methods add `X-XSRF-TOKEN`, and login redirects/password pages map to auth errors.
- `DataTablesResponse` accepts normal DataTables envelopes and treats `data: null` as an empty list.
- Multipart save/edit flows in `alerts.rs` and `watchlists.rs` depend on exact browser form field names and duplicate boolean conventions.
- `client.rs`, `datatables.rs`, `alerts.rs`, `watchlists.rs`, `executive_summary.rs`, and `levels.rs` are high-risk files because small wire changes affect many endpoints.

## TESTS AND FIXTURES

- Tests live inline in `#[cfg(test)] mod tests`; `client/tests/` stores fixtures only.
- Async HTTP tests use `mockito::Server::new_async()`.
- Endpoint tests load fixtures through `crate::test_support::read_fixture()`.
- Model tests use `include_str!()` for compile-time fixture loading.
- Fixture names are snake_case and usually end with `_response.json`; `trades_get_trades_response.json` mirrors the endpoint path.
- `test_support` is compiled only for tests or the `test-support` feature.

## COMMANDS

```bash
cargo test -p volumeleaders-client
cargo run -p volumeleaders-client --example rookie_spike
```

## ANTI-PATTERNS

- Do not expose cookie values, auth cookie values, or XSRF tokens in `Debug`, `Display`, logs, or errors.
- Do not replace explicit cookie headers with a reqwest cookie jar.
- Do not rewrite form encoding with a generic serializer unless tests prove byte-for-byte compatible behavior.
- Do not rename model fields to look more Rust-like if serde field names are matching server payloads.
- Do not move fixture-only files into source tests unless the test layout changes are documented here and in README.
