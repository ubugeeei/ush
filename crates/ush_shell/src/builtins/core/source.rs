use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use rustyline::validate::ValidationResult;

use crate::{Shell, ValueStream, repl};

impl Shell {
    pub(in crate::builtins) fn handle_source(
        &mut self,
        args: &[String],
    ) -> Result<(ValueStream, i32)> {
        let Some(path) = args.first() else {
            bail!("source requires a file path");
        };
        let status = self
            .source_path(&self.normalize_path(path))
            .with_context(|| format!("failed to read {path}"))?;
        Ok((ValueStream::Empty, status))
    }

    pub(crate) fn source_path(&mut self, path: &Path) -> Result<i32> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut chunk = String::new();
        let mut last_status = 0;
        for line in source.lines() {
            if !chunk.is_empty() {
                chunk.push('\n');
            }
            chunk.push_str(line);

            if matches!(repl::validate_input(&chunk), ValidationResult::Incomplete) {
                continue;
            }
            if chunk.trim().is_empty() {
                chunk.clear();
                continue;
            }

            last_status = self.execute(&chunk)?;
            chunk.clear();
        }

        if !chunk.trim().is_empty() {
            last_status = self.execute(&chunk)?;
        }

        Ok(last_status)
    }
}
