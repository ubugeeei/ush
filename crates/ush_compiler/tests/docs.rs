use std::{fs, process::Command};

use tempfile::tempdir;
use ush_compiler::UshCompiler;

const SOURCE: &str = r#"
#| Demo script.
#| Shows generated script docs.
#| @usage demo.ush --man greet
#| @example demo.ush --help

#| Greet a user.
#| @param name target user
#| @return greeting text
fn greet(name: String) -> String {
  return "hi " + name
}

print $ greet "ush"
"#;

#[test]
fn docs_render_help_and_man_sections() {
    let docs = UshCompiler::default().describe_source(SOURCE);

    let help = docs.render_help("demo.ush");
    let man = docs.render_man("demo.ush", Some("greet"));
    let completion = docs.render_completion("gr");

    assert!(help.contains("Demo script."));
    assert!(help.contains("demo.ush [--help]"));
    assert!(help.contains("fn greet(name: String) -> String"));
    assert!(man.contains("PARAMETERS"));
    assert!(man.contains("name - target user"));
    assert_eq!(completion, "greet\n");
}

#[test]
fn compiled_scripts_expose_help_man_and_completion() {
    let compiled = UshCompiler::default()
        .compile_source(SOURCE)
        .expect("compile ush program");
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("demo.sh");
    fs::write(&script, compiled).expect("write script");

    let help = Command::new("/bin/sh")
        .arg(&script)
        .arg("--help")
        .output()
        .expect("run help");
    let man = Command::new("/bin/sh")
        .arg(&script)
        .args(["--man", "greet"])
        .output()
        .expect("run man");
    let complete = Command::new("/bin/sh")
        .arg(&script)
        .args(["--complete", "gr"])
        .output()
        .expect("run completion");

    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("Documented items:"));
    assert!(man.status.success());
    assert!(String::from_utf8_lossy(&man.stdout).contains("RETURNS"));
    assert!(complete.status.success());
    assert_eq!(String::from_utf8_lossy(&complete.stdout), "greet\n");
}
