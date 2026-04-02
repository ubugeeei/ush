mod state;
use self::state::{StatefulShellRun, render_alias_prelude};
use super::{Shell, ValueStream, commands, signal, style};
use anyhow::{Context, Result, anyhow, bail};
use std::{
    collections::HashMap,
    io::Write,
    path::Path,
    process::{Child, Command, Stdio},
};
const STATE_CHUNK_FILE: &str = "USH_INTERNAL_CHUNK_FILE";
const STATE_CWD_FILE: &str = "USH_INTERNAL_STATE_CWD";
const STATE_ENV_FILE: &str = "USH_INTERNAL_STATE_ENV";
const STATE_ALIAS_FILE: &str = "USH_INTERNAL_STATE_ALIAS";
const STATE_DONE_FILE: &str = "USH_INTERNAL_STATE_DONE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JobState {
    Running,
    Done(i32),
}

pub(crate) struct Job {
    id: usize,
    command: String,
    pid: u32,
    child: Child,
    state: JobState,
}

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
    pub(crate) fn handle_jobs(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        if !args.is_empty() {
            bail!("jobs does not accept arguments");
        }
        self.refresh_jobs()?;
        let text = self
            .jobs
            .iter()
            .map(|job| {
                format!(
                    "[{}] {:<7} {}\n",
                    job.id,
                    job.state.label(),
                    job.command
                )
            })
            .collect::<String>();
        Ok((ValueStream::Text(text), 0))
    }

    pub(crate) fn handle_wait(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        self.refresh_jobs()?;
        let job_ids = if args.is_empty() {
            self.jobs.iter().map(|job| job.id).collect::<Vec<_>>()
        } else {
            args.iter()
                .map(|arg| self.parse_job_spec(arg))
                .collect::<Result<Vec<_>>>()?
        };

        let mut last_status = 0;
        for id in job_ids {
            last_status = self.wait_for_job(id)?;
        }

        Ok((ValueStream::Empty, last_status))
    }

    pub(crate) fn handle_disown(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        self.refresh_jobs()?;
        let job_ids = if args.is_empty() {
            vec![self.current_job_id()?]
        } else {
            args.iter()
                .map(|arg| self.parse_job_spec(arg))
                .collect::<Result<Vec<_>>>()?
        };
        self.remove_jobs(&job_ids)?;
        Ok((ValueStream::Empty, 0))
    }

    pub(crate) fn handle_fg(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        self.refresh_jobs()?;
        let id = self.resolve_single_job_spec("fg", args)?;
        let position = self
            .job_position(id)
            .ok_or_else(|| anyhow!("unknown job: %{id}"))?;
        if self.jobs[position].state == JobState::Running {
            signal::continue_background_job(self.jobs[position].pid)
                .with_context(|| format!("failed to continue job %{id}"))?;
        }
        let status = self.wait_for_job(id)?;
        Ok((ValueStream::Empty, status))
    }

    pub(crate) fn handle_bg(&mut self, args: &[String]) -> Result<(ValueStream, i32)> {
        self.refresh_jobs()?;
        let id = self.resolve_single_job_spec("bg", args)?;
        let position = self
            .job_position(id)
            .ok_or_else(|| anyhow!("unknown job: %{id}"))?;
        let job = &mut self.jobs[position];
        if job.state != JobState::Running {
            bail!("job %{id} is no longer running");
        }
        signal::continue_background_job(job.pid)
            .with_context(|| format!("failed to continue job %{id}"))?;
        job.state = JobState::Running;
        Ok((ValueStream::Empty, 0))
    }

    pub(crate) fn spawn_background_job(&mut self, source: &str) -> Result<String> {
        let mut command = Command::new("/bin/sh");
        let mut script = render_alias_prelude(&self.aliases);
        script.push_str(source);
        command
            .arg("-c")
            .arg(script)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        signal::prepare_background_command(&mut command);

        let child = command
            .spawn()
            .with_context(|| format!("failed to spawn background job `{source}`"))?;
        let id = self.next_job_id;
        self.next_job_id += 1;
        let pid = child.id();
        self.jobs.push(Job {
            id,
            command: source.to_string(),
            pid,
            child,
            state: JobState::Running,
        });
        Ok(format!("[{id}] {pid}\n"))
    }

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
        if self.options.stylish
            && let Some((rendered, status)) = self.try_stylish(&resolved, &input)?
        {
            return Ok((rendered, status));
        }
        self.spawn_external(&resolved, input, capture)
    }

    pub(crate) fn spawn_external(
        &mut self,
        resolved: &ResolvedCommand,
        input: ValueStream,
        capture: bool,
    ) -> Result<(ValueStream, i32)> {
        commands::ensure_external_command(&resolved.command)?;
        let mut command = Command::new(&resolved.command);
        populate_command(
            &mut command,
            &self.cwd,
            &self.env,
            &resolved.env_overrides,
            &resolved.args,
            input.is_empty(),
            capture,
        );
        self.spawn_command(
            &mut command,
            input,
            capture,
            &format!("failed to run {}", resolved.command),
        )
    }

    pub(crate) fn execute_posix_shell_stage(
        &mut self,
        spec: &super::parser::CommandSpec,
        input: ValueStream,
        capture: bool,
    ) -> Result<(ValueStream, i32)> {
        let overrides = spec
            .assignments
            .iter()
            .map(|(name, value)| Ok((name.clone(), self.expand_value(value)?)))
            .collect::<Result<HashMap<_, _>>>()?;
        let args = self.expand_args(&spec.args)?;
        let mut command = Command::new("/bin/sh");
        populate_command(
            &mut command,
            &self.cwd,
            &self.env,
            &overrides,
            &args,
            input.is_empty(),
            capture,
        );
        self.spawn_command(&mut command, input, capture, "failed to spawn /bin/sh")
    }

    pub(crate) fn run_fallback(
        &mut self,
        source: &str,
        input: ValueStream,
        capture: bool,
    ) -> Result<(ValueStream, i32)> {
        let state = StatefulShellRun::new(source, &self.aliases)?;
        let mut command = Command::new("/bin/sh");
        command.arg(state.runner_path());
        state.populate_command_env(&mut command);
        populate_command(
            &mut command,
            &self.cwd,
            &self.env,
            &HashMap::new(),
            &[],
            input.is_empty(),
            capture,
        );
        let result = self.spawn_command(&mut command, input, capture, "failed to spawn /bin/sh")?;
        state.apply(self)?;
        Ok(result)
    }

    pub fn run_compiled_script(
        &mut self,
        origin: &Path,
        script: &str,
        args: &[String],
    ) -> Result<i32> {
        let mut command = Command::new("/bin/sh");
        command
            .arg("-s")
            .arg("--")
            .args(args)
            .current_dir(&self.cwd)
            .envs(&self.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        signal::prepare_foreground_command(&mut command);
        let _sigint = signal::SigintGuard::ignore()?;
        let mut child = command.spawn().with_context(|| {
            format!("failed to launch compiled script for {}", origin.display())
        })?;
        child
            .stdin
            .as_mut()
            .context("compiled script stdin unavailable")?
            .write_all(script.as_bytes())?;
        self.last_status = signal::exit_status(child.wait()?);
        Ok(self.last_status)
    }

    fn spawn_command(
        &mut self,
        command: &mut Command,
        input: ValueStream,
        capture: bool,
        context: &str,
    ) -> Result<(ValueStream, i32)> {
        signal::prepare_foreground_command(command);
        let _sigint = signal::SigintGuard::ignore()?;
        let mut child = command.spawn().with_context(|| context.to_string())?;
        write_input(&mut child, &input)?;
        finish_child(child, capture)
    }

    fn try_stylish(
        &self,
        resolved: &ResolvedCommand,
        input: &ValueStream,
    ) -> Result<Option<(ValueStream, i32)>> {
        match resolved.command.as_str() {
            "ls" => Ok(style::render_ls(&self.cwd, &resolved.args)?.map(|output| (output, 0))),
            "cat" => {
                Ok(style::render_cat(&self.cwd, &resolved.args, input)?.map(|output| (output, 0)))
            }
            "ps" => Ok(style::render_ps(&resolved.args)?.map(|output| (output, 0))),
            "kill" => Ok(style::render_kill(&resolved.args)?.map(|output| (output, 0))),
            "grep" => style::render_grep(&self.cwd, &resolved.args, input),
            "git" => Ok(style::render_git(&self.cwd, &resolved.args)?.map(|output| (output, 0))),
            "diff" => style::render_diff(&self.cwd, &resolved.args),
            _ => Ok(None),
        }
    }

    fn refresh_jobs(&mut self) -> Result<()> {
        for job in &mut self.jobs {
            if job.state != JobState::Running {
                continue;
            }
            if let Some(status) = job.child.try_wait()? {
                job.state = JobState::Done(signal::exit_status(status));
            }
        }
        Ok(())
    }

    fn parse_job_spec(&self, value: &str) -> Result<usize> {
        let trimmed = value.strip_prefix('%').unwrap_or(value);
        let id = trimmed
            .parse::<usize>()
            .with_context(|| format!("invalid job spec: {value}"))?;
        if self.job_position(id).is_none() {
            bail!("unknown job: %{id}");
        }
        Ok(id)
    }

    fn resolve_single_job_spec(&self, command: &str, args: &[String]) -> Result<usize> {
        if args.len() > 1 {
            bail!("{command} accepts at most one job spec");
        }
        match args.first() {
            Some(arg) => self.parse_job_spec(arg),
            None => self.current_job_id(),
        }
    }

    fn current_job_id(&self) -> Result<usize> {
        self.jobs
            .last()
            .map(|job| job.id)
            .ok_or_else(|| anyhow!("no jobs"))
    }

    fn job_position(&self, id: usize) -> Option<usize> {
        self.jobs.iter().position(|job| job.id == id)
    }

    fn wait_for_job(&mut self, id: usize) -> Result<i32> {
        let position = self
            .job_position(id)
            .ok_or_else(|| anyhow!("unknown job: %{id}"))?;
        let mut job = self.jobs.remove(position);
        let status = match job.state {
            JobState::Running => signal::exit_status(job.child.wait()?),
            JobState::Done(status) => status,
        };
        Ok(status)
    }

    fn remove_jobs(&mut self, ids: &[usize]) -> Result<()> {
        let mut positions = ids
            .iter()
            .map(|id| {
                self.job_position(*id)
                    .ok_or_else(|| anyhow!("unknown job: %{id}"))
            })
            .collect::<Result<Vec<_>>>()?;
        positions.sort_unstable();
        positions.dedup();
        for position in positions.into_iter().rev() {
            self.jobs.remove(position);
        }
        Ok(())
    }
}

fn populate_command(
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

fn write_input(child: &mut Child, input: &ValueStream) -> Result<()> {
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

fn finish_child(mut child: Child, capture: bool) -> Result<(ValueStream, i32)> {
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

impl JobState {
    fn label(self) -> &'static str {
        match self {
            JobState::Running => "Running",
            JobState::Done(0) => "Done",
            JobState::Done(_) => "Exit",
        }
    }
}

pub(crate) fn is_posix_shell_command(command: &str) -> bool {
    matches!(command, "sh" | "/bin/sh")
}
