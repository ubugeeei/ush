use std::{
    collections::BTreeSet,
    env, fs,
    io::{self, Write},
    path::Path,
    process::Command,
};

use rustyline::validate::ValidationResult;

use anyhow::{Context, Result, anyhow, bail};

use super::test_eval;
use crate::{
    Shell, ValueStream, expand::strip_outer_quotes, process::ResolvedCommand, repl, signal, style,
};

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
            if self.options.stylish {
                return Ok((ValueStream::Text(style::render_aliases(&self.aliases)), 0));
            }

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

    pub(super) fn handle_port(
        &self,
        args: &[String],
        input: ValueStream,
    ) -> Result<(ValueStream, i32)> {
        let ports = port_targets(args, input)?;
        let mut pids = BTreeSet::new();
        for port in ports {
            pids.extend(listening_pids_for_port(port)?);
        }

        if pids.is_empty() {
            return Ok((ValueStream::Empty, 1));
        }

        Ok((
            ValueStream::Lines(pids.into_iter().map(|pid| pid.to_string()).collect()),
            0,
        ))
    }

    pub(super) fn handle_stop(
        &mut self,
        args: &[String],
        input: ValueStream,
    ) -> Result<(ValueStream, i32)> {
        let (signal_value, pids) = stop_targets(args, input)?;
        for pid in pids {
            signal::send_process_signal(pid, signal_value)
                .with_context(|| format!("failed to signal pid {pid}"))?;
        }
        Ok((ValueStream::Empty, 0))
    }

    pub(super) fn handle_source(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
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

    pub(super) fn handle_history(&self, args: &[String]) -> Result<(ValueStream, i32)> {
        let limit = parse_history_limit(args)?;
        let entries = self.read_history_entries();

        if self.options.stylish {
            return Ok((ValueStream::Text(style::render_history(&entries, limit)), 0));
        }

        let text = render_history_plain(&entries, limit);
        Ok((ValueStream::Text(text), 0))
    }

    pub(super) fn read_history(&self) -> String {
        fs::read_to_string(&self.paths.history_file).unwrap_or_default()
    }

    pub(super) fn read_history_entries(&self) -> Vec<String> {
        self.read_history()
            .lines()
            .map(ToString::to_string)
            .collect()
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

fn parse_history_limit(args: &[String]) -> Result<Option<usize>> {
    let mut pending_limit = false;
    let mut limit = None;

    for arg in args {
        if pending_limit {
            limit = Some(parse_history_limit_value(arg)?);
            pending_limit = false;
            continue;
        }

        match arg.as_str() {
            "--limit" => pending_limit = true,
            _ if arg.starts_with("--limit=") => {
                limit = Some(parse_history_limit_value(arg.split_once('=').unwrap().1)?);
            }
            _ if arg.chars().all(|ch| ch.is_ascii_digit()) => {
                limit = Some(parse_history_limit_value(arg)?);
            }
            _ => bail!("history accepts only a numeric limit or --limit N"),
        }
    }

    if pending_limit {
        bail!("history --limit requires a value");
    }

    Ok(limit)
}

fn parse_history_limit_value(value: &str) -> Result<usize> {
    let limit = value.parse::<usize>()?;
    Ok(limit.max(1))
}

fn render_history_plain(entries: &[String], limit: Option<usize>) -> String {
    let start = entries
        .len()
        .saturating_sub(limit.unwrap_or(entries.len()).min(entries.len()));
    let text = entries[start..]
        .iter()
        .enumerate()
        .map(|(offset, entry)| format!("{}\t{}", start + offset + 1, entry))
        .collect::<Vec<_>>()
        .join("\n");
    with_trailing_newline(text)
}

fn port_targets(args: &[String], input: ValueStream) -> Result<Vec<u16>> {
    let raw = if args.is_empty() {
        input.into_lines()?
    } else {
        args.to_vec()
    };

    if raw.is_empty() {
        bail!("port requires at least one port number");
    }

    raw.into_iter()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .trim()
                .parse::<u16>()
                .with_context(|| format!("invalid port: {value}"))
        })
        .collect()
}

fn stop_targets(args: &[String], input: ValueStream) -> Result<(i32, Vec<i32>)> {
    let mut signal_value = default_stop_signal();
    let mut raw_targets = Vec::new();
    let mut index = 0usize;

    while let Some(arg) = args.get(index) {
        match arg.as_str() {
            "--signal" | "-s" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("stop --signal requires a value");
                };
                signal_value = parse_signal_name(value)?;
                index += 2;
            }
            _ if is_signal_flag(arg) => {
                signal_value = parse_signal_name(&arg[1..])?;
                index += 1;
            }
            _ => {
                raw_targets.extend(args[index..].iter().cloned());
                break;
            }
        }
    }

    if raw_targets.is_empty() {
        raw_targets = input.into_lines()?;
    }

    let pids = raw_targets
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_pid_target(&value))
        .collect::<Result<BTreeSet<_>>>()?
        .into_iter()
        .collect::<Vec<_>>();

    if pids.is_empty() {
        bail!("stop requires at least one pid");
    }

    Ok((signal_value, pids))
}

fn listening_pids_for_port(port: u16) -> Result<Vec<i32>> {
    let mut pids = BTreeSet::new();
    pids.extend(run_lsof(&[
        "-nP".to_string(),
        "-t".to_string(),
        format!("-iTCP:{port}"),
        "-sTCP:LISTEN".to_string(),
    ])?);
    pids.extend(run_lsof(&[
        "-nP".to_string(),
        "-t".to_string(),
        format!("-iUDP:{port}"),
    ])?);

    if pids.is_empty() {
        pids.extend(run_lsof(&[
            "-nP".to_string(),
            "-t".to_string(),
            format!("-i:{port}"),
        ])?);
    }

    Ok(pids.into_iter().collect())
}

fn run_lsof(args: &[String]) -> Result<Vec<i32>> {
    let output = Command::new("lsof").args(args).output().with_context(|| {
        format!("failed to run lsof; install it to use `port` ({})", args.join(" "))
    })?;

    if !output.status.success() && output.status.code() != Some(1) {
        bail!(
            "lsof failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    String::from_utf8(output.stdout)
        .context("lsof output was not utf-8")?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.trim()
                .parse::<i32>()
                .with_context(|| format!("invalid pid from lsof: {line}"))
        })
        .collect()
}

fn default_stop_signal() -> i32 {
    #[cfg(unix)]
    {
        libc::SIGTERM
    }
    #[cfg(not(unix))]
    {
        15
    }
}

fn is_signal_flag(value: &str) -> bool {
    value.starts_with('-') && value.len() > 1 && value != "--"
}

fn parse_signal_name(value: &str) -> Result<i32> {
    let trimmed = value.trim();
    let upper = trimmed
        .strip_prefix("SIG")
        .unwrap_or(trimmed)
        .to_ascii_uppercase();

    if let Ok(number) = upper.parse::<i32>() {
        return Ok(number);
    }

    #[cfg(unix)]
    {
        match upper.as_str() {
            "TERM" => Ok(libc::SIGTERM),
            "KILL" => Ok(libc::SIGKILL),
            "INT" => Ok(libc::SIGINT),
            "HUP" => Ok(libc::SIGHUP),
            other => bail!("unsupported signal: {other}"),
        }
    }
    #[cfg(not(unix))]
    {
        match upper.as_str() {
            "TERM" => Ok(15),
            "KILL" => Ok(9),
            "INT" => Ok(2),
            "HUP" => Ok(1),
            other => bail!("unsupported signal: {other}"),
        }
    }
}

fn parse_pid_target(value: &str) -> Result<i32> {
    let trimmed = value.trim();
    let pid = trimmed
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .parse::<i32>()
        .with_context(|| format!("invalid pid: {value}"))?;
    if pid <= 0 {
        bail!("invalid pid: {value}");
    }
    Ok(pid)
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
