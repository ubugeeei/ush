use std::path::PathBuf;

use rustyline::{Context, completion::Completer, hint::Hinter, history::DefaultHistory};
use tempfile::tempdir;

use crate::repl::{ReplJobCandidate, UshHelper};

fn helper(cwd: PathBuf) -> UshHelper {
    let mut helper = UshHelper::new(
        vec![
            "help".to_string(),
            "unset".to_string(),
            "source".to_string(),
            "rm".to_string(),
            "echo".to_string(),
            "grep".to_string(),
        ],
        vec!["HOME".to_string(), "PATH".to_string(), "PWD".to_string()],
        cwd,
    );
    helper.refresh(
        vec![
            "help".to_string(),
            "unset".to_string(),
            "source".to_string(),
            "rm".to_string(),
            "echo".to_string(),
            "grep".to_string(),
        ],
        vec!["HOME".to_string(), "PATH".to_string(), "PWD".to_string()],
        helper.cwd.clone(),
        vec!["ll".to_string()],
        vec![ReplJobCandidate {
            spec: "%1".to_string(),
            summary: "Running  sleep 10".to_string(),
        }],
    );
    helper
}

#[test]
fn completes_builtin_topics_and_env_names() {
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(PathBuf::from("."));

    let (_, help_pairs) = helper.complete("help so", 7, &ctx).expect("help complete");
    let (_, unset_pairs) = helper
        .complete("unset PA", 8, &ctx)
        .expect("unset complete");

    assert!(help_pairs.iter().any(|pair| pair.replacement == "source"));
    assert!(unset_pairs.iter().any(|pair| pair.replacement == "PATH"));
}

#[test]
fn completes_job_specs_and_signal_names() {
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(PathBuf::from("."));

    let (_, fg_pairs) = helper.complete("fg ", 3, &ctx).expect("fg complete");
    let (_, signal_pairs) = helper
        .complete("stop --signal KI", 16, &ctx)
        .expect("stop complete");

    assert!(fg_pairs.iter().any(|pair| pair.replacement == "%1"));
    assert!(signal_pairs.iter().any(|pair| pair.replacement == "KILL"));
}

#[test]
fn completes_source_paths_relative_to_shell_cwd() {
    let dir = tempdir().expect("tempdir");
    std::fs::write(dir.path().join("script.ush"), "echo ok\n").expect("write script");
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(dir.path().to_path_buf());

    let (_, pairs) = helper
        .complete("source sc", 9, &ctx)
        .expect("source complete");

    assert!(pairs.iter().any(|pair| pair.replacement == "script.ush"));
}

#[test]
fn hints_builtin_usage_and_top_level_builtin_summaries() {
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(PathBuf::from("."));

    let (_, command_pairs) = helper.complete("he", 2, &ctx).expect("complete");

    assert!(
        command_pairs.iter().any(|pair| {
            pair.replacement == "help" && pair.display.contains("show builtin help")
        })
    );
    assert_eq!(
        helper.hint("rm ", 3, &ctx),
        Some(
            "  guarded wrapper around rm for recursive deletes  next: <path>  usage: [--yes] [<path>...]".to_string()
        )
    );
}
