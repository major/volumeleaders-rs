//! Output field discovery command.

use tracing::instrument;

use crate::cli::FieldsArgs;
use crate::cli::error::usage_error;
use crate::cli::field_metadata::discover;
use crate::cli::output::{finish_output, print_json};

/// Handles `fields <command path>`.
#[instrument(skip_all)]
pub fn handle(args: &FieldsArgs) -> i32 {
    match discover(&args.command_path) {
        Some(discovery) => finish_output(print_json(&discovery)),
        None => usage_error(format!(
            "unknown command path for field discovery: {}",
            args.command_path.join(" ")
        )),
    }
}
