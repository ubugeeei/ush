use std::{fs, process::Command};

use tempfile::tempdir;
use ush_compiler::UshCompiler;

fn run_program(source: &str) -> String {
    let compiled = UshCompiler::default()
        .compile_source(source)
        .expect("compile ush program");
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("program.sh");
    fs::write(&script, compiled).expect("write script");

    let output = Command::new("/bin/sh")
        .arg(&script)
        .output()
        .expect("run compiled script");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn inline_shell_statement_runs_verbatim_command() {
    let output = run_program("let greeting = \"hello\"\n$ printf '%s\\n' \"$greeting\"\n");
    assert_eq!(output, "hello\n");
}

#[test]
fn inline_shell_statement_is_allowed_in_match_arms() {
    let output = run_program(
        "let greeting = \"hello\"\nmatch greeting {\n  \"hello\" => $ printf '%s\\n' matched\n  _ => print \"fallback\"\n}\n",
    );
    assert_eq!(output, "matched\n");
}
