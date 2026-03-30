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
fn named_arguments_and_defaults_work_in_any_order() {
    let output = run_program(
        r#"
        fn greet(name: String, #[default(2)] count: Int) -> String {
          return name + ":" + count
        }
        print $ greet count: 3 name: "ush"
        print $ greet name: "mini"
    "#,
    );

    assert_eq!(output, "ush:3\nmini:2\n");
}

#[test]
fn type_declarations_lower_like_structs() {
    let output = run_program(
        r#"
        type User {
          name: String,
          age: Int,
        }
        let user = User { name: "ush", age: 7 }
        match user {
          User { name, age } => print name + ":" + age
          _ => print "fallback"
        }
    "#,
    );

    assert_eq!(output, "ush:7\n");
}

#[test]
fn alias_declarations_lower_to_shell_aliases() {
    let compiled = UshCompiler::default()
        .compile_source(
            r#"
            alias ll = "ls -la"
        "#,
        )
        .expect("compile");

    assert!(compiled.contains("alias ll='ls -la'"));
}

#[test]
fn bin_scripts_generate_cli_wrapper_and_completions() {
    let dir = tempdir().expect("tempdir");
    let script = dir.path().join("bin.ush");
    fs::write(
        &script,
        r#"
        #| Demo CLI.
        fn bin(#[alias("n")] name: String, #[default(2)] count: Int, verbose: Bool) {
          print name + ":" + count
          print verbose
        }
    "#,
    )
    .expect("write bin.ush");

    let compiled = UshCompiler::default()
        .compile_file(&script)
        .expect("compile file");

    assert!(compiled.contains("__ush_run_bin()"));
    assert!(!compiled.contains("'--name'|' -n'"));
    assert!(compiled.contains("'--name'|'-n'"));
    assert!(compiled.contains("'--count'"));
    assert!(compiled.contains("'--verbose'"));
    assert!(compiled.contains("__ush_complete"));
}
