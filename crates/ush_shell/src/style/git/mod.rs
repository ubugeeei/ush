mod branch;
mod log;
mod model;
mod status;

use std::{path::Path, process::Command};

use anyhow::{Context, Result};

use crate::helpers::ValueStream;

pub fn render_git(cwd: &Path, args: &[String]) -> Result<Option<ValueStream>> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Ok(None);
    };
    match subcommand.as_str() {
        "status" => status::render_git_status(cwd, rest),
        "branch" => branch::render_git_branch(cwd, rest),
        "log" => log::render_git_log(cwd, rest),
        _ => Ok(None),
    }
}

fn git_capture(cwd: &Path, args: &[String]) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .context("failed to run git")?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8(output.stdout).unwrap_or_else(
        |error| String::from_utf8_lossy(&error.into_bytes()).to_string(),
    )))
}
