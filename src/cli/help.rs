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
- stderr is reserved for diagnostics and structured runtime errors.

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
7  reserved for strict empty-result handling

Runtime errors are written to stderr as one compact JSON object:
{"ok":false,"error":{"kind":"auth_error","message":"browser cookies are missing or expired"}}

Recovery guidance:
- Exit 2: check `--help`, `commands`, `schema`, or this help topic.
- Exit 3: run `doctor`, then log in to VolumeLeaders again if needed.
- Exit 4 or 5: retry later or narrow the request.
- Exit 6: check field names and JSON-processing pipeline assumptions.
"#;

const SCHEMA_HELP: &str = r#"schema

Use `schema` and `commands` for binary-native CLI discovery.

`volumeleaders-agent schema` emits compact JSON generated from the live clap tree. It includes the binary version, auth model, leaf command paths, aliases, auth requirements, help text, and argument metadata.

`volumeleaders-agent commands` emits one sorted leaf command path per line. It is lighter than `schema` and useful when an agent only needs to choose a command.

`volumeleaders-agent commands --grouped` emits top-level groups with short descriptions for each leaf command.

Useful discovery commands:
- volumeleaders-agent commands
- volumeleaders-agent commands --grouped
- volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == "trade list")'
"#;

const EXAMPLES_HELP: &str = r#"examples

High-value command examples:

volumeleaders-agent doctor
volumeleaders-agent commands
volumeleaders-agent commands --grouped
volumeleaders-agent schema | jq '.commands[] | select(.preferred_path == "trade list")'

volumeleaders-agent report list
volumeleaders-agent report dark-pool-sweeps
volumeleaders-agent trade list --ticker NVDA
volumeleaders-agent trade list --ticker NVDA --from 2026-05-01 --to 2026-05-27 --fields ticker,date,price,volume,venue
volumeleaders-agent trade dashboard --ticker NVDA

volumeleaders-agent volume institutional --ticker AAPL
volumeleaders-agent volume institutional --from 2026-05-01 --to 2026-05-27 --limit 50
volumeleaders-agent market earnings --ticker AAPL
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
        assert!(topic_text(HelpTopic::Environment).contains("browser profiles"));
        assert!(topic_text(HelpTopic::ExitCodes).contains("3  auth error"));
        assert!(topic_text(HelpTopic::Schema).contains("commands --grouped"));
        assert!(topic_text(HelpTopic::Examples).contains("trade list --ticker NVDA"));
    }

    #[test]
    fn every_topic_has_trailing_newline() {
        for topic in [
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
