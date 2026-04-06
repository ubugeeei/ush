use std::{path::Path, process::Command};

use rustyline::completion::Pair;

use super::{dedupe_pairs, described_pair};

pub(super) fn local_branch_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    ref_pairs_for_namespace(cwd, "refs/heads", needle, "local branch")
}

pub(super) fn remote_branch_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    ref_pairs_for_namespace(cwd, "refs/remotes", needle, "remote branch")
        .into_iter()
        .filter(|pair| !pair.replacement.ends_with("/HEAD"))
        .collect()
}

pub(super) fn tag_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    ref_pairs_for_namespace(cwd, "refs/tags", needle, "tag")
}

pub(super) fn branch_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    let mut pairs = local_branch_pairs(cwd, needle);
    pairs.extend(remote_branch_pairs(cwd, needle));
    pairs.extend(tag_pairs(cwd, needle));
    dedupe_pairs(pairs)
}

pub(super) fn ref_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    let mut pairs = branch_pairs(cwd, needle);
    pairs.extend(recent_commit_pairs(cwd, needle));
    dedupe_pairs(pairs)
}

pub(super) fn commit_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    let mut pairs = recent_commit_pairs(cwd, needle);
    pairs.extend(branch_pairs(cwd, needle));
    dedupe_pairs(pairs)
}

pub(super) fn remote_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    git_rows(cwd, &["remote"])
        .into_iter()
        .filter(|remote| remote.starts_with(needle))
        .map(|remote| described_pair(&remote, "remote"))
        .collect()
}

pub(super) fn stash_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    git_rows(cwd, &["stash", "list", "--format=%gd%x09%s"])
        .into_iter()
        .filter_map(|row| {
            let (name, subject) = split_row(&row);
            name.starts_with(needle)
                .then(|| described_pair(&name, &format!("stash  {subject}")))
        })
        .collect()
}

fn ref_pairs_for_namespace(cwd: &Path, namespace: &str, needle: &str, label: &str) -> Vec<Pair> {
    git_rows(
        cwd,
        &[
            "for-each-ref",
            "--format=%(refname:short)%09%(subject)",
            namespace,
        ],
    )
    .into_iter()
    .filter_map(|row| {
        let (name, subject) = split_row(&row);
        name.starts_with(needle).then(|| {
            let summary = if subject.is_empty() {
                label.to_string()
            } else {
                format!("{label}  {subject}")
            };
            described_pair(&name, &summary)
        })
    })
    .collect()
}

fn recent_commit_pairs(cwd: &Path, needle: &str) -> Vec<Pair> {
    git_rows(cwd, &["log", "--format=%h%x09%s", "-n", "30", "--all"])
        .into_iter()
        .filter_map(|row| {
            let (sha, subject) = split_row(&row);
            sha.starts_with(needle)
                .then(|| described_pair(&sha, &subject))
        })
        .collect()
}

fn git_rows(cwd: &Path, args: &[&str]) -> Vec<String> {
    let Ok(output) = Command::new("git").current_dir(cwd).args(args).output() else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn split_row(row: &str) -> (String, String) {
    row.split_once('\t')
        .map(|(left, right)| (left.to_string(), right.to_string()))
        .unwrap_or_else(|| (row.to_string(), String::new()))
}
