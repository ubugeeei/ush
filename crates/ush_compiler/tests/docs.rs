use std::{fs, process::Command};

use tempfile::tempdir;
use ush_compiler::UshCompiler;

const SOURCE: &str = r#"
#| Demo script.
#|
#| Shows generated script docs in a more std-like shape, with a short summary,
#| longer description paragraphs, and explicit sections for warnings and links.
#| @usage demo.ush --man greet
#| @note `--help` stays compact, while `--man` expands the full guide.
#| @warning Generated docs are still experimental and may change between releases.
#| @see examples/docs.ush
#| @example demo.ush --help

#| Greet a user.
#|
#| This tiny function exists mostly to demonstrate rustdoc-style source comments.
#| The first line acts like a summary, and the following prose becomes detail text.
#| @param name target user
#| @return greeting text
#| @note Accepts plain user-facing names.
#| @warning Output is plain text, so callers should add escaping if they embed it in HTML.
#| @error This function does not raise typed `.ush` errors today.
#| @see demo.ush --man greet
#| @example demo.ush --man greet
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
    assert!(help.contains("Notes:"));
    assert!(help.contains("Warnings:"));
    assert!(help.contains("See also:"));
    assert!(help.contains("demo.ush [--help]"));
    assert!(help.contains("fn greet(name: String) -> String"));
    assert!(help.contains("errors: This function does not raise typed `.ush` errors today."));
    assert!(man.contains("PARAMETERS"));
    assert!(man.contains("NOTES"));
    assert!(man.contains("WARNINGS"));
    assert!(man.contains("ERRORS"));
    assert!(man.contains("SEE ALSO"));
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
    assert!(String::from_utf8_lossy(&help.stdout).contains("Warnings:"));
    assert!(man.status.success());
    assert!(String::from_utf8_lossy(&man.stdout).contains("RETURNS"));
    assert!(String::from_utf8_lossy(&man.stdout).contains("ERRORS"));
    assert!(complete.status.success());
    assert_eq!(String::from_utf8_lossy(&complete.stdout), "greet\n");
}
