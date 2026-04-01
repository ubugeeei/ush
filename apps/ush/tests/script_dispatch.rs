use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn sh_scripts_run_through_posix_sh_with_arguments() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("hello.sh");
    fs::write(
        &script,
        "#!/bin/sh\nprintf 'hello:%s\\n' \"$1\"\nprintf 'mode:%s\\n' \"$USH_STYLISH\"\n",
    )
    .expect("write script");

    let output = ush()
        .args(["-s", script.to_str().unwrap(), "world"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "hello:world\nmode:true\n"
    );
}
