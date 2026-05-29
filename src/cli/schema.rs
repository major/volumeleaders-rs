//! Machine-readable schema generated from the live clap command tree.

use clap::{Arg, ArgAction, Command, CommandFactory};
use serde::Serialize;

use crate::cli::Cli;
use crate::cli::command_examples::{CommandExample, examples_for_path};
use crate::cli::output::{finish_output, print_json};

const SCHEMA_VERSION: u8 = 1;
const BINARY_NAME: &str = "volumeleaders-agent";

/// Emit the CLI schema as compact JSON.
pub fn handle() -> i32 {
    finish_output(print_json(&build_schema()))
}

#[derive(Debug, Serialize)]
struct CliSchema {
    schema_version: u8,
    binary: &'static str,
    version: &'static str,
    auth: AuthSchema,
    commands: Vec<CommandSchema>,
}

#[derive(Debug, Serialize)]
struct AuthSchema {
    kind: &'static str,
    sources: [&'static str; 2],
    network_required_for_doctor: bool,
}

#[derive(Debug, Serialize)]
struct CommandSchema {
    path: Vec<String>,
    is_alias: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias_for: Option<Vec<String>>,
    preferred_path: String,
    aliases: Vec<String>,
    auth_required: bool,
    mutating: bool,
    supports_dry_run: bool,
    requires_confirmation: bool,
    about: Option<String>,
    long_about: Option<String>,
    examples: Vec<CommandExample>,
    args: Vec<ArgSchema>,
}

#[derive(Clone, Debug, Serialize)]
struct ArgSchema {
    name: String,
    long: Option<String>,
    short: Option<char>,
    value_name: Option<String>,
    kind: ArgKind,
    required: bool,
    default: Option<Vec<String>>,
    parser: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separators: Option<Vec<&'static str>>,
    possible_values: Vec<String>,
    multi_value: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ArgKind {
    Flag,
    Option,
    Positional,
}

fn build_schema() -> CliSchema {
    let command = Cli::command();
    let global_args: Vec<ArgSchema> = command
        .get_arguments()
        .filter(|arg| !arg.is_hide_set())
        .map(arg_schema)
        .collect();
    let mut commands = Vec::new();

    collect_leaf_commands(&command, &mut Vec::new(), &global_args, &mut commands);
    add_alias_commands(&mut commands);
    commands.sort_by(|left, right| {
        left.preferred_path
            .cmp(&right.preferred_path)
            .then_with(|| left.path.cmp(&right.path))
    });

    CliSchema {
        schema_version: SCHEMA_VERSION,
        binary: BINARY_NAME,
        version: env!("CARGO_PKG_VERSION"),
        auth: AuthSchema {
            kind: "browser_cookies",
            sources: ["chrome", "firefox"],
            network_required_for_doctor: false,
        },
        commands,
    }
}

fn collect_leaf_commands(
    command: &Command,
    path: &mut Vec<String>,
    global_args: &[ArgSchema],
    commands: &mut Vec<CommandSchema>,
) {
    for subcommand in command
        .get_subcommands()
        .filter(|command| !command.is_hide_set())
    {
        path.push(subcommand.get_name().to_string());
        if subcommand.has_subcommands() {
            collect_leaf_commands(subcommand, path, global_args, commands);
        } else {
            commands.push(command_schema(subcommand, path, global_args));
        }
        path.pop();
    }
}

fn command_schema(command: &Command, path: &[String], global_args: &[ArgSchema]) -> CommandSchema {
    let mut args = global_args.to_vec();
    args.extend(
        command
            .get_arguments()
            .filter(|arg| !arg.is_hide_set())
            .map(arg_schema),
    );

    CommandSchema {
        path: path.to_vec(),
        is_alias: false,
        alias_for: None,
        preferred_path: path.join(" "),
        aliases: aliases_for_path(path),
        auth_required: auth_required(path),
        mutating: is_mutating(path),
        supports_dry_run: supports_dry_run(path),
        requires_confirmation: requires_confirmation(path),
        about: command.get_about().map(ToString::to_string),
        long_about: command.get_long_about().map(ToString::to_string),
        examples: examples_for_path(path).to_vec(),
        args,
    }
}

fn add_alias_commands(commands: &mut Vec<CommandSchema>) {
    for (alias, canonical) in top_level_aliases() {
        let canonical_path = canonical.map(str::to_string).to_vec();
        let Some(canonical_command) = commands
            .iter()
            .find(|command| command.path == canonical_path)
        else {
            continue;
        };

        commands.push(CommandSchema {
            path: vec![alias.to_string()],
            is_alias: true,
            alias_for: Some(canonical_path),
            preferred_path: canonical_command.preferred_path.clone(),
            aliases: Vec::new(),
            auth_required: canonical_command.auth_required,
            mutating: canonical_command.mutating,
            supports_dry_run: canonical_command.supports_dry_run,
            requires_confirmation: canonical_command.requires_confirmation,
            about: Some(format!("Alias for `{}`.", canonical_command.preferred_path)),
            long_about: canonical_command.long_about.clone(),
            examples: canonical_command.examples.clone(),
            args: canonical_command.args.clone(),
        });
    }
}

fn aliases_for_path(path: &[String]) -> Vec<String> {
    top_level_aliases()
        .into_iter()
        .filter(|(_, canonical)| {
            canonical.len() == path.len()
                && canonical
                    .iter()
                    .zip(path)
                    .all(|(left, right)| *left == right)
        })
        .map(|(alias, _)| alias.to_string())
        .collect()
}

fn top_level_aliases() -> [(&'static str, [&'static str; 2]); 3] {
    [
        ("trades", ["trade", "list"]),
        ("dashboard", ["trade", "dashboard"]),
        ("levels", ["trade", "levels"]),
    ]
}

fn auth_required(path: &[String]) -> bool {
    !matches!(path, [command] if command == "commands" || command == "doctor" || command == "fields" || command == "help" || command == "schema" || command == "completions")
}

fn is_mutating(path: &[String]) -> bool {
    matches!(
        path,
        [group, command]
            if (group == "alert" && matches!(command.as_str(), "create" | "edit" | "delete"))
                || (group == "watchlist"
                    && matches!(command.as_str(), "create" | "edit" | "delete" | "add-ticker"))
    )
}

fn supports_dry_run(path: &[String]) -> bool {
    is_mutating(path)
}

fn requires_confirmation(path: &[String]) -> bool {
    matches!(
        path,
        [group, command]
            if matches!(group.as_str(), "alert" | "watchlist") && command == "delete"
    )
}

fn arg_schema(arg: &Arg) -> ArgSchema {
    let name = arg_name(arg);
    let parser = parser_name(&name, arg);
    let possible_values = discoverable_possible_values(&name, arg);

    ArgSchema {
        semantic_type: semantic_type(&name, arg),
        format: semantic_format(&name),
        separators: semantic_separators(&name),
        name,
        long: arg.get_long().map(ToString::to_string),
        short: arg.get_short(),
        value_name: value_name(arg),
        kind: arg_kind(arg),
        required: arg.is_required_set(),
        default: default_values(arg),
        parser,
        possible_values,
        multi_value: multi_value(arg),
    }
}

fn semantic_type(name: &str, arg: &Arg) -> Option<&'static str> {
    if is_date_arg(name) {
        return Some("date");
    }
    if name == "days" {
        return Some("lookback-days");
    }
    if is_ticker_arg(name) {
        return Some("ticker-list");
    }
    if is_money_arg(name) {
        return Some("money");
    }
    if is_boolean_filter_arg(name, arg) {
        return Some("boolean-filter");
    }
    if !possible_values(arg).is_empty() {
        return Some("enum");
    }
    if is_integer_arg(name) {
        return Some("integer");
    }
    if is_number_arg(name) {
        return Some("number");
    }

    None
}

fn semantic_format(name: &str) -> Option<&'static str> {
    is_date_arg(name).then_some("YYYY-MM-DD")
}

fn semantic_separators(name: &str) -> Option<Vec<&'static str>> {
    is_ticker_arg(name).then_some(vec![",", " "])
}

fn is_date_arg(name: &str) -> bool {
    matches!(name, "date" | "start-date" | "end-date")
}

fn is_ticker_arg(name: &str) -> bool {
    matches!(name, "ticker" | "tickers")
}

fn is_money_arg(name: &str) -> bool {
    name.contains("dollars") || name.contains("money")
}

fn is_boolean_filter_arg(name: &str, _arg: &Arg) -> bool {
    matches!(
        name,
        "dark-pools"
            | "dark-pool"
            | "sweep"
            | "sweeps"
            | "late-prints"
            | "sig-prints"
            | "signature-prints"
            | "normal-prints"
            | "timely-prints"
            | "lit-exchanges"
            | "blocks"
            | "premarket"
            | "premarket-trades"
            | "rth"
            | "rth-trades"
            | "ah"
            | "ah-trades"
            | "opening"
            | "opening-trades"
            | "closing"
            | "closing-trades"
            | "phantom"
            | "phantom-print"
            | "phantom-trades"
            | "offsetting"
            | "offsetting-print"
            | "offsetting-trades"
            | "even-shared"
    )
}

fn is_integer_arg(name: &str) -> bool {
    name == "limit"
        || name == "length"
        || name == "start"
        || name.ends_with("count")
        || name.contains("volume")
        || name.contains("rank")
        || name.contains("order-column")
        || name.contains("security-type")
}

fn is_number_arg(name: &str) -> bool {
    name.contains("price")
        || name.contains("vcd")
        || name.contains("multiplier")
        || name.contains("relative-size")
        || name.contains("rsi")
        || name.contains("market-cap")
}

fn arg_name(arg: &Arg) -> String {
    arg.get_long()
        .map(ToString::to_string)
        .unwrap_or_else(|| arg.get_id().to_string())
}

fn value_name(arg: &Arg) -> Option<String> {
    arg.get_value_names()
        .and_then(|names| names.first())
        .map(ToString::to_string)
}

fn arg_kind(arg: &Arg) -> ArgKind {
    if arg.get_long().is_none() && arg.get_short().is_none() {
        return ArgKind::Positional;
    }

    match arg.get_action() {
        ArgAction::SetTrue
        | ArgAction::SetFalse
        | ArgAction::Count
        | ArgAction::Help
        | ArgAction::Version => ArgKind::Flag,
        _ => ArgKind::Option,
    }
}

fn default_values(arg: &Arg) -> Option<Vec<String>> {
    let values: Vec<String> = arg
        .get_default_values()
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();

    (!values.is_empty()).then_some(values)
}

fn parser_name(name: &str, arg: &Arg) -> String {
    if (is_bool_value_parser(arg) && is_boolean_filter_arg(name, arg))
        || is_bool_filter_flag(name, arg)
    {
        return "bool".to_string();
    }

    if !possible_values(arg).is_empty() {
        return "enum".to_string();
    }

    match arg.get_action() {
        ArgAction::SetTrue | ArgAction::SetFalse => "bool",
        ArgAction::Count => "count",
        _ => "string",
    }
    .to_string()
}

fn is_bool_value_parser(arg: &Arg) -> bool {
    let values = possible_values(arg);
    matches!(arg.get_action(), ArgAction::Set)
        && values.len() == 2
        && values.iter().any(|value| value == "true")
        && values.iter().any(|value| value == "false")
}

fn is_bool_filter_flag(name: &str, arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::SetTrue | ArgAction::SetFalse)
        && is_boolean_filter_arg(name, arg)
}

fn possible_values(arg: &Arg) -> Vec<String> {
    arg.get_value_parser()
        .possible_values()
        .map(|values| values.map(|value| value.get_name().to_string()).collect())
        .unwrap_or_default()
}

fn discoverable_possible_values(name: &str, arg: &Arg) -> Vec<String> {
    let values = possible_values(arg);
    if !values.is_empty() {
        return values;
    }

    custom_validation_possible_values(name)
}

fn custom_validation_possible_values(name: &str) -> Vec<String> {
    match name {
        "trade-level-count" => ["5", "10", "20", "50"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn multi_value(arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::Append | ArgAction::Count)
        || arg
            .get_num_args()
            .is_some_and(|range| range.max_values() > 1)
}

#[cfg(test)]
mod tests {
    use clap::{Arg, ArgAction, Command, CommandFactory};
    use serde_json::Value;

    use crate::cli::Cli;

    use crate::cli::command_examples::examples_for_path;

    use super::{CommandSchema, add_alias_commands, build_schema, parser_name};

    fn schema_value() -> Value {
        serde_json::to_value(build_schema()).unwrap()
    }

    #[test]
    fn schema_includes_root_contract() {
        let schema = schema_value();

        assert_eq!(schema["schema_version"], 1);
        assert_eq!(schema["binary"], "volumeleaders-agent");
        assert_eq!(schema["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(schema["auth"]["kind"], "browser_cookies");
        assert_eq!(
            schema["auth"]["sources"],
            serde_json::json!(["chrome", "firefox"])
        );
    }

    #[test]
    fn schema_includes_leaf_commands_from_clap_tree() {
        let schema = schema_value();
        let commands = schema["commands"].as_array().unwrap();
        let paths: Vec<_> = commands
            .iter()
            .map(|command| command["preferred_path"].as_str().unwrap())
            .collect();

        assert!(paths.contains(&"schema"));
        assert!(paths.contains(&"commands"));
        assert!(paths.contains(&"doctor"));
        assert!(paths.contains(&"fields"));
        assert!(paths.contains(&"help"));
        assert!(paths.contains(&"trade list"));
        assert!(paths.contains(&"trade dashboard"));
        assert!(paths.contains(&"trade levels"));
        assert!(paths.contains(&"volume institutional"));
        assert!(paths.contains(&"market earnings"));
        assert!(paths.contains(&"watchlist configs"));
        assert!(paths.contains(&"alert configs"));
    }

    #[test]
    fn schema_paths_match_live_clap_leaf_commands_and_aliases() {
        let schema = schema_value();
        let mut schema_paths = schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|command| string_array(&command["path"]))
            .collect::<Vec<_>>();
        schema_paths.sort();

        let mut expected_paths = live_leaf_paths();
        expected_paths.extend([
            vec!["dashboard".to_string()],
            vec!["levels".to_string()],
            vec!["trades".to_string()],
        ]);
        expected_paths.sort();

        assert_eq!(schema_paths, expected_paths);
    }

    #[test]
    fn schema_includes_global_flags_on_every_command() {
        let schema = schema_value();
        let commands = schema["commands"].as_array().unwrap();

        for command in commands {
            let args = command["args"].as_array().unwrap();
            assert!(args.iter().any(|arg| arg["long"] == "strict-empty"
                && arg["kind"] == "flag"
                && arg["parser"] == "enum"));
            assert!(args.iter().any(|arg| arg["long"] == "verbose"
                && arg["short"] == "v"
                && arg["kind"] == "flag"
                && arg["parser"] == "count"));
        }
    }

    #[test]
    fn schema_marks_local_discovery_commands_as_auth_free() {
        let schema = schema_value();
        let commands = schema["commands"].as_array().unwrap();

        let schema_command = commands
            .iter()
            .find(|command| command["preferred_path"] == "schema")
            .unwrap();
        let doctor_command = commands
            .iter()
            .find(|command| command["preferred_path"] == "doctor")
            .unwrap();
        let commands_command = commands
            .iter()
            .find(|command| command["preferred_path"] == "commands")
            .unwrap();
        let help_command = commands
            .iter()
            .find(|command| command["preferred_path"] == "help")
            .unwrap();
        let fields_command = commands
            .iter()
            .find(|command| command["preferred_path"] == "fields")
            .unwrap();
        let trade_command = commands
            .iter()
            .find(|command| command["preferred_path"] == "trade list")
            .unwrap();

        assert_eq!(schema_command["auth_required"], false);
        assert_eq!(doctor_command["auth_required"], false);
        assert_eq!(commands_command["auth_required"], false);
        assert_eq!(fields_command["auth_required"], false);
        assert_eq!(help_command["auth_required"], false);
        assert_eq!(trade_command["auth_required"], true);
    }

    #[test]
    fn schema_includes_top_level_alias_metadata() {
        let schema = schema_value();
        let commands = schema["commands"].as_array().unwrap();

        let trades_alias = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(["trades"]))
            .unwrap();
        let dashboard_alias = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(["dashboard"]))
            .unwrap();
        let levels_alias = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(["levels"]))
            .unwrap();

        assert_eq!(
            trades_alias["alias_for"],
            serde_json::json!(["trade", "list"])
        );
        assert_eq!(trades_alias["preferred_path"], "trade list");
        assert_eq!(trades_alias["auth_required"], true);
        assert_eq!(trades_alias["is_alias"], true);
        assert_eq!(
            dashboard_alias["alias_for"],
            serde_json::json!(["trade", "dashboard"])
        );
        assert_eq!(dashboard_alias["preferred_path"], "trade dashboard");
        assert_eq!(dashboard_alias["is_alias"], true);
        assert_eq!(
            levels_alias["alias_for"],
            serde_json::json!(["trade", "levels"])
        );
        assert_eq!(levels_alias["preferred_path"], "trade levels");
        assert_eq!(levels_alias["is_alias"], true);

        let trade_list = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(["trade", "list"]))
            .unwrap();
        let trade_dashboard = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(["trade", "dashboard"]))
            .unwrap();
        let trade_levels = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(["trade", "levels"]))
            .unwrap();

        assert_eq!(trade_list["is_alias"], false);
        assert_eq!(trade_list["aliases"], serde_json::json!(["trades"]));
        assert!(trade_list["alias_for"].is_null());
        assert_eq!(trade_dashboard["is_alias"], false);
        assert_eq!(trade_dashboard["aliases"], serde_json::json!(["dashboard"]));
        assert!(trade_dashboard["alias_for"].is_null());
        assert_eq!(trade_levels["is_alias"], false);
        assert_eq!(trade_levels["aliases"], serde_json::json!(["levels"]));
        assert!(trade_levels["alias_for"].is_null());
    }

    #[test]
    fn alias_generation_skips_missing_canonical_commands() {
        let mut commands = vec![CommandSchema {
            path: vec!["trade".to_string(), "list".to_string()],
            is_alias: false,
            alias_for: None,
            preferred_path: "trade list".to_string(),
            aliases: vec!["trades".to_string()],
            auth_required: true,
            mutating: false,
            supports_dry_run: false,
            requires_confirmation: false,
            about: Some("List trades".to_string()),
            long_about: Some("List trades.\n\nExamples:\n  volumeleaders-agent trade list NVDA\n  volumeleaders-agent trade list AAPL".to_string()),
            examples: examples_for_path(&["trade".to_string(), "list".to_string()]).to_vec(),
            args: Vec::new(),
        }];

        add_alias_commands(&mut commands);

        assert!(commands.iter().any(|command| command.path == ["trades"]));
        assert!(
            commands
                .iter()
                .all(|command| command.path != ["dashboard"] && command.path != ["levels"])
        );
    }

    #[test]
    fn schema_includes_argument_metadata() {
        let schema = schema_value();
        let trade_list = schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|command| command["path"] == serde_json::json!(["trade", "list"]))
            .unwrap();
        let args = trade_list["args"].as_array().unwrap();

        assert!(
            args.iter()
                .any(|arg| arg["name"] == "fields" && arg["long"] == "fields")
        );
        assert!(args.iter().any(|arg| arg["name"] == "all-fields"
            && arg["long"] == "all-fields"
            && arg["kind"] == "flag"));
        assert!(args.iter().any(|arg| arg["name"] == "limit"
            && arg["long"] == "limit"
            && arg["semantic_type"] == "integer"
            && arg["default"] == serde_json::json!(["1000"])));
        assert!(args.iter().any(|arg| arg["name"] == "tickers"
            && arg["kind"] == "positional"
            && arg["semantic_type"] == "ticker-list"
            && arg["separators"] == serde_json::json!([",", " "])
            && arg["value_name"] == "TICKERS"
            && arg["multi_value"] == true));
        assert!(args.iter().any(|arg| {
            arg["name"] == "strict-empty"
                && arg["long"] == "strict-empty"
                && arg["kind"] == "flag"
                && arg["parser"] == "enum"
        }));
        assert!(args.iter().any(|arg| {
            arg["name"] == "verbose"
                && arg["long"] == "verbose"
                && arg["short"] == "v"
                && arg["kind"] == "flag"
                && arg["parser"] == "count"
                && arg["multi_value"] == true
        }));
    }

    #[test]
    fn schema_includes_structured_examples_for_required_commands() {
        let schema = schema_value();
        let commands = schema["commands"].as_array().unwrap();

        for preferred_path in [
            "doctor",
            "commands",
            "schema",
            "trade list",
            "trade dashboard",
            "trade levels",
            "report list",
            "report dark-pool-sweeps",
            "volume institutional",
            "market earnings",
            "watchlist tickers",
        ] {
            let command = commands
                .iter()
                .find(|command| command["preferred_path"] == preferred_path)
                .unwrap_or_else(|| panic!("missing command {preferred_path}"));
            let examples = command["examples"].as_array().unwrap();

            assert!(
                examples.len() >= 2,
                "{preferred_path} should have at least two structured examples"
            );
            for example in examples {
                assert!(
                    example["description"]
                        .as_str()
                        .is_some_and(|value| !value.is_empty()),
                    "{preferred_path} example should include a description"
                );
                assert!(
                    example["command"]
                        .as_str()
                        .is_some_and(|value| value.starts_with("volumeleaders-agent ")),
                    "{preferred_path} example should include a volumeleaders-agent command"
                );
            }
        }
    }

    #[test]
    fn schema_copies_structured_examples_to_aliases() {
        let schema = schema_value();
        let commands = schema["commands"].as_array().unwrap();
        let trade_list = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(["trade", "list"]))
            .unwrap();
        let trades_alias = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(["trades"]))
            .unwrap();

        assert_eq!(trades_alias["examples"], trade_list["examples"]);
    }

    #[test]
    fn schema_positionals_have_stable_names() {
        let schema = schema_value();
        let commands = schema["commands"].as_array().unwrap();

        for (path, expected_name, expected_value_name, expected_required, expected_multi_value) in [
            (
                serde_json::json!(["trade", "list"]),
                "tickers",
                "TICKERS",
                false,
                true,
            ),
            (
                serde_json::json!(["trade", "dashboard"]),
                "ticker",
                "TICKER",
                true,
                false,
            ),
            (serde_json::json!(["help"]), "topic", "TOPIC", true, false),
            (
                serde_json::json!(["completions"]),
                "shell",
                "SHELL",
                true,
                false,
            ),
        ] {
            let command = commands
                .iter()
                .find(|command| command["path"] == path)
                .unwrap();
            let positional = command["args"]
                .as_array()
                .unwrap()
                .iter()
                .find(|arg| arg["kind"] == "positional")
                .unwrap();

            assert_eq!(positional["name"], expected_name);
            assert_eq!(positional["value_name"], expected_value_name);
            assert_eq!(positional["required"], expected_required);
            assert_eq!(positional["multi_value"], expected_multi_value);
        }
    }

    #[test]
    fn schema_includes_semantic_argument_metadata() {
        let schema = schema_value();
        let commands = schema["commands"].as_array().unwrap();

        assert_arg_semantics(
            commands,
            &["trade", "list"],
            "start-date",
            "date",
            Some("YYYY-MM-DD"),
            None,
        );
        assert_arg_semantics(
            commands,
            &["trade", "list"],
            "tickers",
            "ticker-list",
            None,
            Some(serde_json::json!([",", " "])),
        );
        assert_arg_semantics(
            commands,
            &["trade", "list"],
            "min-dollars",
            "money",
            None,
            None,
        );
        assert_arg_semantics(commands, &["trade", "list"], "limit", "integer", None, None);
        assert_arg_semantics(
            commands,
            &["trade", "list"],
            "min-price",
            "number",
            None,
            None,
        );
        assert_arg_semantics(
            commands,
            &["trade", "list"],
            "dark-pools",
            "boolean-filter",
            None,
            None,
        );
        assert_arg_semantics(
            commands,
            &["volume", "institutional"],
            "date",
            "date",
            Some("YYYY-MM-DD"),
            None,
        );
        assert_arg_semantics(
            commands,
            &["volume", "institutional"],
            "tickers",
            "ticker-list",
            None,
            Some(serde_json::json!([",", " "])),
        );
        assert_arg_semantics(
            commands,
            &["watchlist", "create"],
            "tickers",
            "ticker-list",
            None,
            Some(serde_json::json!([",", " "])),
        );
        assert_arg_semantics(
            commands,
            &["watchlist", "create"],
            "min-dollars",
            "money",
            None,
            None,
        );
        assert_arg_semantics(
            commands,
            &["watchlist", "create"],
            "normal-prints",
            "boolean-filter",
            None,
            None,
        );
        assert_arg_semantics(commands, &["completions"], "shell", "enum", None, None);
    }

    #[test]
    fn schema_exposes_custom_validation_possible_values() {
        let schema = schema_value();
        let commands = schema["commands"].as_array().unwrap();

        for path in [
            serde_json::json!(["trade", "levels"]),
            serde_json::json!(["trade", "level-touches"]),
        ] {
            let command = commands
                .iter()
                .find(|command| command["path"] == path)
                .unwrap();
            let arg = command["args"]
                .as_array()
                .unwrap()
                .iter()
                .find(|arg| arg["name"] == "trade-level-count")
                .unwrap();

            assert_eq!(arg["semantic_type"], "integer");
            assert_eq!(arg["parser"], "string");
            assert_eq!(
                arg["possible_values"],
                serde_json::json!(["5", "10", "20", "50"])
            );
        }
    }

    #[test]
    fn schema_help_command_exposes_topic_values() {
        let schema = schema_value();
        let help = schema["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|command| command["preferred_path"] == "help")
            .unwrap();
        let topic_arg = help["args"]
            .as_array()
            .unwrap()
            .iter()
            .find(|arg| arg["kind"] == "positional")
            .unwrap();

        assert_eq!(topic_arg["name"], "topic");
        assert_eq!(topic_arg["parser"], "enum");
        assert_eq!(
            topic_arg["possible_values"],
            serde_json::json!([
                "agent",
                "auth",
                "environment",
                "exit-codes",
                "schema",
                "examples"
            ])
        );
    }

    #[test]
    fn parser_names_cover_flag_action_types() {
        let false_flag = Arg::new("disabled")
            .long("disabled")
            .action(ArgAction::SetFalse);
        let count_flag = Arg::new("verbose").short('v').action(ArgAction::Count);

        assert_eq!(parser_name("dark-pool", &false_flag), "bool");
        assert_eq!(parser_name("verbose", &count_flag), "count");
    }

    fn assert_arg_semantics(
        commands: &[Value],
        path: &[&str],
        name: &str,
        semantic_type: &str,
        format: Option<&str>,
        separators: Option<Value>,
    ) {
        let command_path = path.iter().map(|part| part.to_string()).collect::<Vec<_>>();
        let command = commands
            .iter()
            .find(|command| command["path"] == serde_json::json!(command_path))
            .unwrap_or_else(|| panic!("missing command path {path:?}"));
        let arg = command["args"]
            .as_array()
            .unwrap()
            .iter()
            .find(|arg| arg["name"] == name)
            .unwrap_or_else(|| panic!("missing arg {name} for {path:?}"));

        assert_eq!(arg["semantic_type"], semantic_type);
        if let Some(format) = format {
            assert_eq!(arg["format"], format);
        }
        if let Some(separators) = separators {
            assert_eq!(arg["separators"], separators);
        }
    }

    fn live_leaf_paths() -> Vec<Vec<String>> {
        let command = Cli::command();
        let mut paths = Vec::new();

        collect_leaf_paths(&command, &mut Vec::new(), &mut paths);

        paths
    }

    fn collect_leaf_paths(command: &Command, path: &mut Vec<String>, paths: &mut Vec<Vec<String>>) {
        for subcommand in command
            .get_subcommands()
            .filter(|command| !command.is_hide_set())
        {
            path.push(subcommand.get_name().to_string());
            if subcommand.has_subcommands() {
                collect_leaf_paths(subcommand, path, paths);
            } else {
                paths.push(path.clone());
            }
            path.pop();
        }
    }

    fn string_array(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect()
    }
}
