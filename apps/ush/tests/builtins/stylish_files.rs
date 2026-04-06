use super::*;

#[test]
fn stylish_ls_a_includes_dot_entries_and_hidden_files() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join(".hidden"), "secret\n").expect("write");
    fs::write(dir.path().join("visible"), "open\n").expect("write");

    let output = ush()
        .args(["-s", "-c", "ls -a"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_all"), &stdout);
}

#[test]
fn stylish_ls_combined_short_flags_keep_rich_output() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join(".hidden"), "secret\n").expect("write");
    fs::write(dir.path().join("visible"), "open\n").expect("write");

    let output = ush()
        .args(["-s", "-c", "ls -lah"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_lah"), &stdout);
}

#[test]
fn stylish_ls_almost_all_shows_hidden_without_dot_entries() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join(".hidden"), "secret\n").expect("write");
    fs::write(dir.path().join("visible"), "open\n").expect("write");

    let output = ush()
        .args(["-s", "-c", "ls -A"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_almost_all"), &stdout);
}

#[test]
fn stylish_ls_keeps_explicit_hidden_targets_visible_without_all_flag() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join(".env"), "TOKEN=ush\n").expect("write");

    let output = ush()
        .args(["-s", "-c", "ls .env"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_hidden_target"), &stdout);
}

#[test]
fn stylish_ls_handles_broken_symlinks() {
    let dir = tempdir().expect("tempdir");
    symlink("missing-target", dir.path().join("broken-link")).expect("symlink");

    let output = ush()
        .args(["-s", "-c", "ls"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    let stdout = normalize_ls_output(&String::from_utf8_lossy(&output.stdout));
    assert!(output.stderr.is_empty());
    assert_snapshot(&fixture("stylish_ls_broken_symlink"), &stdout);
}

#[test]
fn stylish_diff_renders_hunks_and_preserves_exit_code() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("before.txt"), "alpha\nbeta\n").expect("write before");
    fs::write(dir.path().join("after.txt"), "alpha\ngamma\n").expect("write after");

    let output = ush()
        .args(["-s", "-c", "diff before.txt after.txt"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_diff"), &stdout);
}

#[test]
fn stylish_diff_unified_flag_keeps_rich_output() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("before.txt"), "alpha\nbeta\n").expect("write before");
    fs::write(dir.path().join("after.txt"), "alpha\ngamma\n").expect("write after");

    let output = ush()
        .args(["-s", "-c", "diff -u before.txt after.txt"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_diff_u"), &stdout);
}

#[test]
fn stylish_grep_groups_file_matches() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("sample.txt"),
        "alpha\nfoo first\nbeta\nfoo second\n",
    )
    .expect("write sample");

    let output = ush()
        .args(["-s", "-c", "grep foo sample.txt"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_grep_file"), &stdout);
}

#[test]
fn stylish_grep_reads_pipeline_input() {
    let output = ush()
        .args(["-s", "-c", "printf 'alpha\nfoo\nbeta\n' | grep foo"])
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_grep_stdin"), &stdout);
}

#[test]
fn stylish_grep_no_matches_preserves_exit_code() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("sample.txt"), "alpha\nbeta\n").expect("write sample");

    let output = ush()
        .args(["-s", "-c", "grep foo sample.txt"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_grep_no_matches"), &stdout);
}
