use rustyline::{At, Cmd, Event, KeyCode, KeyEvent, Modifiers, Movement, Word};

use super::specs::{BindingAction, BindingSpec, binding_specs};
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
