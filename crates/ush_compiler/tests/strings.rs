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
fn multiline_strings_are_dedented() {
    let output = run_program(
        r#"
        let page = """
          <div>
            hello
          </div>
        """
        print page
        print $ page.starts_with("<div>")
        print $ page.ends_with("</div>")
      "#,
    );

    assert_eq!(output, "<div>\n  hello\n</div>\ntrue\ntrue\n");
}
