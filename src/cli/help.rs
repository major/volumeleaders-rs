//! Built-in operational help topics.

use std::io::{self, Write};

use crate::cli::error::CliExit;
use crate::cli::output::finish_output;
use crate::cli::{HelpArgs, HelpTopic};

/// Emit a built-in operational help topic as plain text.
pub fn handle(args: &HelpArgs) -> Result<(), CliExit> {
    finish_output(write_text(topic_text(args.topic)))
}

fn topic_text(topic: HelpTopic) -> &'static str {
    match topic {
        HelpTopic::Agent => AGENT_HELP,
        HelpTopic::Auth => AUTH_HELP,
        HelpTopic::Environment => ENVIRONMENT_HELP,
        HelpTopic::ExitCodes => EXIT_CODES_HELP,
        HelpTopic::Schema => SCHEMA_HELP,
        HelpTopic::Examples => EXAMPLES_HELP,
        HelpTopic::Workflows => WORKFLOWS_HELP,
    }
}

fn write_text(output: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(output.as_bytes())
}

const AGENT_HELP: &str = r#"agent

Guidance for non-interactive automation and coding agents:

1. Check local readiness first.
   Run `volumeleaders-agent doctor` before authenticated data commands when automation needs to confirm credential readiness. `doctor` is local-only by default and does not make a network request. If auth is not ready, read the JSON `auth.actions` array and do exactly what it says. Use `volumeleaders-agent doctor --live` when you also need a low-cost authenticated connectivity check.

2. Discover commands before guessing.
Use `volumeleaders-agent commands` for a quick leaf-command list, `volumeleaders-agent commands --grouped` for grouped descriptions, `volumeleaders-agent schema` for machine-readable command, alias, auth, mutation safety, help, stable argument, semantic argument, and boolean option metadata, and `volumeleaders-agent fields <command path>` for field projection metadata.

3. Keep streams separate.
   Successful data commands write compact JSON to stdout. Discovery and help commands write plain text to stdout. Diagnostics, verbosity logs from `-v`/`-vv`/`-vvv`, and structured runtime errors go to stderr.

4. Shape output deliberately.
   Use `volumeleaders-agent fields trade list` to discover exact case-sensitive field names before passing command-specific `--fields` or `--all-fields`. The CLI intentionally keeps `jq` as an external pipeline step: built-in projection trims fields first, then external `jq` handles reshaping, filtering, sorting, and pretty-printing without changing stderr diagnostics.

5. Treat empty rows and mutations carefully.
Use global `--strict-empty` when an empty record array should fail automation with exit code 7. Avoid mutating alert and watchlist commands unless the user explicitly requested the mutation. When mutation is requested, run the command with `--dry-run` first; live delete commands require `--yes`.

6. Recover by exit code.
Exit 2 means usage or argument validation failed, including unknown `--fields` names. Exit 3 means credential login setup failed. Exit 4 means HTTP transport failed. Exit 5 means the API returned an error. Exit 6 means JSON parsing or output transformation failed. Exit 7 means `--strict-empty` rejected an empty record array.

Copy-paste examples:

volumeleaders-agent doctor
volumeleaders-agent doctor --live
volumeleaders-agent commands --grouped
volumeleaders-agent fields trade list
volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == "trade list")'
volumeleaders-agent schema | jq '.commands[] | select(.mutating == true) | {path, supports_dry_run, requires_confirmation}'
volumeleaders-agent help exit-codes
volumeleaders-agent alert create --name BigTechSweeps --tickers AAPL,MSFT --dry-run
volumeleaders-agent --strict-empty trades NVDA
volumeleaders-agent trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields FullTimeString24,Volume,Price,Dollars
volumeleaders-agent volume institutional --date 2026-05-27 --tickers AAPL,NVDA --limit 50 --fields Ticker,Dollars | jq '.[] | select(.Dollars > 1000000) | {ticker: .Ticker, dollars: .Dollars}'
"#;

const AUTH_HELP: &str = r#"auth

VolumeLeaders live-data commands authenticate with a cached session or username/password login credentials.

Credential source order for live commands:
1. Use the cached session from `~/.cache/volumeleaders-agent/cookies.json` when it exists and the XSRF refresh succeeds.
2. If the cache is missing, invalid, or expired, use non-empty `VL_USERNAME` and `VL_PASSWORD` environment variables.
3. If neither environment variable is set, read `~/.config/volumeleaders-agent/config.json`.
4. If credentials are still unavailable or invalid, exit 3 with structured auth guidance on stderr.

Config file shape:
{"username":"YOUR_EMAIL","password":"YOUR_PASSWORD"}

Environment variables take precedence over the config file. If either `VL_USERNAME` or `VL_PASSWORD` is set, both must be set and non-empty; a partial or empty environment setup is an auth error and the config file is not used.

Run `volumeleaders-agent doctor` for a safe local readiness check. It exits 0 when a cached session is usable or complete credentials are configured. It exits 3 when auth is missing or invalid and includes an `auth.actions` array with exact steps for LLM callers. Run `volumeleaders-agent doctor --live` to verify the credentials against VolumeLeaders and create or refresh the cached session. Cookie values, passwords, and XSRF tokens are never printed.

Common fixes:
- Set `VL_USERNAME` and `VL_PASSWORD` in the process environment.
- Or create `~/.config/volumeleaders-agent/config.json` with username and password fields.
- Do not set only one auth environment variable.
- Run `volumeleaders-agent doctor --live` after editing credentials.
"#;

const ENVIRONMENT_HELP: &str = r#"environment

The CLI uses platform XDG directories through the operating system account running `volumeleaders-agent`.

Expected setup:
- Rust users can run the CLI with `cargo run -- ...`; installed users run `volumeleaders-agent ...`.
- Cached sessions are stored under `~/.cache/volumeleaders-agent/cookies.json`.
- Optional credential config is stored at `~/.config/volumeleaders-agent/config.json` unless `XDG_CONFIG_HOME` points somewhere else.
- Credential environment variables are `VL_USERNAME` and `VL_PASSWORD`; they override config file credentials.
- stdout is reserved for command data, either compact JSON for data commands or plain text for discovery/help commands.
- stderr is reserved for diagnostics and structured runtime errors. Use `-v`, `-vv`, or `-vvv` to enable info, debug, or trace diagnostics without changing stdout.

Use `volumeleaders-agent doctor` before live data commands when automation needs to confirm local auth readiness without spending API quota. If it exits 3, parse `auth.actions` from stdout and apply those steps. Use `volumeleaders-agent doctor --live` only when automation needs to verify authenticated connectivity too.
"#;

const EXIT_CODES_HELP: &str = r#"exit-codes

Semantic exit codes are stable for automation:

0  success
2  usage error, invalid arguments, unknown --fields names, or clap validation failure
3  auth error, cached session or login credentials are missing, expired, invalid, or incomplete
4  HTTP transport error
5  VolumeLeaders API error response
6  JSON parse or output transformation error
7  strict empty-result handling requested with --strict-empty

Runtime errors are written to stderr as one compact JSON object:
{"ok":false,"error":{"kind":"auth_error","message":"set VL_USERNAME and VL_PASSWORD environment variables or create ~/.config/volumeleaders-agent/config.json with username and password fields"}}

Recovery guidance:
- Exit 2: check `--help`, `commands`, `schema`, `fields <command path>`, or this help topic.
- Exit 3: run `doctor`, read `auth.actions`, then set both env vars or create the XDG config file before retrying.
- Exit 4 or 5: retry later or narrow the request.
- Exit 6: check field names and JSON-processing pipeline assumptions.
- Exit 7: check the ticker, widen the date range, relax filters, or accept that no configured rows may be valid account state.

Verbosity: no `-v` logs warnings and errors, `-v` enables info diagnostics, `-vv` enables debug diagnostics, and `-vvv` enables trace diagnostics. Diagnostic logs always go to stderr.
"#;

const SCHEMA_HELP: &str = r#"schema

Use `schema` and `commands` for binary-native CLI discovery.

`volumeleaders-agent schema` emits compact JSON generated from the live clap tree. It includes the binary version, auth model, leaf command paths, explicit alias metadata, auth requirements, mutating and dry-run safety metadata, help text, argument metadata with stable names and semantic hints, boolean flag versus value-taking option shape, and structured command examples.

`volumeleaders-agent fields <command path>` emits compact JSON with raw output fields accepted by `--fields`, including each exact case-sensitive field name, short description, and type hint. It does not need live API rows. Unknown projected fields are reported as structured usage errors with exit code 2 before output is written.

Built-in output shaping is intentionally limited to `--fields` and `--all-fields`. For jq-style filtering, sorting, object construction, or pretty-printing, pipe stdout to external `jq`; diagnostics and runtime error JSON remain on stderr.

Common top-level aliases such as `trades`, `dashboard`, and `levels` are reported with their canonical `trade ...` preferred paths. Alias entries set `is_alias` and `alias_for`; canonical entries list their aliases so agents can normalize generated commands.

`volumeleaders-agent commands` emits one sorted leaf command path per line. It is lighter than `schema` and useful when an agent only needs to choose a command.

`volumeleaders-agent commands --grouped` emits top-level groups with short descriptions for each leaf command.

Each schema command entry can include an `examples` array with structured copy-paste commands so automation does not need to parse `long_about` prose.

Useful discovery commands:
- volumeleaders-agent commands
- volumeleaders-agent commands --grouped
- volumeleaders-agent fields trade list
- volumeleaders-agent fields volume institutional
- volumeleaders-agent trade list NVDA --fields FullTimeString24,Price,Dollars,DollarsMultiplier | jq '.[] | select(.Dollars > 1000000)'
- volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == "trade list")'
"#;

const EXAMPLES_HELP: &str = r#"examples

High-value command examples:

volumeleaders-agent doctor
volumeleaders-agent doctor --live
volumeleaders-agent commands
volumeleaders-agent commands --grouped
volumeleaders-agent fields trade list
volumeleaders-agent fields volume institutional | jq '.fields[].name'
volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == "trade list")'

volumeleaders-agent report list
volumeleaders-agent report dark-pool-sweeps
volumeleaders-agent trades NVDA
volumeleaders-agent trade list NVDA
volumeleaders-agent -vv trade list NVDA
volumeleaders-agent --strict-empty trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields FullTimeString24,Volume,Price,Dollars
volumeleaders-agent trade list NVDA --fields Ticker,Dollars | jq '.[] | select(.Dollars > 1000000)'
volumeleaders-agent dashboard NVDA
volumeleaders-agent trade dashboard NVDA
volumeleaders-agent levels NVDA

volumeleaders-agent volume institutional --date 2026-05-27 --tickers AAPL
volumeleaders-agent volume institutional --date 2026-05-27 --limit 50
volumeleaders-agent market earnings --start-date 2026-05-01 --end-date 2026-05-27
volumeleaders-agent watchlist tickers --watchlist-key 123

These examples are also exposed as structured schema `examples` entries for machine-readable discovery.
"#;

const WORKFLOWS_HELP: &str = r#"workflows

Workflow guidance for common agent tasks:

General rules:
- Start with defaults first. Add date ranges, tickers, filters, or `jq` only when the research question requires them.
- Institutional prints show unusual activity. They do not guarantee buy or sell intent.
- Run `volumeleaders-agent fields <command path>` before choosing `--fields`; field names are exact and case-sensitive.
- Use `volumeleaders-agent doctor` before live-data workflows when automation needs to confirm local auth readiness.

1. Confirm local readiness
Recommended first command: `volumeleaders-agent doctor`

Copy-paste examples:
volumeleaders-agent doctor
volumeleaders-agent doctor --live
volumeleaders-agent help auth

2. Discover the right command before guessing
Recommended first command: `volumeleaders-agent commands --grouped`

Copy-paste examples:
volumeleaders-agent commands --grouped
volumeleaders-agent schema | jq '.commands[] | {path: .preferred_path, auth_required, mutating}'
volumeleaders-agent help schema

3. Single ticker institutional context
Recommended first command: `volumeleaders-agent trade dashboard NVDA`

Copy-paste examples:
volumeleaders-agent trade dashboard NVDA
volumeleaders-agent fields trade dashboard
volumeleaders-agent trade dashboard NVDA --fields trades.TradeRank,clusters.MinFullTimeString24,levels.TradeLevelRank,cluster_bombs.TradeCount
volumeleaders-agent trade list NVDA
volumeleaders-agent fields trade list
volumeleaders-agent trade list NVDA --fields FullTimeString24,Volume,Price,Dollars,DollarsMultiplier
volumeleaders-agent trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields FullTimeString24,Volume,Price,Dollars | jq '.[] | select(.Dollars > 1000000)'

4. Broad daily scan
Recommended first command: `volumeleaders-agent report top-100-rank`

Copy-paste examples:
volumeleaders-agent report top-100-rank
volumeleaders-agent fields report top-100-rank
volumeleaders-agent report top-100-rank --days 5 --fields FullTimeString24,Price,Dollars,DollarsMultiplier,TradeRank

5. Support and resistance context
Recommended first command: `volumeleaders-agent trade dashboard NVDA`

Copy-paste examples:
volumeleaders-agent trade dashboard NVDA
volumeleaders-agent trade levels NVDA
volumeleaders-agent fields trade levels
volumeleaders-agent trade levels NVDA --trade-level-count 10 --fields Ticker,TradeLevelRank,Price,TradeLevelTouches,Dollars

6. Sudden activity bursts
Recommended first command: `volumeleaders-agent trade cluster-bombs NVDA`

Copy-paste examples:
volumeleaders-agent trade cluster-bombs NVDA
volumeleaders-agent fields trade cluster-bombs
volumeleaders-agent trade cluster-bombs NVDA --fields Ticker,TradeClusterBombRank,TradeCount,Dollars,window

7. Repeated activity near a price
Recommended first command: `volumeleaders-agent trade clusters NVDA`

Copy-paste examples:
volumeleaders-agent trade clusters NVDA
volumeleaders-agent trade levels NVDA
volumeleaders-agent fields trade clusters
volumeleaders-agent trade clusters NVDA --fields Ticker,TradeClusterRank,Price,TradeCount,Dollars,window

8. Leveraged ETF sentiment
Recommended first command: `volumeleaders-agent trade sentiment`

Copy-paste examples:
volumeleaders-agent trade sentiment
volumeleaders-agent trade sentiment --start-date 2026-05-01 --end-date 2026-05-27
volumeleaders-agent help schema

9. Earnings with institutional context
Recommended first command: `volumeleaders-agent market earnings`

Copy-paste examples:
volumeleaders-agent market earnings
volumeleaders-agent market earnings --start-date 2026-05-01 --end-date 2026-05-27
volumeleaders-agent trade dashboard NVDA

10. Check volume leaderboards
Recommended first command: `volumeleaders-agent volume institutional`

Copy-paste examples:
volumeleaders-agent volume institutional
volumeleaders-agent fields volume institutional
volumeleaders-agent volume institutional --date 2026-05-27 --limit 50 --fields Ticker,Dollars

11. Inspect or plan mutating alert and watchlist changes
Recommended first command: `volumeleaders-agent alert configs`

Copy-paste examples:
volumeleaders-agent alert configs
volumeleaders-agent watchlist configs
volumeleaders-agent schema | jq '.commands[] | select(.mutating == true) | {path: .preferred_path, supports_dry_run, requires_confirmation}'
volumeleaders-agent alert create --name BigTechSweeps --tickers AAPL,MSFT --dry-run
volumeleaders-agent watchlist delete --key 123 --dry-run
"#;

#[cfg(test)]
mod tests {
    use std::io;

    use crate::cli::HelpTopic;
    use crate::cli::output::finish_output;

    use super::topic_text;

    #[test]
    fn topics_include_expected_operational_content() {
        assert!(topic_text(HelpTopic::Auth).contains("doctor"));
        assert!(topic_text(HelpTopic::Agent).contains("non-interactive automation"));
        assert!(topic_text(HelpTopic::Agent).contains("commands --grouped"));
        assert!(topic_text(HelpTopic::Agent).contains("fields trade list"));
        assert!(topic_text(HelpTopic::Agent).contains("external `jq` handles reshaping"));
        assert!(topic_text(HelpTopic::Agent).contains("--strict-empty trades NVDA"));
        assert!(topic_text(HelpTopic::Agent).contains("--fields Ticker,Dollars | jq"));
        assert!(topic_text(HelpTopic::Environment).contains("XDG directories"));
        assert!(topic_text(HelpTopic::Auth).contains("auth.actions"));
        assert!(topic_text(HelpTopic::ExitCodes).contains("3  auth error"));
        assert!(topic_text(HelpTopic::ExitCodes).contains("unknown --fields names"));
        assert!(topic_text(HelpTopic::ExitCodes).contains("-vvv"));
        assert!(topic_text(HelpTopic::Schema).contains("commands --grouped"));
        assert!(topic_text(HelpTopic::Schema).contains("fields volume institutional"));
        assert!(topic_text(HelpTopic::Schema).contains("exact case-sensitive field name"));
        assert!(topic_text(HelpTopic::Schema).contains("external `jq`"));
        assert!(topic_text(HelpTopic::Schema).contains("trades"));
        assert!(topic_text(HelpTopic::Examples).contains("trades NVDA"));
        assert!(topic_text(HelpTopic::Examples).contains("fields trade list"));
        assert!(topic_text(HelpTopic::Examples).contains("--fields Ticker,Dollars | jq"));
        assert!(topic_text(HelpTopic::Examples).contains("--strict-empty trade list NVDA"));
        assert!(topic_text(HelpTopic::Examples).contains("-vv trade list NVDA"));
        assert!(topic_text(HelpTopic::Workflows).contains("Start with defaults first"));
        assert!(
            topic_text(HelpTopic::Workflows).contains("Institutional prints show unusual activity")
        );
        assert!(topic_text(HelpTopic::Workflows).contains("fields <command path>"));
        assert!(topic_text(HelpTopic::Workflows).contains("Recommended first command"));
        assert!(topic_text(HelpTopic::Workflows).contains("trade dashboard NVDA"));
        assert!(topic_text(HelpTopic::Workflows).contains("report top-100-rank"));
        assert!(topic_text(HelpTopic::Workflows).contains("trade cluster-bombs"));
        assert!(topic_text(HelpTopic::Workflows).contains("trade sentiment"));
        assert!(topic_text(HelpTopic::Workflows).contains("market earnings"));
        assert!(topic_text(HelpTopic::Workflows).contains("alert configs"));
    }

    #[test]
    fn every_topic_has_trailing_newline() {
        for topic in [
            HelpTopic::Agent,
            HelpTopic::Auth,
            HelpTopic::Environment,
            HelpTopic::ExitCodes,
            HelpTopic::Schema,
            HelpTopic::Examples,
            HelpTopic::Workflows,
        ] {
            assert!(topic_text(topic).ends_with('\n'));
        }
    }

    #[test]
    fn write_errors_map_to_json_exit_code() {
        assert!(finish_output(Err(io::Error::other("stdout closed"))).is_err());
    }
}
