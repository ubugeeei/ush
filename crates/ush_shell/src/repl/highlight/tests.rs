use rustyline::CompletionType;

use crate::repl::selection::SelectionMove;
use crate::repl::{
    UshHelper,
    highlight::{highlight_hint, highlight_line, highlight_prompt},
};

#[test]
fn highlights_commands_variables_and_comments() {
    let helper = UshHelper::new(
        vec!["echo".to_string(), "grep".to_string()],
        vec!["PATH".to_string()],
        std::env::temp_dir(),
    );
    let line = highlight_line(&helper, "echo $PATH # note");

    assert!(line.contains("\u{1b}[1;38;5;111mecho\u{1b}[0m"));
    assert!(line.contains("\u{1b}[1;38;5;151m$PATH\u{1b}[0m"));
    assert!(line.contains("\u{1b}[2;38;5;244m# note\u{1b}[0m"));
}

#[test]
fn highlights_default_prompt_by_segment() {
    let prompt = highlight_prompt("~/.../ubugeeei/ush $ ");

    assert!(prompt.contains("\u{1b}[1;38;5;223m~/.../ubugeeei/ush\u{1b}[0m"));
    assert!(prompt.contains("\u{1b}[1;38;5;150m$\u{1b}[0m"));
}

#[test]
fn uses_soft_hint_color() {
    assert_eq!(
        highlight_hint("suffix"),
        "\u{1b}[2;38;5;245msuffix\u{1b}[0m"
    );
}

#[test]
fn highlights_selected_region() {
    let helper = UshHelper::new(vec!["echo".to_string()], vec![], std::env::temp_dir());
    helper
        .selection_handle()
        .extend("echo hello", 5, SelectionMove::WordRight);

    let line = highlight_line(&helper, "echo hello");

    assert!(line.contains("\u{1b}[48;5;239;38;5;255mhello\u{1b}[0m"));
}

#[test]
fn highlights_active_circular_candidate() {
    let helper = UshHelper::new(vec!["echo".to_string()], vec![], std::env::temp_dir());
    let candidate = super::highlight_candidate(&helper, "echo", CompletionType::Circular, true);

    assert!(candidate.contains("\u{1b}[1;48;5;111;38;5;235m"));
}
