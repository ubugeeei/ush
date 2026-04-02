use std::{fs, process::Command};

mod support;

use support::assert_snapshot;
use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn fixture(name: &str) -> String {
    format!("sammary/{name}.stdout")
}

#[test]
fn sammary_summarizes_globbed_files_and_totals() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("a.txt"), "a\nb\n").expect("write");
    fs::write(dir.path().join("b.txt"), "c\n").expect("write");

    let output = ush()
        .args(["-c", "sammary '*.txt'"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("plain"), &stdout);
}

#[test]
fn sammary_uses_rich_output_in_stylish_mode() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("app.rs"), "fn main() {}\n").expect("write");

    let output = ush()
        .args(["-s", "-c", "sammary '*.rs'"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish"), &stdout);
}

#[test]
fn sammary_recurse_walks_directories_and_directory_globs() {
    let dir = tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join("src/bin")).expect("mkdir");
    fs::write(dir.path().join("src/lib.rs"), "lib\n").expect("write");
    fs::write(dir.path().join("src/bin/main.rs"), "main\n").expect("write");

    for output in [
        ush()
            .args(["-c", "sammary src"])
            .current_dir(dir.path())
            .output(),
        ush()
            .args(["-c", "sammary 's*'"])
            .current_dir(dir.path())
            .output(),
    ] {
        let output = output.expect("run ush");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_snapshot(&fixture("recurse"), &stdout);
    }
}

#[test]
fn sammary_excludes_lock_files_by_default_and_can_restore_them() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("Cargo.lock"), "pkg\n").expect("write");
    fs::write(dir.path().join("package-lock.json"), "{ }\n").expect("write");
    fs::write(dir.path().join("main.rs"), "fn main() {}\n").expect("write");

    let default = ush()
        .args(["-c", "sammary ."])
        .current_dir(dir.path())
        .output()
        .expect("run ush");
    let included = ush()
        .args(["-c", "sammary --include-lock ."])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    let default_stdout = String::from_utf8_lossy(&default.stdout);
    assert!(default.status.success());
    assert!(default.stderr.is_empty());
    assert_snapshot(&fixture("default_excludes_lock"), &default_stdout);

    let included_stdout = String::from_utf8_lossy(&included.stdout);
    assert!(included.status.success());
    assert!(included.stderr.is_empty());
    assert_snapshot(&fixture("include_lock"), &included_stdout);
}
