use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn port_builtin_reports_listening_process_ids() {
    if !command_exists("lsof") || !command_exists("python3") {
        return;
    }

    let (mut server, port, _dir) = spawn_python_server();
    let output = ush()
        .args(["-c", &format!("port {port}")])
        .output()
        .expect("run ush");

    let _ = terminate_child(&mut server);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout
            .lines()
            .any(|line| line.trim() == server.id().to_string())
    );
}

#[test]
fn port_pipeline_stop_terminates_listening_process() {
    if !command_exists("lsof") || !command_exists("python3") {
        return;
    }

    let (mut server, port, _dir) = spawn_python_server();
    let output = ush()
        .args(["-c", &format!("port {port} | stop")])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let status = wait_for_exit(&mut server, Duration::from_secs(5))
        .unwrap_or_else(|| panic!("server process did not exit after stop"));
    assert!(!status.success());
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn spawn_python_server() -> (Child, u16, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let ready = dir.path().join("ready.txt");
    let child = Command::new("python3")
        .args(["-c", PYTHON_SERVER, ready.to_str().expect("utf8 path")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn python server");
    let port = read_port_file(&ready);
    (child, port, dir)
}

fn read_port_file(path: &Path) -> u16 {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(contents) = fs::read_to_string(path)
            && let Ok(port) = contents.trim().parse::<u16>()
        {
            return port;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("python server did not write port file");
}

fn wait_for_exit(child: &mut Child, timeout: Duration) -> Option<std::process::ExitStatus> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some(status) = child.try_wait().expect("try_wait") {
            return Some(status);
        }
        thread::sleep(Duration::from_millis(25));
    }
    None
}

fn terminate_child(child: &mut Child) -> Option<std::process::ExitStatus> {
    if let Some(status) = child.try_wait().expect("try_wait") {
        return Some(status);
    }
    child.kill().expect("kill child");
    child.wait().ok()
}

const PYTHON_SERVER: &str = r#"
import pathlib
import socket
import sys
import time

ready = pathlib.Path(sys.argv[1])
sock = socket.socket()
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
sock.bind(("127.0.0.1", 0))
sock.listen(1)
ready.write_text(str(sock.getsockname()[1]))

while True:
    time.sleep(1)
"#;
