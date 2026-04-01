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
            bol(),
        ),
        select(
            KeyCode::Down,
            Modifiers::CTRL_SHIFT,
            SelectionMove::LineEnd,
            eol(),
        ),
        select(
            KeyCode::Up,
            Modifiers::CTRL_ALT_SHIFT,
            SelectionMove::LineStart,
            bol(),
        ),
        select(
            KeyCode::Down,
            Modifiers::CTRL_ALT_SHIFT,
            SelectionMove::LineEnd,
            eol(),
        ),
        select(
            KeyCode::Left,
            Modifiers::SHIFT,
            SelectionMove::CharLeft,
            back_char(),
        ),
        select(
            KeyCode::Right,
            Modifiers::SHIFT,
            SelectionMove::CharRight,
            forward_char(),
        ),
        command(KeyCode::Left, Modifiers::ALT, back_word()),
        command(KeyCode::Right, Modifiers::ALT, forward_word()),
        command(KeyCode::Left, Modifiers::CTRL, back_word()),
        command(KeyCode::Right, Modifiers::CTRL, forward_word()),
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
            bol(),
        ),
        select(
            KeyCode::Right,
            Modifiers::CTRL_SHIFT,
            SelectionMove::LineEnd,
            eol(),
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
    bindings.extend(core_ctrl_bindings());
    bindings.extend(super_arrow_bindings());
    bindings.extend(home_end_bindings());
    bindings
}

fn core_ctrl_bindings() -> [BindingSpec; 9] {
    [
        command_char('A', Modifiers::CTRL, bol()),
        command_char('C', Modifiers::CTRL, Cmd::Interrupt),
        command_char('E', Modifiers::CTRL, eol()),
        command_char('K', Modifiers::CTRL, Cmd::Kill(Movement::EndOfLine)),
        command_char('L', Modifiers::CTRL, Cmd::ClearScreen),
        command_char('N', Modifiers::CTRL, Cmd::NextHistory),
        command_char('P', Modifiers::CTRL, Cmd::PreviousHistory),
        command_char('U', Modifiers::CTRL, Cmd::Kill(Movement::BeginningOfLine)),
        command_char('W', Modifiers::CTRL, Cmd::Kill(back_word_movement())),
    ]
}

fn super_arrow_bindings() -> Vec<BindingSpec> {
    let mut bindings = Vec::new();
    for code in [KeyCode::Left, KeyCode::Up] {
        bindings.push(command(code, Modifiers::SUPER, bol()));
        bindings.push(select(
            code,
            Modifiers::SUPER_SHIFT,
            SelectionMove::LineStart,
            bol(),
        ));
    }
    for code in [KeyCode::Right, KeyCode::Down] {
        bindings.push(command(code, Modifiers::SUPER, eol()));
        bindings.push(select(
            code,
            Modifiers::SUPER_SHIFT,
            SelectionMove::LineEnd,
            eol(),
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
        bindings.push(command(KeyCode::Home, mods, bol()));
        bindings.push(command(KeyCode::End, mods, eol()));
    }
    for mods in [
        Modifiers::SHIFT,
        Modifiers::ALT_SHIFT,
        Modifiers::CTRL_SHIFT,
        Modifiers::CTRL_ALT_SHIFT,
        Modifiers::SUPER_SHIFT,
    ] {
        bindings.push(select(KeyCode::Home, mods, SelectionMove::LineStart, bol()));
        bindings.push(select(KeyCode::End, mods, SelectionMove::LineEnd, eol()));
    }
    bindings
}

fn command(code: KeyCode, mods: Modifiers, cmd: Cmd) -> BindingSpec {
    bind_key(code, mods, BindingAction::Command(cmd))
}

fn command_char(ch: char, mods: Modifiers, cmd: Cmd) -> BindingSpec {
    bind_key(KeyCode::Char(ch), mods, BindingAction::Command(cmd))
}

fn select(code: KeyCode, mods: Modifiers, movement: SelectionMove, cmd: Cmd) -> BindingSpec {
    bind_key(code, mods, BindingAction::Select(movement, cmd))
}

fn select_word(code: KeyCode, mods: Modifiers, movement: SelectionMove) -> BindingSpec {
    select(
        code,
        mods,
        movement,
        match code {
            KeyCode::Left => back_word(),
            _ => forward_word(),
        },
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

fn bol() -> Cmd {
    Cmd::Move(Movement::BeginningOfLine)
}

fn eol() -> Cmd {
    Cmd::Move(Movement::EndOfLine)
}

fn back_char() -> Cmd {
    Cmd::Move(Movement::BackwardChar(1))
}

fn forward_char() -> Cmd {
    Cmd::Move(Movement::ForwardChar(1))
}

fn back_word() -> Cmd {
    Cmd::Move(back_word_movement())
}

fn forward_word() -> Cmd {
    Cmd::Move(Movement::ForwardWord(1, At::AfterEnd, Word::Emacs))
}

fn back_word_movement() -> Movement {
    Movement::BackwardWord(1, Word::Emacs)
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
