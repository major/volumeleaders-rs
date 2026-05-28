//! Human-readable command discovery generated from the live clap tree.

use std::io::{self, Write};

use clap::{Command, CommandFactory};

use crate::cli::Cli;
use crate::cli::CommandsArgs;
use crate::cli::output::finish_output;

/// Emit available leaf commands as plain text.
pub fn handle(args: &CommandsArgs) -> i32 {
    let output = if args.grouped {
        grouped_commands_text()
    } else {
        flat_commands_text()
    };

    finish_output(write_text(&output))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LeafCommand {
    path: Vec<String>,
    about: Option<String>,
}

impl LeafCommand {
    fn preferred_path(&self) -> String {
        self.path.join(" ")
    }
}

fn flat_commands_text() -> String {
    let mut leaves = leaf_commands();
    leaves.sort_by_key(LeafCommand::preferred_path);

    let mut output = leaves
        .iter()
        .map(LeafCommand::preferred_path)
        .collect::<Vec<_>>()
        .join("\n");
    output.push('\n');
    output
}

fn grouped_commands_text() -> String {
    let mut leaves = leaf_commands();
    leaves.sort_by(|left, right| left.path.cmp(&right.path));

    let mut output = String::new();
    let mut current_group: Option<String> = None;

    for leaf in leaves {
        let group = group_name(&leaf);
        if current_group.as_deref() != Some(group) {
            if current_group.is_some() {
                output.push('\n');
            }
            output.push_str(group);
            output.push('\n');
            current_group = Some(group.to_string());
        }

        output.push_str("  ");
        output.push_str(&leaf_name(&leaf));
        if let Some(about) = leaf.about {
            output.push_str("  ");
            output.push_str(&about);
        }
        output.push('\n');
    }

    output
}

fn leaf_commands() -> Vec<LeafCommand> {
    let command = Cli::command();
    let mut leaves = Vec::new();

    collect_leaf_commands(&command, &mut Vec::new(), &mut leaves);
    leaves
}

fn collect_leaf_commands(command: &Command, path: &mut Vec<String>, leaves: &mut Vec<LeafCommand>) {
    for subcommand in command
        .get_subcommands()
        .filter(|command| !command.is_hide_set())
    {
        path.push(subcommand.get_name().to_string());
        if subcommand.has_subcommands() {
            collect_leaf_commands(subcommand, path, leaves);
        } else {
            leaves.push(LeafCommand {
                path: path.clone(),
                about: subcommand.get_about().map(ToString::to_string),
            });
        }
        path.pop();
    }
}

fn group_name(leaf: &LeafCommand) -> &str {
    leaf.path.first().map(String::as_str).unwrap_or("local")
}

fn leaf_name(leaf: &LeafCommand) -> String {
    if leaf.path.len() == 1 {
        return leaf.path[0].clone();
    }

    leaf.path[1..].join(" ")
}

fn write_text(output: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(output.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::cli::output::finish_output;

    use super::{flat_commands_text, grouped_commands_text};

    #[test]
    fn flat_commands_lists_sorted_leaf_paths() {
        let output = flat_commands_text();
        let lines: Vec<_> = output.lines().collect();
        let mut sorted = lines.clone();
        sorted.sort_unstable();

        assert_eq!(lines, sorted);
        assert!(lines.contains(&"commands"));
        assert!(lines.contains(&"doctor"));
        assert!(lines.contains(&"schema"));
        assert!(lines.contains(&"trade list"));
        assert!(lines.contains(&"volume institutional"));
        assert!(lines.contains(&"watchlist tickers"));
    }

    #[test]
    fn grouped_commands_include_groups_and_descriptions() {
        let output = grouped_commands_text();

        assert!(output.contains("trade\n"));
        assert!(
            output
                .lines()
                .any(|line| line.starts_with("  list  ") && line.contains("trades"))
        );
        assert!(output.contains("volume\n"));
        assert!(
            output
                .lines()
                .any(|line| line.starts_with("  institutional  ") && line.contains("volume"))
        );
        assert!(output.contains("commands\n"));
        assert!(
            output
                .lines()
                .any(|line| line.starts_with("  commands  ") && line.contains("leaf command paths"))
        );
    }

    #[test]
    fn grouped_leaf_count_matches_flat_output() {
        let flat_count = flat_commands_text().lines().count();
        let grouped_leaf_count = grouped_commands_text()
            .lines()
            .filter(|line| line.starts_with("  "))
            .count();

        assert_eq!(grouped_leaf_count, flat_count);
    }

    #[test]
    fn write_errors_map_to_json_exit_code() {
        assert_eq!(finish_output(Err(io::Error::other("stdout closed"))), 6);
    }
}
