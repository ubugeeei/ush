use std::io;
#[cfg(unix)]
use std::{os::unix::process::CommandExt, process::Command};

#[cfg(unix)]
pub(crate) struct SigintGuard(libc::sighandler_t);

#[cfg(unix)]
impl SigintGuard {
    pub(crate) fn ignore() -> io::Result<Self> {
        let previous = unsafe { libc::signal(libc::SIGINT, libc::SIG_IGN) };
        if previous == libc::SIG_ERR {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(previous))
    }
}

#[cfg(unix)]
impl Drop for SigintGuard {
    fn drop(&mut self) {
        let _ = unsafe { libc::signal(libc::SIGINT, self.0) };
    }
}

#[cfg(not(unix))]
pub(crate) struct SigintGuard;

#[cfg(not(unix))]
impl SigintGuard {
    pub(crate) fn ignore() -> io::Result<Self> {
        Ok(Self)
    }
}

#[cfg(unix)]
pub(crate) fn prepare_foreground_command(command: &mut Command) {
    unsafe {
        command.pre_exec(|| {
            let restored = libc::signal(libc::SIGINT, libc::SIG_DFL);
            if restored == libc::SIG_ERR {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(unix)]
pub(crate) fn prepare_background_command(command: &mut Command) {
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(io::Error::last_os_error());
            }
            let restored = libc::signal(libc::SIGINT, libc::SIG_DFL);
            if restored == libc::SIG_ERR {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(not(unix))]
pub(crate) fn prepare_foreground_command(_: &mut std::process::Command) {}

#[cfg(not(unix))]
pub(crate) fn prepare_background_command(_: &mut std::process::Command) {}

#[cfg(unix)]
pub(crate) fn continue_background_job(pid: u32) -> io::Result<()> {
    let result = unsafe { libc::kill(-(pid as i32), libc::SIGCONT) };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn continue_background_job(_: u32) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
pub(crate) fn exit_status(status: std::process::ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;

    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(1))
}

#[cfg(not(unix))]
pub(crate) fn exit_status(status: std::process::ExitStatus) -> i32 {
    status.code().unwrap_or(1)
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    #[test]
    fn converts_sigint_into_shell_style_exit_code() {
        let status = std::process::Command::new("/bin/sh")
            .arg("-c")
            .arg("kill -INT $$")
            .status()
            .expect("run shell");

        assert_eq!(super::exit_status(status), 130);
    }
}
