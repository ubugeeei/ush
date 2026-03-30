use rustyline::{Context, completion::Completer, hint::Hinter, history::History};
use tempfile::tempdir;

use super::{UshHelper, create_editor};

#[test]
fn history_hint_prefers_previous_entries() {
    let dir = tempdir().expect("tempdir");
    let history_file = dir.path().join("history.txt");
    let mut editor = create_editor(
        &history_file,
        10,
        vec!["echo".to_string()],
        vec!["PATH".to_string()],
    )
    .expect("editor");
    editor.add_history_entry("echo hello").expect("history");
    let ctx = Context::new(editor.history());
    let helper = UshHelper::new(vec!["echo".to_string()], vec!["PATH".to_string()]);

    assert_eq!(helper.hint("echo h", 6, &ctx), Some("ello".to_string()));
}

#[test]
fn tab_completion_exposes_selection_hint() {
    let history = rustyline::history::DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = UshHelper::new(
        vec!["git".to_string(), "grep".to_string()],
        vec!["PATH".to_string()],
    );
    helper.complete("g", 1, &ctx).expect("complete");

    assert_eq!(
        helper.hint("git", 3, &ctx),
        Some("  [1/2] tab: next  shift-tab: prev  enter: accept".to_string())
    );
}

#[test]
fn editor_respects_history_limit() {
    let dir = tempdir().expect("tempdir");
    let history_file = dir.path().join("history.txt");
    let mut editor = create_editor(
        &history_file,
        2,
        vec!["echo".to_string()],
        vec!["PATH".to_string()],
    )
    .expect("editor");

    editor.add_history_entry("echo one").expect("history");
    editor.add_history_entry("echo two").expect("history");
    editor.add_history_entry("echo three").expect("history");

    assert_eq!(editor.history().len(), 2);
}
