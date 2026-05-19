use std::io::{self, Write};

use anyhow::Result;

use crate::{Shell, ValueStream, process::ResolvedCommand};

impl Shell {
    pub(in crate::builtins) fn execute_rm(
        &mut self,
        args: &[String],
        input: ValueStream,
    ) -> Result<(ValueStream, i32)> {
        let mut filtered = Vec::new();
        let mut force_yes = false;
        for arg in args {
            if arg == "--yes" {
                force_yes = true;
            } else {
                filtered.push(arg.clone());
            }
        }

        let dangerous = rm_requests_recursive_delete(&filtered);
        if dangerous && self.options.interaction && !force_yes {
            eprint!("ush: confirm `rm {}` [y/N] ", filtered.join(" "));
            io::stderr().flush()?;
            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            if !matches!(answer.trim(), "y" | "Y" | "yes" | "YES") {
                return Ok((ValueStream::Empty, 130));
            }
        }

        let resolved = ResolvedCommand::new("rm", filtered);
        self.spawn_external(&resolved, input, false)
    }
}

fn rm_requests_recursive_delete(args: &[String]) -> bool {
    let mut parsing_options = true;

    for arg in args {
        if !parsing_options {
            continue;
        }
        if arg == "--" {
            parsing_options = false;
            continue;
        }
        if arg == "--recursive" || arg.starts_with("--recursive=") {
            return true;
        }
        if arg.starts_with("--") {
            continue;
        }
        let Some(flags) = arg.strip_prefix('-') else {
            continue;
        };
        if flags.is_empty() {
            continue;
        }
        if flags.chars().any(|flag| matches!(flag, 'r' | 'R')) {
            return true;
        }
    }

    false
}
