use std::{
    env, fs,
    io::{self, Write},
};

use anyhow::{Context, Result, anyhow, bail};

use super::test_eval;
use crate::{Shell, ValueStream, expand::strip_outer_quotes, process::ResolvedCommand, style};

impl Shell {
    pub(super) fn change_directory(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        let target = args.first().cloned().unwrap_or_else(|| {
            self.env
                .get("HOME")
                .cloned()
                .unwrap_or_else(|| ".".to_string())
        });
        let path = self.normalize_path(&target);
        env::set_current_dir(&path)
            .with_context(|| format!("failed to change directory to {}", path.display()))?;
        self.cwd = env::current_dir()?;
        self.env
            .insert("PWD".to_string(), self.cwd.display().to_string());
        Ok((ValueStream::Empty, 0))
    }

    pub(super) fn render_pwd(&self) -> String {
        if self.options.stylish {
            format!(
                "{} {}\n",
                style::paint("\u{1b}[1;34m", "cwd"),
                style::paint("\u{1b}[1;36m", self.cwd.display())
            )
        } else {
            format!("{}\n", self.cwd.display())
        }
    }

    pub(super) fn handle_alias(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        if args.is_empty() {
            let text = self
                .aliases
                .iter()
                .map(|(name, value)| format!("alias {name}='{}'", value.replace('\'', r#"'\''"#)))
                .collect::<Vec<_>>()
                .join("\n");
            return Ok((ValueStream::Text(with_trailing_newline(text)), 0));
        }

        for arg in args {
            let (name, value) = arg
                .split_once('=')
                .ok_or_else(|| anyhow!("alias syntax must be name=value"))?;
            self.aliases
                .insert(name.to_string(), strip_outer_quotes(value).to_string());
        }
        Ok((ValueStream::Empty, 0))
    }

    pub(super) fn handle_unalias(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        for arg in args {
            self.aliases.remove(arg);
        }
        Ok((ValueStream::Empty, 0))
    }

    pub(super) fn handle_export(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        for arg in args {
            let (name, value) = arg
                .split_once('=')
                .ok_or_else(|| anyhow!("export syntax must be NAME=value"))?;
            self.env.insert(name.to_string(), self.expand_value(value)?);
        }
        Ok((ValueStream::Empty, 0))
    }

    pub(super) fn handle_unset(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        for arg in args {
            if arg.starts_with('-') && arg != "-v" {
                bail!("unsupported unset option: {arg}");
            }
            if arg != "-v" {
                self.env.remove(arg);
            }
        }
        Ok((ValueStream::Empty, 0))
    }

    pub(super) fn handle_source(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        let Some(path) = args.first() else {
            bail!("source requires a file path");
        };
        let source = fs::read_to_string(self.normalize_path(path))
            .with_context(|| format!("failed to read {path}"))?;
        for line in source.lines() {
            self.execute(line)?;
        }
        Ok((ValueStream::Empty, 0))
    }

    pub(super) fn handle_exit(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        let status = args
            .first()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(self.last_status);
        std::process::exit(status);
    }

    pub(super) fn handle_test(&self, command: &str, args: &[String]) -> Result<(ValueStream, i32)> {
        let parts = if command == "[" {
            let Some(last) = args.last() else {
                bail!("[ requires an expression");
            };
            if last != "]" {
                bail!("[ requires a closing `]`");
            }
            &args[..args.len() - 1]
        } else {
            args
        };
        let matched = test_eval::evaluate(self, parts)?;
        Ok((ValueStream::Empty, if matched { 0 } else { 1 }))
    }

    pub(super) fn read_history(&self) -> String {
        fs::read_to_string(&self.paths.history_file).unwrap_or_default()
    }

    pub(super) fn execute_rm(
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

        let dangerous = filtered
            .iter()
            .any(|arg| arg == "-rf" || arg == "-fr" || arg == "--recursive");
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

pub(super) fn render_echo(args: &[String]) -> String {
    let mut index = 0usize;
    while args.get(index).is_some_and(|arg| arg == "-n") {
        index += 1;
    }
    let mut text = args[index..].join(" ");
    if index == 0 {
        text.push('\n');
    }
    text
}

fn with_trailing_newline(text: String) -> String {
    if text.is_empty() {
        text
    } else {
        format!("{text}\n")
    }
}
