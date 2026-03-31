mod specs;
#[cfg(test)]
mod tests;

use rustyline::{
    Cmd, ConditionalEventHandler, Editor, Event, EventContext, EventHandler, KeyCode, KeyEvent,
    Modifiers, Movement, RepeatCount, history::DefaultHistory,
};

use self::specs::{BindingAction, binding_specs};
use super::{
    UshHelper,
    selection::{SelectionDelete, SelectionHandle},
};

pub fn configure_editor(
    editor: &mut Editor<UshHelper, DefaultHistory>,
    selection: SelectionHandle,
) {
    for event in selection_delete_events() {
        editor.bind_sequence(
            event,
            EventHandler::Conditional(Box::new(SelectionDeleteHandler(selection.clone()))),
        );
    }
    for spec in binding_specs() {
        editor.bind_sequence(
            spec.event,
            EventHandler::Conditional(Box::new(BindingHandler::new(
                spec.action,
                selection.clone(),
            ))),
        );
    }
    editor.bind_sequence(
        Event::Any,
        EventHandler::Conditional(Box::new(ClearSelectionHandler(selection))),
    );
}

struct BindingHandler {
    action: BindingAction,
    selection: SelectionHandle,
}

impl BindingHandler {
    fn new(action: BindingAction, selection: SelectionHandle) -> Self {
        Self { action, selection }
    }
}

impl ConditionalEventHandler for BindingHandler {
    fn handle(&self, _: &Event, _: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        match &self.action {
            BindingAction::Command(cmd) => {
                self.selection.clear();
                Some(cmd.clone())
            }
            BindingAction::Select(movement, cmd) => {
                self.selection.extend(ctx.line(), ctx.pos(), *movement);
                Some(cmd.clone())
            }
        }
    }
}

struct ClearSelectionHandler(SelectionHandle);

impl ConditionalEventHandler for ClearSelectionHandler {
    fn handle(&self, evt: &Event, _: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        if let Some(delete) = self.0.delete_action(ctx.line()) {
            if let Some(cmd) = selection_edit_command(evt, delete) {
                self.0.clear();
                return Some(cmd);
            }
        }
        if self.0.has_selection() && !should_keep_selection(evt) {
            self.0.clear();
        }
        None
    }
}

struct SelectionDeleteHandler(SelectionHandle);

impl ConditionalEventHandler for SelectionDeleteHandler {
    fn handle(&self, evt: &Event, _: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        let range = self.0.range()?;
        let cmd = selection_delete_command(evt, ctx.line(), ctx.pos(), range)?;
        self.0.clear();
        Some(cmd)
    }
}

fn selection_edit_command(evt: &Event, delete: SelectionDelete) -> Option<Cmd> {
    match evt.get(0) {
        Some(KeyEvent(KeyCode::Char(ch), mods))
            if !mods.contains(Modifiers::CTRL) && !mods.contains(Modifiers::ALT) =>
        {
            Some(Cmd::Replace(delete_movement(delete), Some(ch.to_string())))
        }
        Some(KeyEvent(KeyCode::Backspace | KeyCode::Delete, _)) => {
            Some(Cmd::Kill(delete_movement(delete)))
        }
        Some(KeyEvent(KeyCode::Char('H' | 'h'), mods))
            if mods.contains(Modifiers::CTRL) && !mods.contains(Modifiers::ALT) =>
        {
            Some(Cmd::Kill(delete_movement(delete)))
        }
        Some(KeyEvent(KeyCode::Char('D' | 'd'), mods))
            if mods.contains(Modifiers::CTRL) && !mods.contains(Modifiers::ALT) =>
        {
            Some(Cmd::Kill(delete_movement(delete)))
        }
        _ => None,
    }
}

fn selection_delete_command(
    evt: &Event,
    line: &str,
    pos: usize,
    range: (usize, usize),
) -> Option<Cmd> {
    if !is_delete_event(evt) {
        return None;
    }
    let (start, end) = range;
    let prefix = line.get(..start)?;
    let suffix = line.get(end..)?;
    Some(match pos {
        value if value == end => Cmd::Replace(Movement::BeginningOfLine, Some(prefix.to_string())),
        value if value == start => Cmd::Replace(Movement::EndOfLine, Some(suffix.to_string())),
        _ => Cmd::Replace(Movement::WholeLine, Some(format!("{prefix}{suffix}"))),
    })
}

fn selection_delete_events() -> [Event; 6] {
    [
        Event::from(KeyEvent(KeyCode::Backspace, Modifiers::NONE)),
        Event::from(KeyEvent(KeyCode::Delete, Modifiers::NONE)),
        Event::from(KeyEvent(KeyCode::Char('H'), Modifiers::CTRL)),
        Event::from(KeyEvent(KeyCode::Char('h'), Modifiers::CTRL)),
        Event::from(KeyEvent(KeyCode::Char('D'), Modifiers::CTRL)),
        Event::from(KeyEvent(KeyCode::Char('d'), Modifiers::CTRL)),
    ]
}

fn should_keep_selection(evt: &Event) -> bool {
    matches!(
        evt.get(0),
        Some(KeyEvent(KeyCode::UnknownEscSeq | KeyCode::Null, _))
    )
}

fn is_delete_event(evt: &Event) -> bool {
    match evt.get(0) {
        Some(KeyEvent(KeyCode::Backspace | KeyCode::Delete, _)) => true,
        Some(KeyEvent(KeyCode::Char('H' | 'h' | 'D' | 'd'), mods)) => {
            mods.contains(Modifiers::CTRL) && !mods.contains(Modifiers::ALT)
        }
        _ => false,
    }
}

fn delete_movement(delete: SelectionDelete) -> Movement {
    match delete {
        SelectionDelete::Backward(count) => Movement::BackwardChar(repeat_count(count)),
        SelectionDelete::Forward(count) => Movement::ForwardChar(repeat_count(count)),
    }
}

fn repeat_count(count: usize) -> RepeatCount {
    RepeatCount::try_from(count).unwrap_or(RepeatCount::MAX)
}
