use std::{fs, process::Command};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

mod support;

use support::assert_snapshot;
use tempfile::tempdir;

fn ush() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ush"))
}

fn fixture(name: &str) -> String {
    format!("html/{name}.stdout")
}

#[test]
fn html_helper_opens_pipeline_output_in_browser() {
    let dir = tempdir().expect("tempdir");
    let log = dir.path().join("opened-path.txt");
    install_fake_openers(dir.path(), &log);

    let output = ush()
        .env("PATH", prefixed_path(dir.path()))
        .args(["-c", "printf '<html><body>ok</body></html>' | html"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert!(output.stderr.is_empty());
    let opened = fs::read_to_string(&log).expect("read log");
    let html = fs::read_to_string(opened.trim()).expect("read html");
    assert_snapshot(&fixture("html_helper"), &html);
}

#[test]
fn json_helper_falls_back_to_browser_for_html_input() {
    let dir = tempdir().expect("tempdir");
    let log = dir.path().join("opened-path.txt");
    install_fake_openers(dir.path(), &log);

    let output = ush()
        .env("PATH", prefixed_path(dir.path()))
        .args(["-c", "printf '<div>fallback</div>' | json"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert!(output.stderr.is_empty());
    let opened = fs::read_to_string(&log).expect("read log");
    let html = fs::read_to_string(opened.trim()).expect("read html");
    assert_snapshot(&fixture("json_fallback"), &html);
}

#[test]
fn xml_helper_pretty_prints_valid_xml() {
    let output = ush()
        .args(["-c", "printf '<root><item>ok</item></root>' | xml"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("xml_pretty"), &stdout);
}

#[test]
fn xml_helper_falls_back_to_browser_for_invalid_xml() {
    let dir = tempdir().expect("tempdir");
    let log = dir.path().join("opened-path.txt");
    install_fake_openers(dir.path(), &log);

    let output = ush()
        .env("PATH", prefixed_path(dir.path()))
        .args(["-c", "printf '<div></span>' | xml"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert!(output.stderr.is_empty());
    let opened = fs::read_to_string(&log).expect("read log");
    let html = fs::read_to_string(opened.trim()).expect("read html");
    assert_snapshot(&fixture("xml_fallback"), &html);
}

fn install_fake_openers(bin_dir: &std::path::Path, log: &std::path::Path) {
    for name in ["open", "xdg-open"] {
        let path = bin_dir.join(name);
        fs::write(
            &path,
            format!("#!/bin/sh\nprintf '%s' \"$1\" > '{}'\n", log.display()),
        )
        .expect("write fake opener");
        #[cfg(unix)]
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
}

fn prefixed_path(bin_dir: &std::path::Path) -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    format!("{}:{current}", bin_dir.display())
}
