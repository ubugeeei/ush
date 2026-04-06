mod rm;

use std::{collections::BTreeSet, process::Command};

use anyhow::{Context, Result, bail};

use crate::{Shell, ValueStream, signal};

impl Shell {
    pub(in crate::builtins) fn handle_port(
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

    pub(in crate::builtins) fn handle_stop(
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
        format!(
            "failed to run lsof; install it to use `port` ({})",
            args.join(" ")
        )
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
