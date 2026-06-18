//! Machine-readable schema generated from the live clap command tree.

use clap::{Arg, ArgAction, Command, CommandFactory};
use serde::Serialize;

use crate::cli::Cli;
use crate::cli::error::CliExit;
use crate::cli::output::{finish_output, print_json};

const SCHEMA_VERSION: u8 = 1;
const BINARY_NAME: &str = "volumeleaders-agent";

/// Emit the CLI schema as compact JSON.
pub fn handle() -> Result<(), CliExit> {
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
    sources: [&'static str; 3],
    network_required_for_doctor: bool,
}

#[derive(Clone, Debug, Serialize)]
struct CommandExample {
    description: String,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
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
            kind: "credential_login",
            sources: ["xdg_cache", "environment", "xdg_config"],
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
        examples: examples_from_command(command, path),
        args,
    }
}

fn examples_from_command(command: &Command, path: &[String]) -> Vec<CommandExample> {
    let Some(long_about) = command.get_long_about() else {
        return Vec::new();
    };
    let preferred_path = path.join(" ");
    long_about
        .to_string()
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with(BINARY_NAME))
        .enumerate()
        .map(|(index, line)| CommandExample {
            description: format!("Example {} for {preferred_path}", index + 1),
            command: line.to_string(),
            notes: None,
        })
        .collect()
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
#[path = "schema_tests.rs"]
mod tests;
