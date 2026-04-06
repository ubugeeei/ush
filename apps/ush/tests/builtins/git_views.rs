use super::*;

#[test]
fn stylish_git_status_renders_rich_output() {
    let dir = tempdir().expect("tempdir");
    init_git_repo(dir.path());
    run_git(dir.path(), &["checkout", "-q", "-b", "stylish-status"]);
    fs::write(dir.path().join("tracked.txt"), "hello\nworld\n").expect("update tracked");
    fs::write(dir.path().join("fresh.txt"), "new\n").expect("write untracked");

    let output = ush()
        .args(["-s", "-c", "git status"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot(&fixture("stylish_git_status"), &stdout);
}

#[test]
fn stylish_git_branch_renders_current_branch_without_tables() {
    let dir = tempdir().expect("tempdir");
    init_git_repo(dir.path());
    run_git(dir.path(), &["checkout", "-q", "-b", "stylish-branch"]);
    run_git(dir.path(), &["branch", "feature/ui"]);

    let output = ush()
        .args(["-s", "-c", "git branch"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = normalize_iso_dates(&normalize_git_hashes(&String::from_utf8_lossy(
        &output.stdout,
    )));
    assert_snapshot(&fixture("stylish_git_branch"), &stdout);
}

#[test]
fn stylish_git_log_renders_recent_commits() {
    let dir = tempdir().expect("tempdir");
    init_git_repo(dir.path());
    run_git(dir.path(), &["checkout", "-q", "-b", "stylish-log"]);
    fs::write(dir.path().join("tracked.txt"), "hello\nworld\n").expect("update tracked");
    run_git(dir.path(), &["commit", "-qam", "second pass"]);

    let output = ush()
        .args(["-s", "-c", "git log -2"])
        .current_dir(dir.path())
        .output()
        .expect("run ush");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = normalize_iso_dates(&normalize_git_hashes(&String::from_utf8_lossy(
        &output.stdout,
    )));
    assert_snapshot(&fixture("stylish_git_log"), &stdout);
}
