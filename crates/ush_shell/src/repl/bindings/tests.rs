use rustyline::{At, Cmd, Event, KeyCode, KeyEvent, Modifiers, Movement, Word};

use super::specs::{BindingAction, BindingSpec, binding_specs};
use super::{selection_edit_command, should_keep_selection};
use crate::repl::selection::SelectionDelete;
use crate::repl::selection::SelectionMove;

#[test]
fn includes_selection_shortcuts_for_shift_and_option() {
    let bindings = binding_specs();

    assert!(bindings.contains(&BindingSpec {
        event: Event::from(KeyEvent(KeyCode::Left, Modifiers::SHIFT)),
        action: BindingAction::Select(
            SelectionMove::CharLeft,
            Cmd::Move(Movement::BackwardChar(1)),
        ),
    }));
    assert!(bindings.contains(&BindingSpec {
        event: Event::from(KeyEvent(KeyCode::Right, Modifiers::ALT_SHIFT)),
        action: BindingAction::Select(
            SelectionMove::WordRight,
            Cmd::Move(Movement::ForwardWord(1, At::AfterEnd, Word::Emacs)),
        ),
    }));
    assert!(bindings.contains(&BindingSpec {
        event: Event::from(KeyEvent(KeyCode::Right, Modifiers::CTRL_SHIFT)),
        action: BindingAction::Select(SelectionMove::LineEnd, Cmd::Move(Movement::EndOfLine),),
    }));
}

#[test]
fn includes_history_and_home_end_variants() {
    let bindings = binding_specs();

    assert!(bindings.contains(&BindingSpec {
        event: Event::from(KeyEvent(KeyCode::Down, Modifiers::ALT)),
        action: BindingAction::Command(Cmd::HistorySearchForward),
    }));
    assert!(bindings.contains(&BindingSpec {
        event: Event::from(KeyEvent(KeyCode::Home, Modifiers::SHIFT)),
        action: BindingAction::Select(
            SelectionMove::LineStart,
            Cmd::Move(Movement::BeginningOfLine),
        ),
    }));
    assert!(bindings.contains(&BindingSpec {
        event: Event::from(KeyEvent(KeyCode::End, Modifiers::CTRL_ALT_SHIFT)),
        action: BindingAction::Select(SelectionMove::LineEnd, Cmd::Move(Movement::EndOfLine),),
    }));
}

#[test]
fn replaces_selected_text_on_plain_character_input() {
    let command = selection_edit_command(
        &Event::from(KeyEvent::new('x', Modifiers::NONE)),
        SelectionDelete::Forward(2),
    );

    assert_eq!(
        command,
        Some(Cmd::Replace(
            Movement::ForwardChar(2),
            Some("x".to_string())
        ))
    );
}

#[test]
fn deletes_selected_text_on_ctrl_h_backspace() {
    let command = selection_edit_command(
        &Event::from(KeyEvent(KeyCode::Char('H'), Modifiers::CTRL)),
        SelectionDelete::Forward(4),
    );

    assert_eq!(command, Some(Cmd::Kill(Movement::ForwardChar(4))));
}

#[test]
fn deletes_selected_text_on_ctrl_d() {
    let command = selection_edit_command(
        &Event::from(KeyEvent(KeyCode::Char('D'), Modifiers::CTRL)),
        SelectionDelete::Backward(3),
    );

    assert_eq!(command, Some(Cmd::Kill(Movement::BackwardChar(3))));
}

#[test]
fn preserves_selection_on_unknown_escape_sequences() {
    assert!(should_keep_selection(&Event::from(KeyEvent(
        KeyCode::UnknownEscSeq,
        Modifiers::NONE,
    ))));
    assert!(should_keep_selection(&Event::from(KeyEvent(
        KeyCode::Null,
        Modifiers::NONE,
    ))));
}
