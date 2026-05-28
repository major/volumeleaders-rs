//! Built-in operational help topics.

use std::io::{self, Write};

use crate::cli::output::finish_output;
use crate::cli::{HelpArgs, HelpTopic};

/// Emit a built-in operational help topic as plain text.
pub fn handle(args: &HelpArgs) -> i32 {
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
    }
}

fn write_text(output: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(output.as_bytes())
}

const AGENT_HELP: &str = r#"agent

Guidance for non-interactive automation and coding agents:

1. Check local readiness first.
   Run `volumeleaders-agent doctor` before authenticated data commands when automation needs to confirm browser-cookie readiness. `doctor` is local-only by default and does not make a network request.

2. Discover commands before guessing.
   Use `volumeleaders-agent commands` for a quick leaf-command list, `volumeleaders-agent commands --grouped` for grouped descriptions, `volumeleaders-agent schema` for machine-readable command, alias, auth, help, and stable argument metadata, and `volumeleaders-agent fields <command path>` for field projection metadata.

3. Keep streams separate.
   Successful data commands write compact JSON to stdout. Discovery and help commands write plain text to stdout. Diagnostics, verbosity logs from `-v`/`-vv`/`-vvv`, and structured runtime errors go to stderr.

4. Shape output deliberately.
   Use `volumeleaders-agent fields trade list` to discover exact case-sensitive field names before passing command-specific `--fields` or `--all-fields`, and pipe stdout to external `jq` for inspection or transformations.

5. Treat empty rows and mutations carefully.
   Use global `--strict-empty` when an empty record array should fail automation with exit code 7. Avoid mutating alert and watchlist commands unless the user explicitly requested the mutation.

6. Recover by exit code.
   Exit 2 means usage or argument validation failed. Exit 3 means browser auth failed. Exit 4 means HTTP transport failed. Exit 5 means the API returned an error. Exit 6 means JSON parsing or output transformation failed. Exit 7 means `--strict-empty` rejected an empty record array.

Copy-paste examples:

volumeleaders-agent doctor
volumeleaders-agent commands --grouped
volumeleaders-agent fields trade list
volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == "trade list")'
volumeleaders-agent help exit-codes
volumeleaders-agent --strict-empty trades NVDA
volumeleaders-agent trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields Ticker,DateTime,Price,Dollars
volumeleaders-agent volume institutional --date 2026-05-27 --tickers AAPL,NVDA --limit 50 | jq '.[] | {ticker: .Ticker, dollars: .Dollars}'
"#;

const AUTH_HELP: &str = r#"auth

VolumeLeaders live-data commands authenticate with browser cookies from a local Chrome or Firefox profile. Log in at https://www.volumeleaders.com in a browser first, then rerun the CLI.

The CLI needs the ASP.NET session cookie, forms auth cookie, and request verification cookie. Cookie values and XSRF tokens are never printed.

Run `volumeleaders-agent doctor` for a safe local readiness check. It does not make a network request by default and exits 0 when the local browser-cookie session looks usable, or 3 when auth is missing or invalid.

Common fixes:
- Log in to VolumeLeaders again and retry.
- Use the same OS user that owns the browser profile.
- Close browser profile locks if your platform requires it.
- Check `help environment` for local profile expectations.
"#;

const ENVIRONMENT_HELP: &str = r#"environment

The CLI reads local browser profiles through the operating system account running `volumeleaders-agent`. It currently has no config file or environment-variable precedence layer.

Expected setup:
- Rust users can run the CLI with `cargo run -- ...`; installed users run `volumeleaders-agent ...`.
- Browser-cookie auth expects a logged-in Chrome or Firefox profile for https://www.volumeleaders.com.
- stdout is reserved for command data, either compact JSON for data commands or plain text for discovery/help commands.
- stderr is reserved for diagnostics and structured runtime errors. Use `-v`, `-vv`, or `-vvv` to enable info, debug, or trace diagnostics without changing stdout.

Use `volumeleaders-agent doctor` before live data commands when automation needs to confirm local auth readiness without spending API quota.
"#;

const EXIT_CODES_HELP: &str = r#"exit-codes

Semantic exit codes are stable for automation:

0  success
2  usage error, invalid arguments, or clap validation failure
3  auth error, browser cookies are missing, expired, or invalid
4  HTTP transport error
5  VolumeLeaders API error response
6  JSON parse or output transformation error
7  strict empty-result handling requested with --strict-empty

Runtime errors are written to stderr as one compact JSON object:
{"ok":false,"error":{"kind":"auth_error","message":"browser cookies are missing or expired"}}

Recovery guidance:
- Exit 2: check `--help`, `commands`, `schema`, or this help topic.
- Exit 3: run `doctor`, then log in to VolumeLeaders again if needed.
- Exit 4 or 5: retry later or narrow the request.
- Exit 6: check field names and JSON-processing pipeline assumptions.
- Exit 7: check the ticker, widen the date range, relax filters, or accept that no configured rows may be valid account state.

Verbosity: no `-v` logs warnings and errors, `-v` enables info diagnostics, `-vv` enables debug diagnostics, and `-vvv` enables trace diagnostics. Diagnostic logs always go to stderr.
"#;

const SCHEMA_HELP: &str = r#"schema

Use `schema` and `commands` for binary-native CLI discovery.

`volumeleaders-agent schema` emits compact JSON generated from the live clap tree. It includes the binary version, auth model, leaf command paths, explicit alias metadata, auth requirements, help text, and argument metadata with stable names.

`volumeleaders-agent fields <command path>` emits compact JSON with output fields accepted by `--fields`, including each exact field name, short description, and type hint. It does not need live API rows.

Common top-level aliases such as `trades`, `dashboard`, and `levels` are reported with their canonical `trade ...` preferred paths. Alias entries set `is_alias` and `alias_for`; canonical entries list their aliases so agents can normalize generated commands.

`volumeleaders-agent commands` emits one sorted leaf command path per line. It is lighter than `schema` and useful when an agent only needs to choose a command.

`volumeleaders-agent commands --grouped` emits top-level groups with short descriptions for each leaf command.

Useful discovery commands:
- volumeleaders-agent commands
- volumeleaders-agent commands --grouped
- volumeleaders-agent fields trade list
- volumeleaders-agent fields volume institutional
- volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == "trade list")'
"#;

const EXAMPLES_HELP: &str = r#"examples

High-value command examples:

volumeleaders-agent doctor
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
volumeleaders-agent --strict-empty trade list NVDA --start-date 2026-05-01 --end-date 2026-05-27 --fields Ticker,DateTime,Price,Dollars
volumeleaders-agent dashboard NVDA
volumeleaders-agent trade dashboard NVDA
volumeleaders-agent levels NVDA

volumeleaders-agent volume institutional --date 2026-05-27 --tickers AAPL
volumeleaders-agent volume institutional --date 2026-05-27 --limit 50
volumeleaders-agent market earnings --start-date 2026-05-01 --end-date 2026-05-27
volumeleaders-agent watchlist tickers --watchlist-key 123
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
        assert!(topic_text(HelpTopic::Agent).contains("--strict-empty trades NVDA"));
        assert!(topic_text(HelpTopic::Environment).contains("browser profiles"));
        assert!(topic_text(HelpTopic::ExitCodes).contains("3  auth error"));
        assert!(topic_text(HelpTopic::ExitCodes).contains("-vvv"));
        assert!(topic_text(HelpTopic::Schema).contains("commands --grouped"));
        assert!(topic_text(HelpTopic::Schema).contains("fields volume institutional"));
        assert!(topic_text(HelpTopic::Schema).contains("trades"));
        assert!(topic_text(HelpTopic::Examples).contains("trades NVDA"));
        assert!(topic_text(HelpTopic::Examples).contains("fields trade list"));
        assert!(topic_text(HelpTopic::Examples).contains("--strict-empty trade list NVDA"));
        assert!(topic_text(HelpTopic::Examples).contains("-vv trade list NVDA"));
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
        ] {
            assert!(topic_text(topic).ends_with('\n'));
        }
    }

    #[test]
    fn write_errors_map_to_json_exit_code() {
        assert_eq!(finish_output(Err(io::Error::other("stdout closed"))), 6);
    }
}
