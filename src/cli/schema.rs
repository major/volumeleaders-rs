//! Machine-readable schema generated from the live clap command tree.

use clap::{Arg, ArgAction, Command, CommandFactory};
use serde::Serialize;

use crate::cli::Cli;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    alias_for: Option<Vec<String>>,
    preferred_path: String,
    aliases: Vec<String>,
    auth_required: bool,
    about: Option<String>,
    long_about: Option<String>,
    args: Vec<ArgSchema>,
}

#[derive(Clone, Debug, Serialize)]
struct ArgSchema {
    long: Option<String>,
    short: Option<char>,
    value_name: Option<String>,
    kind: ArgKind,
    required: bool,
    default: Option<Vec<String>>,
    parser: String,
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
        alias_for: None,
        preferred_path: path.join(" "),
        aliases: command
            .get_visible_aliases()
            .map(ToString::to_string)
            .collect(),
        auth_required: auth_required(path),
        about: command.get_about().map(ToString::to_string),
        long_about: command.get_long_about().map(ToString::to_string),
        args,
    }
}

fn add_alias_commands(commands: &mut Vec<CommandSchema>) {
    for (alias, canonical) in [
        ("trades", ["trade", "list"]),
        ("dashboard", ["trade", "dashboard"]),
        ("levels", ["trade", "levels"]),
    ] {
        let canonical_path = canonical.map(str::to_string).to_vec();
        let Some(canonical_command) = commands
            .iter()
            .find(|command| command.path == canonical_path)
        else {
            continue;
        };

        commands.push(CommandSchema {
            path: vec![alias.to_string()],
            alias_for: Some(canonical_path),
            preferred_path: canonical_command.preferred_path.clone(),
            aliases: Vec::new(),
            auth_required: canonical_command.auth_required,
            about: Some(format!("Alias for `{}`.", canonical_command.preferred_path)),
            long_about: canonical_command.long_about.clone(),
            args: canonical_command.args.clone(),
        });
    }
}

fn auth_required(path: &[String]) -> bool {
    !matches!(path, [command] if command == "commands" || command == "doctor" || command == "fields" || command == "help" || command == "schema" || command == "completions")
}

fn arg_schema(arg: &Arg) -> ArgSchema {
    ArgSchema {
        long: arg.get_long().map(ToString::to_string),
        short: arg.get_short(),
        value_name: value_name(arg),
        kind: arg_kind(arg),
        required: arg.is_required_set(),
        default: default_values(arg),
        parser: parser_name(arg),
        possible_values: possible_values(arg),
        multi_value: multi_value(arg),
    }
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

fn parser_name(arg: &Arg) -> String {
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

fn possible_values(arg: &Arg) -> Vec<String> {
    arg.get_value_parser()
        .possible_values()
        .map(|values| values.map(|value| value.get_name().to_string()).collect())
        .unwrap_or_default()
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
        assert_eq!(
            dashboard_alias["alias_for"],
            serde_json::json!(["trade", "dashboard"])
        );
        assert_eq!(dashboard_alias["preferred_path"], "trade dashboard");
        assert_eq!(
            levels_alias["alias_for"],
            serde_json::json!(["trade", "levels"])
        );
        assert_eq!(levels_alias["preferred_path"], "trade levels");
    }

    #[test]
    fn alias_generation_skips_missing_canonical_commands() {
        let mut commands = vec![CommandSchema {
            path: vec!["trade".to_string(), "list".to_string()],
            alias_for: None,
            preferred_path: "trade list".to_string(),
            aliases: Vec::new(),
            auth_required: true,
            about: Some("List trades".to_string()),
            long_about: Some("List trades.\n\nExamples:\n  volumeleaders-agent trade list NVDA\n  volumeleaders-agent trade list AAPL".to_string()),
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

        assert!(args.iter().any(|arg| arg["long"] == "fields"));
        assert!(
            args.iter()
                .any(|arg| arg["long"] == "all-fields" && arg["kind"] == "flag")
        );
        assert!(
            args.iter()
                .any(|arg| arg["long"] == "limit" && arg["default"] == serde_json::json!(["1000"]))
        );
        assert!(
            args.iter()
                .any(|arg| arg["kind"] == "positional" && arg["multi_value"] == true)
        );
        assert!(args.iter().any(|arg| {
            arg["long"] == "strict-empty" && arg["kind"] == "flag" && arg["parser"] == "enum"
        }));
        assert!(args.iter().any(|arg| {
            arg["long"] == "verbose"
                && arg["short"] == "v"
                && arg["kind"] == "flag"
                && arg["parser"] == "count"
                && arg["multi_value"] == true
        }));
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

        assert_eq!(parser_name(&false_flag), "bool");
        assert_eq!(parser_name(&count_flag), "count");
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
