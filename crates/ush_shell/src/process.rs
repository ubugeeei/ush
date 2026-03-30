use std::{
    collections::HashMap,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, anyhow};
use which::which;

use super::{Shell, ValueStream, style};

#[derive(Debug)]
pub(crate) struct ResolvedCommand {
    command: String,
    args: Vec<String>,
    env_overrides: HashMap<String, String>,
}

impl ResolvedCommand {
    pub(crate) fn new(command: &str, args: Vec<String>) -> Self {
        Self {
            command: command.to_string(),
            args,
            env_overrides: HashMap::new(),
        }
    }
}

impl Shell {
    pub(crate) fn execute_external(
        &mut self,
        spec: &super::parser::CommandSpec,
        input: ValueStream,
        capture: bool,
    ) -> Result<(ValueStream, i32)> {
        let resolved = ResolvedCommand {
            command: spec.command.clone(),
            args: self.expand_args(&spec.args)?,
            env_overrides: spec
                .assignments
                .iter()
                .map(|(name, value)| Ok((name.clone(), self.expand_value(value)?)))
                .collect::<Result<HashMap<_, _>>>()?,
        };

        if self.options.stylish {
            if let Some(rendered) = self.try_stylish(&resolved, &input)? {
                return Ok((rendered, 0));
            }
        }

        self.spawn_external(&resolved, input, capture)
    }

    pub(crate) fn spawn_external(
        &mut self,
        resolved: &ResolvedCommand,
        input: ValueStream,
        capture: bool,
    ) -> Result<(ValueStream, i32)> {
        if which(&resolved.command).is_err() {
            return Err(anyhow!("command not found: {}", resolved.command));
        }

        let mut command = Command::new(&resolved.command);
        command
            .args(&resolved.args)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .envs(&resolved.env_overrides)
            .stderr(Stdio::inherit())
            .stdin(if input.is_empty() {
                Stdio::inherit()
            } else {
                Stdio::piped()
            })
            .stdout(if capture {
                Stdio::piped()
            } else {
                Stdio::inherit()
            });

        let mut child = command
            .spawn()
            .with_context(|| format!("failed to run {}", resolved.command))?;
        if !input.is_empty() {
            child
                .stdin
                .as_mut()
                .context("child stdin unavailable")?
                .write_all(&input.to_bytes()?)?;
        }

        if capture {
            let output = child.wait_with_output()?;
            let status = output.status.code().unwrap_or(1);
            let stdout = String::from_utf8(output.stdout)
                .unwrap_or_else(|error| String::from_utf8_lossy(&error.into_bytes()).to_string());
            Ok((ValueStream::Text(stdout), status))
        } else {
            Ok((ValueStream::Empty, child.wait()?.code().unwrap_or(1)))
        }
    }

    pub(crate) fn execute_posix_shell_stage(
        &mut self,
        spec: &super::parser::CommandSpec,
        input: ValueStream,
        capture: bool,
    ) -> Result<(ValueStream, i32)> {
        let args = self.expand_args(&spec.args)?;
        let env_overrides = spec
            .assignments
            .iter()
            .map(|(name, value)| Ok((name.clone(), self.expand_value(value)?)))
            .collect::<Result<HashMap<_, _>>>()?;

        let mut command = Command::new("/bin/sh");
        command
            .args(&args)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .envs(env_overrides)
            .stderr(Stdio::inherit())
            .stdin(if input.is_empty() {
                Stdio::inherit()
            } else {
                Stdio::piped()
            })
            .stdout(if capture {
                Stdio::piped()
            } else {
                Stdio::inherit()
            });

        let mut child = command.spawn().context("failed to spawn /bin/sh")?;
        if !input.is_empty() {
            child
                .stdin
                .as_mut()
                .context("posix shell stdin unavailable")?
                .write_all(&input.to_bytes()?)?;
        }

        if capture {
            let output = child.wait_with_output()?;
            Ok((
                ValueStream::Text(String::from_utf8_lossy(&output.stdout).to_string()),
                output.status.code().unwrap_or(1),
            ))
        } else {
            Ok((ValueStream::Empty, child.wait()?.code().unwrap_or(1)))
        }
    }

    pub(crate) fn run_fallback(
        &mut self,
        source: &str,
        input: ValueStream,
        capture: bool,
    ) -> Result<(ValueStream, i32)> {
        let mut command = Command::new("/bin/sh");
        command
            .arg("-c")
            .arg(source)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .stderr(Stdio::inherit())
            .stdin(if input.is_empty() {
                Stdio::inherit()
            } else {
                Stdio::piped()
            })
            .stdout(if capture {
                Stdio::piped()
            } else {
                Stdio::inherit()
            });

        let mut child = command.spawn().context("failed to spawn /bin/sh")?;
        if !input.is_empty() {
            child
                .stdin
                .as_mut()
                .context("fallback stdin unavailable")?
                .write_all(&input.to_bytes()?)?;
        }

        if capture {
            let output = child.wait_with_output()?;
            Ok((
                ValueStream::Text(String::from_utf8_lossy(&output.stdout).to_string()),
                output.status.code().unwrap_or(1),
            ))
        } else {
            Ok((ValueStream::Empty, child.wait()?.code().unwrap_or(1)))
        }
    }

    pub fn run_compiled_script(
        &mut self,
        origin: &Path,
        script: &str,
        args: &[String],
    ) -> Result<i32> {
        let mut child = Command::new("/bin/sh")
            .arg("-s")
            .arg("--")
            .args(args)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| {
                format!("failed to launch compiled script for {}", origin.display())
            })?;

        child
            .stdin
            .as_mut()
            .context("compiled script stdin unavailable")?
            .write_all(script.as_bytes())?;
        self.last_status = child.wait()?.code().unwrap_or(1);
        Ok(self.last_status)
    }

    fn try_stylish(
        &self,
        resolved: &ResolvedCommand,
        input: &ValueStream,
    ) -> Result<Option<ValueStream>> {
        match resolved.command.as_str() {
            "ls" => style::render_ls(&self.cwd, &resolved.args),
            "cat" => style::render_cat(&self.cwd, &resolved.args, input),
            "ps" => style::render_ps(&resolved.args),
            "kill" => style::render_kill(&resolved.args),
            _ => Ok(None),
        }
    }
}

pub(crate) fn is_posix_shell_command(command: &str) -> bool {
    matches!(command, "sh" | "/bin/sh")
}
