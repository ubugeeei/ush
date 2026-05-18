#![cfg(unix)]

use std::{
    io,
    os::unix::process::CommandExt,
    process::{Command, Stdio},
    time::Duration,
};

mod common;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn sigint_interrupts_child_without_killing_ush_process() {
    let mut command = ush();
    command
        .args(["-c", "sleep 30"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let mut child = command.spawn().expect("spawn ush");
    assert!(
        wait_for_child_process(child.id(), Duration::from_secs(2)),
        "ush did not spawn child process in time",
    );
    let sent = unsafe { libc::kill(-(child.id() as i32), libc::SIGINT) };

    assert_eq!(sent, 0, "send sigint to ush process group");
    assert_eq!(child.wait().expect("wait ush").code(), Some(130));
}

fn wait_for_child_process(pid: u32, timeout: Duration) -> bool {
    let parent = pid.to_string();
    common::wait_until(timeout, || {
        let output = Command::new("ps")
            .args(["-axo", "ppid=,pid="])
            .output()
            .expect("run ps");
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .any(|ppid| ppid == parent)
    })
}
