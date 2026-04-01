use std::{fmt::Write as _, fs, process::Command};

use tempfile::tempdir;
use ush_compiler::UshCompiler;

fn render_trace(source: &str) -> String {
    let compiled = UshCompiler::default()
        .compile_source_with_sourcemap(source)
        .expect("compile ush program");
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("program.sh");
    fs::write(&script, &compiled.shell).expect("write script");

    let output = Command::new("/bin/sh")
        .arg(&script)
        .output()
        .expect("run compiled script");

    let mut report = String::new();
    report.push_str("== mapped shell ==\n");
    report.push_str(&compiled.sourcemap.render_mapped_listing());
    report.push_str("\n== runtime ==\n");
    let _ = writeln!(report, "status: {}", output.status.code().unwrap_or(-1));
    push_stream(&mut report, "stdout", &output.stdout);
    push_stream(&mut report, "stderr", &output.stderr);
    report
}

fn push_stream(report: &mut String, name: &str, bytes: &[u8]) {
    let _ = writeln!(report, "{name}:");
    let text = String::from_utf8_lossy(bytes);
    if text.is_empty() {
        report.push_str("(empty)\n");
        return;
    }
    report.push_str(&text);
    if !text.ends_with('\n') {
        report.push('\n');
    }
}

#[test]
fn compiled_scripts_can_snapshot_mapped_shell_and_runtime_io() {
    let trace = render_trace(concat!(
        "let greeting = \"hello\"\n",
        "if true {\n",
        "  print greeting\n",
        "}\n",
        "shell \"printf '%s\\n' warn >&2\"\n",
        "print greeting + \"!\"\n",
    ));

    assert_eq!(trace, include_str!("fixtures/io_snapshot.txt"),);
}
