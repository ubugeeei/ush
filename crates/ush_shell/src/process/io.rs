use std::{
    collections::HashMap,
    io::Write,
    path::Path,
    process::{Child, Command, Stdio},
};

use anyhow::{Context, Result};

use crate::{ValueStream, signal};

pub(super) fn populate_command(
    command: &mut Command,
    cwd: &Path,
    env: &HashMap<String, String>,
    overrides: &HashMap<String, String>,
    args: &[String],
    inherit_stdin: bool,
    capture: bool,
) {
    command
        .args(args)
        .current_dir(cwd)
        .envs(env)
        .envs(overrides)
        .stderr(Stdio::inherit())
        .stdin(if inherit_stdin {
            Stdio::inherit()
        } else {
            Stdio::piped()
        })
        .stdout(if capture {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
}

pub(super) fn write_input(child: &mut Child, input: &ValueStream) -> Result<()> {
    if input.is_empty() {
        return Ok(());
    }
    child
        .stdin
        .as_mut()
        .context("child stdin unavailable")?
        .write_all(&input.to_bytes()?)?;
    Ok(())
}

pub(super) fn finish_child(mut child: Child, capture: bool) -> Result<(ValueStream, i32)> {
    if capture {
        let output = child.wait_with_output()?;
        let stdout = String::from_utf8(output.stdout)
            .unwrap_or_else(|error| String::from_utf8_lossy(&error.into_bytes()).to_string());
        return Ok((
            ValueStream::Text(stdout),
            signal::exit_status(output.status),
        ));
    }
    Ok((ValueStream::Empty, signal::exit_status(child.wait()?)))
}
