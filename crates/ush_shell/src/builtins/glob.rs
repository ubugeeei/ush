use std::io::{self, IsTerminal, Read};

use anyhow::{Context, Result, bail};

use crate::{Shell, ValueStream};

impl Shell {
    pub(super) fn handle_glob(
        &self,
        args: &[String],
        input: ValueStream,
    ) -> Result<(ValueStream, i32)> {
        let patterns = if args.is_empty() {
            let piped = input
                .into_lines()?
                .into_iter()
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();
            if piped.is_empty() {
                read_patterns_from_stdin()?
            } else {
                piped
            }
        } else {
            args.to_vec()
        };

        if patterns.is_empty() {
            bail!("glob requires at least one pattern");
        }

        let mut matches = Vec::new();
        for pattern in patterns {
            let mut resolved = glob::glob(&pattern)
                .with_context(|| format!("invalid glob pattern: {pattern}"))?
                .filter_map(Result::ok)
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>();
            resolved.sort();
            matches.extend(resolved);
        }

        if matches.is_empty() {
            return Ok((ValueStream::Empty, 1));
        }

        Ok((ValueStream::Text(format!("{}\n", matches.join("\n"))), 0))
    }
}

fn read_patterns_from_stdin() -> Result<Vec<String>> {
    if io::stdin().is_terminal() {
        return Ok(Vec::new());
    }

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("failed to read glob patterns from stdin")?;
    Ok(buffer
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}
