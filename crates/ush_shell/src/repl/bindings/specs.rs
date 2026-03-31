use rustyline::{At, Cmd, Event, KeyCode, KeyEvent, Modifiers, Movement, Word};

use crate::repl::selection::SelectionMove;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BindingSpec {
    pub(super) event: Event,
    pub(super) action: BindingAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum BindingAction {
    Command(Cmd),
    Select(SelectionMove, Cmd),
}

pub(super) fn binding_specs() -> Vec<BindingSpec> {
    let mut bindings = vec![
        command(
            KeyCode::Up,
            Modifiers::SHIFT,
            Cmd::LineUpOrPreviousHistory(1),
        ),
        command(
            KeyCode::Down,
            Modifiers::SHIFT,
            Cmd::LineDownOrNextHistory(1),
        ),
        command(KeyCode::Up, Modifiers::ALT, Cmd::HistorySearchBackward),
        command(KeyCode::Down, Modifiers::ALT, Cmd::HistorySearchForward),
        command(KeyCode::Up, Modifiers::CTRL, Cmd::BeginningOfHistory),
        command(KeyCode::Down, Modifiers::CTRL, Cmd::EndOfHistory),
        select(
            KeyCode::Up,
            Modifiers::CTRL_SHIFT,
            SelectionMove::LineStart,
            Cmd::Move(Movement::BeginningOfLine),
        ),
        select(
            KeyCode::Down,
            Modifiers::CTRL_SHIFT,
            SelectionMove::LineEnd,
            Cmd::Move(Movement::EndOfLine),
        ),
        select(
            KeyCode::Up,
            Modifiers::CTRL_ALT_SHIFT,
            SelectionMove::LineStart,
            Cmd::Move(Movement::BeginningOfLine),
        ),
        select(
            KeyCode::Down,
            Modifiers::CTRL_ALT_SHIFT,
            SelectionMove::LineEnd,
            Cmd::Move(Movement::EndOfLine),
        ),
        select(
            KeyCode::Left,
            Modifiers::SHIFT,
            SelectionMove::CharLeft,
            Cmd::Move(Movement::BackwardChar(1)),
        ),
        select(
            KeyCode::Right,
            Modifiers::SHIFT,
            SelectionMove::CharRight,
            Cmd::Move(Movement::ForwardChar(1)),
        ),
        command(
            KeyCode::Left,
            Modifiers::ALT,
            Cmd::Move(Movement::BackwardWord(1, Word::Emacs)),
        ),
        command(
            KeyCode::Right,
            Modifiers::ALT,
            Cmd::Move(Movement::ForwardWord(1, At::AfterEnd, Word::Emacs)),
        ),
        command(
            KeyCode::Left,
            Modifiers::CTRL,
            Cmd::Move(Movement::BackwardWord(1, Word::Emacs)),
        ),
        command(
            KeyCode::Right,
            Modifiers::CTRL,
            Cmd::Move(Movement::ForwardWord(1, At::AfterEnd, Word::Emacs)),
        ),
        select_word(KeyCode::Left, Modifiers::ALT_SHIFT, SelectionMove::WordLeft),
        select_word(
            KeyCode::Right,
            Modifiers::ALT_SHIFT,
            SelectionMove::WordRight,
        ),
        select(
            KeyCode::Left,
            Modifiers::CTRL_SHIFT,
            SelectionMove::LineStart,
            Cmd::Move(Movement::BeginningOfLine),
        ),
        select(
            KeyCode::Right,
            Modifiers::CTRL_SHIFT,
            SelectionMove::LineEnd,
            Cmd::Move(Movement::EndOfLine),
        ),
        select_big_word(KeyCode::Left, SelectionMove::BigWordLeft),
        select_big_word(KeyCode::Right, SelectionMove::BigWordRight),
        bind_char(
            '<',
            Modifiers::ALT,
            BindingAction::Command(Cmd::BeginningOfHistory),
        ),
        bind_char(
            '>',
            Modifiers::ALT,
            BindingAction::Command(Cmd::EndOfHistory),
        ),
    ];
    bindings.extend(super_arrow_bindings());
    bindings.extend(home_end_bindings());
    bindings
}

fn super_arrow_bindings() -> Vec<BindingSpec> {
    let mut bindings = Vec::new();
    for code in [KeyCode::Left, KeyCode::Up] {
        bindings.push(command(
            code,
            Modifiers::SUPER,
            Cmd::Move(Movement::BeginningOfLine),
        ));
        bindings.push(select(
            code,
            Modifiers::SUPER_SHIFT,
            SelectionMove::LineStart,
            Cmd::Move(Movement::BeginningOfLine),
        ));
    }
    for code in [KeyCode::Right, KeyCode::Down] {
        bindings.push(command(
            code,
            Modifiers::SUPER,
            Cmd::Move(Movement::EndOfLine),
        ));
        bindings.push(select(
            code,
            Modifiers::SUPER_SHIFT,
            SelectionMove::LineEnd,
            Cmd::Move(Movement::EndOfLine),
        ));
    }
    bindings
}

fn home_end_bindings() -> Vec<BindingSpec> {
    let mut bindings = Vec::new();
    for mods in [
        Modifiers::NONE,
        Modifiers::ALT,
        Modifiers::CTRL,
        Modifiers::CTRL_ALT,
        Modifiers::SUPER,
    ] {
        bindings.push(command(
            KeyCode::Home,
            mods,
            Cmd::Move(Movement::BeginningOfLine),
        ));
        bindings.push(command(KeyCode::End, mods, Cmd::Move(Movement::EndOfLine)));
    }
    for mods in [
        Modifiers::SHIFT,
        Modifiers::ALT_SHIFT,
        Modifiers::CTRL_SHIFT,
        Modifiers::CTRL_ALT_SHIFT,
        Modifiers::SUPER_SHIFT,
    ] {
        bindings.push(select(
            KeyCode::Home,
            mods,
            SelectionMove::LineStart,
            Cmd::Move(Movement::BeginningOfLine),
        ));
        bindings.push(select(
            KeyCode::End,
            mods,
            SelectionMove::LineEnd,
            Cmd::Move(Movement::EndOfLine),
        ));
    }
    bindings
}

fn command(code: KeyCode, mods: Modifiers, cmd: Cmd) -> BindingSpec {
    bind_key(code, mods, BindingAction::Command(cmd))
}

fn select(code: KeyCode, mods: Modifiers, movement: SelectionMove, cmd: Cmd) -> BindingSpec {
    bind_key(code, mods, BindingAction::Select(movement, cmd))
}

fn select_word(code: KeyCode, mods: Modifiers, movement: SelectionMove) -> BindingSpec {
    select(
        code,
        mods,
        movement,
        Cmd::Move(match code {
            KeyCode::Left => Movement::BackwardWord(1, Word::Emacs),
            _ => Movement::ForwardWord(1, At::AfterEnd, Word::Emacs),
        }),
    )
}

fn select_big_word(code: KeyCode, movement: SelectionMove) -> BindingSpec {
    select(
        code,
        Modifiers::CTRL_ALT_SHIFT,
        movement,
        Cmd::Move(match code {
            KeyCode::Left => Movement::BackwardWord(1, Word::Big),
            _ => Movement::ForwardWord(1, At::AfterEnd, Word::Big),
        }),
    )
}

fn bind_key(code: KeyCode, mods: Modifiers, action: BindingAction) -> BindingSpec {
    BindingSpec {
        event: Event::from(KeyEvent(code, mods)),
        action,
    }
}

fn bind_char(ch: char, mods: Modifiers, action: BindingAction) -> BindingSpec {
    BindingSpec {
        event: Event::from(KeyEvent::new(ch, mods)),
        action,
    }
}
