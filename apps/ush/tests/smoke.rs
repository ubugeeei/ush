pub(crate) use std::fs;
use std::process::Command;

#[path = "smoke/basic_cases.rs"]
mod basic_cases;
#[path = "smoke/script_cases.rs"]
mod script_cases;
mod support;

pub(crate) use support::assert_snapshot;
pub(crate) use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'\''"#))
}

fn fixture(name: &str) -> String {
    format!("smoke/{name}.stdout")
}

fn normalize_path(text: &str, path: &std::path::Path, marker: &str) -> String {
    text.replace(&path.display().to_string(), marker)
}
