use clap::CommandFactory;
use clap_complete::generate;
use std::io;
use tracing::instrument;

use crate::cli::CompletionsArgs;

/// Generates shell completion scripts for the given shell and writes to stdout.
#[instrument(skip_all)]
pub fn handle(args: &CompletionsArgs) {
    let mut cmd = crate::cli::Cli::command();
    let mut stdout = io::stdout();
    generate(args.shell, &mut cmd, "volumeleaders-agent", &mut stdout);
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use clap_complete::{Shell, generate};

    #[test]
    fn completions_generate_nonempty_output() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            let mut out = Vec::new();
            let mut cmd = crate::cli::Cli::command();
            generate(shell, &mut cmd, "volumeleaders-agent", &mut out);
            assert!(
                !out.is_empty(),
                "completions for {shell:?} should not be empty"
            );
        }
    }
}
