use std::process::Command;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn example_path(name: &str) -> String {
    format!("{}/../../examples/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn ush_scripts_support_multiline_strings() {
    let output = ush()
        .arg(example_path("multiline_string.ush"))
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "<article>\n  hello\n</article>\ntrue\n"
    );
}
