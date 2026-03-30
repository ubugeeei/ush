use std::{
    env, fs,
    io::{self, Write},
};

use anyhow::{Context, Result, anyhow, bail};

use super::{Shell, ValueStream, expand::strip_outer_quotes, process::ResolvedCommand};

impl Shell {
    pub(crate) fn execute_builtin(
        &mut self,
        spec: &super::parser::CommandSpec,
        input: ValueStream,
    ) -> Result<(ValueStream, i32)> {
        let args = self.expand_args(&spec.args)?;

        match spec.command.as_str() {
            "cd" => self.change_directory(&args),
            "pwd" => Ok((ValueStream::Text(self.render_pwd()), 0)),
            "alias" => self.handle_alias(&args),
            "unalias" => self.handle_unalias(&args),
            "history" => Ok((ValueStream::Text(self.read_history()), 0)),
            "export" => self.handle_export(&args),
            "help" => Ok((ValueStream::Text(help_text()), 0)),
            "source" => self.handle_source(&args),
            "exit" => self.handle_exit(&args),
            "rm" => self.execute_rm(&args, input),
            other => bail!("unsupported builtin: {other}"),
        }
    }

    fn change_directory(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
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

    fn render_pwd(&self) -> String {
        if self.options.stylish {
            format!("\u{1b}[1;34m{}\u{1b}[0m\n", self.cwd.display())
        } else {
            format!("{}\n", self.cwd.display())
        }
    }

    fn handle_alias(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        if args.is_empty() {
            let text = self
                .aliases
                .iter()
                .map(|(name, value)| format!("alias {name}='{}'", value.replace('\'', r#"'\''"#)))
                .collect::<Vec<_>>()
                .join("\n");
            let text = if text.is_empty() {
                text
            } else {
                format!("{text}\n")
            };
            return Ok((ValueStream::Text(text), 0));
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

    fn handle_unalias(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        for arg in args {
            self.aliases.remove(arg);
        }
        Ok((ValueStream::Empty, 0))
    }

    fn handle_export(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        for arg in args {
            let (name, value) = arg
                .split_once('=')
                .ok_or_else(|| anyhow!("export syntax must be NAME=value"))?;
            self.env.insert(name.to_string(), self.expand_value(value)?);
        }
        Ok((ValueStream::Empty, 0))
    }

    fn handle_source(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
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

    fn handle_exit(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        let status = args
            .first()
            .and_then(|value| value.parse::<i32>().ok())
            .unwrap_or(self.last_status);
        std::process::exit(status);
    }

    fn read_history(&self) -> String {
        fs::read_to_string(&self.paths.history_file).unwrap_or_default()
    }

    fn execute_rm(&mut self, args: &[String], input: ValueStream) -> Result<(ValueStream, i32)> {
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

fn help_text() -> String {
    [
        "ush builtins:",
        "  cd <dir>",
        "  pwd",
        "  alias name=value",
        "  unalias name",
        "  export NAME=value",
        "  history",
        "  source <file>",
        "  rm -rf <path>    # guarded unless --yes or USH_INTERACTION=false",
        "",
        "interactive shortcuts:",
        "  Up/Down          history",
        "  Shift-Left/Right select chars",
        "  Shift-Up/Down    same as Up/Down when terminal passes them through",
        "  Option-Up/Down   prefix history search",
        "  Option-Left/Right word move",
        "  Option-Shift-Left/Right select word",
        "  Ctrl-Alt-Shift-Left/Right select big word",
        "  Home/End         line start/end",
        "  Shift-Home/End   select to line edge (Cmd-Left/Right if terminal maps them)",
        "",
        "structured helpers:",
        "  length",
        "  lines",
        "  json",
        "  map(it -> upper(it))",
        "  filter(it -> contains(it, \"foo\"))",
        "  any(it -> starts_with(it, \"warn\"))",
    ]
    .join("\n")
        + "\n"
}
