//! Dry-run planning helpers for mutating CLI commands.

use serde::Serialize;
use serde_json::json;

use crate::cli::output::{finish_output, print_json};

/// Writes a compact JSON dry-run plan to stdout.
pub(crate) fn print_dry_run_plan<T: Serialize + ?Sized>(
    command: &'static str,
    operation: &'static str,
    request: &T,
) -> i32 {
    finish_output(print_json(&dry_run_plan(command, operation, request)))
}

fn dry_run_plan<T: Serialize + ?Sized>(
    command: &'static str,
    operation: &'static str,
    request: &T,
) -> serde_json::Value {
    json!({
        "dry_run": true,
        "operation": operation,
        "command": command,
        "auth_required": true,
        "request": request,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_plan_includes_mutation_context() {
        let request = json!({"key": 123});
        let plan = dry_run_plan("alert delete", "delete", &request);

        assert_eq!(plan["dry_run"], true);
        assert_eq!(plan["operation"], "delete");
        assert_eq!(plan["command"], "alert delete");
        assert_eq!(plan["auth_required"], true);
        assert_eq!(plan["request"], request);
    }
}
