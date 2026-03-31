use std::{fs, process::Command};

use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

#[test]
fn ush_scripts_support_use_imports_and_std_modules() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("modules.ush");
    fs::write(
        &script,
        r#"
        use std::env::{get_or, set}
        use std::path::{exists, tmpfile}
        set "USH_STD_SMOKE" "ok"
        let file = tmpfile()
        print $ exists file
        print $ get_or "USH_STD_SMOKE" "missing"
        "#,
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "true\nok\n");
}

#[test]
fn ush_scripts_support_std_fs_and_command_helpers() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("system.ush");
    fs::write(
        &script,
        r#"
        use std::command::{capture, exists, status}
        use std::fs::{read_text, tmpfile, write_text}
        let path = tmpfile()
        write_text path "ready"
        print $ exists "sh"
        print $ capture "printf '%s\n' cmd"
        print $ read_text path
        print $ status "exit 3"
        "#,
    )
    .expect("write script");

    let output = ush().arg(&script).output().expect("run ush");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "true\ncmd\nready\n3\n"
    );
}
