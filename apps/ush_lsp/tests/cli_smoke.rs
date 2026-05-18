//! Smoke tests for the LSP server's command-line surface.
//!
//! These do not speak full LSP — that is exercised by the upstream
//! lsp-server crate's own integration tests — but they pin the
//! `--version` / `--help` / unknown-flag contract that operators rely
//! on when packaging or scripting around the binary.

use std::process::Command;

fn ush_lsp() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush_lsp"))
}

#[test]
fn prints_version() {
    let out = ush_lsp().arg("--version").output().expect("spawn ush_lsp");
    assert!(out.status.success(), "exited {:?}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with("ush_lsp "),
        "expected `ush_lsp <version>` on stdout, got: {stdout}"
    );
}

#[test]
fn short_version_flag_works() {
    let out = ush_lsp().arg("-V").output().expect("spawn ush_lsp");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("ush_lsp "));
}

#[test]
fn help_describes_stdio_mode() {
    let out = ush_lsp().arg("--help").output().expect("spawn ush_lsp");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("speak LSP over stdio"),
        "help must mention stdio mode, got: {stdout}"
    );
}

#[test]
fn unknown_flag_is_rejected() {
    let out = ush_lsp()
        .arg("--definitely-not-a-real-flag")
        .output()
        .expect("spawn ush_lsp");
    assert!(
        !out.status.success(),
        "unknown flag must cause a non-zero exit"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown argument"),
        "expected `unknown argument` on stderr, got: {stderr}"
    );
}
