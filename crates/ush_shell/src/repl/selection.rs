use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct SelectionHandle(Arc<Mutex<SelectionState>>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionDelete {
    Backward(usize),
    Forward(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionMove {
    CharLeft,
    CharRight,
    WordLeft,
    WordRight,
    BigWordLeft,
    BigWordRight,
    LineStart,
    LineEnd,
}

#[derive(Default)]
struct SelectionState {
    anchor: Option<usize>,
    head: Option<usize>,
}

impl SelectionHandle {
    pub fn extend(&self, line: &str, pos: usize, movement: SelectionMove) -> usize {
        let next = movement.apply(line, pos);
        let mut state = self.0.lock().expect("selection lock");
        let anchor = state.anchor.get_or_insert(pos);
        if *anchor == next {
            *state = SelectionState::default();
            return next;
        }
        state.head = Some(next);
        next
    }

    pub fn clear(&self) {
        *self.0.lock().expect("selection lock") = SelectionState::default();
    }

    pub fn range(&self) -> Option<(usize, usize)> {
        let state = self.0.lock().expect("selection lock");
        normalized_range(&state)
    }

    pub fn has_selection(&self) -> bool {
        self.range().is_some()
    }

    pub fn delete_action(&self, line: &str) -> Option<SelectionDelete> {
        let state = self.0.lock().expect("selection lock");
        let (anchor, head) = (state.anchor?, state.head?);
        let (start, end, direction) = if anchor < head {
            (anchor, head, SelectionDelete::Backward(0))
        } else {
            (head, anchor, SelectionDelete::Forward(0))
        };
        let count = line.get(start..end)?.chars().count();
        match direction {
            SelectionDelete::Backward(_) if count > 0 => Some(SelectionDelete::Backward(count)),
            SelectionDelete::Forward(_) if count > 0 => Some(SelectionDelete::Forward(count)),
            _ => None,
        }
    }
}

impl SelectionMove {
    fn apply(self, line: &str, pos: usize) -> usize {
        match self {
            Self::CharLeft => previous_boundary(line, pos),
            Self::CharRight => next_boundary(line, pos),
            Self::WordLeft => move_word_left(line, pos, false),
            Self::WordRight => move_word_right(line, pos, false),
            Self::BigWordLeft => move_word_left(line, pos, true),
            Self::BigWordRight => move_word_right(line, pos, true),
            Self::LineStart => 0,
            Self::LineEnd => line.len(),
        }
    }
}

fn normalized_range(state: &SelectionState) -> Option<(usize, usize)> {
    let (anchor, head) = (state.anchor?, state.head?);
    if anchor == head {
        None
    } else {
        Some((anchor.min(head), anchor.max(head)))
    }
}

fn move_word_left(line: &str, pos: usize, big: bool) -> usize {
    let idx = skip_left(line, pos, char::is_whitespace);
    let Some(ch) = previous_char(line, idx) else {
        return 0;
    };
    if big {
        return skip_left(line, idx, |value| !value.is_whitespace());
    }
    let word = is_word_char(ch);
    skip_left(line, idx, |value| {
        if word {
            is_word_char(value)
        } else {
            !value.is_whitespace() && !is_word_char(value)
        }
    })
}

fn move_word_right(line: &str, pos: usize, big: bool) -> usize {
    let idx = skip_right(line, pos, char::is_whitespace);
    let Some(ch) = current_char(line, idx) else {
        return line.len();
    };
    if big {
        return skip_right(line, idx, |value| !value.is_whitespace());
    }
    let word = is_word_char(ch);
    skip_right(line, idx, |value| {
        if word {
            is_word_char(value)
        } else {
            !value.is_whitespace() && !is_word_char(value)
        }
    })
}

fn skip_left(line: &str, mut idx: usize, keep: impl Fn(char) -> bool) -> usize {
    while let Some(ch) = previous_char(line, idx) {
        if !keep(ch) {
            break;
        }
        idx = previous_boundary(line, idx);
    }
    idx
}

fn skip_right(line: &str, mut idx: usize, keep: impl Fn(char) -> bool) -> usize {
    while let Some(ch) = current_char(line, idx) {
        if !keep(ch) {
            break;
        }
        idx = next_boundary(line, idx);
    }
    idx
}

fn previous_boundary(line: &str, pos: usize) -> usize {
    line[..pos]
        .char_indices()
        .next_back()
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

fn next_boundary(line: &str, pos: usize) -> usize {
    current_char(line, pos)
        .map(|ch| pos + ch.len_utf8())
        .unwrap_or_else(|| line.len())
}

fn previous_char(line: &str, pos: usize) -> Option<char> {
    line.get(..pos)?.chars().next_back()
}

fn current_char(line: &str, pos: usize) -> Option<char> {
    line.get(pos..)?.chars().next()
}

fn is_word_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::{SelectionDelete, SelectionHandle, SelectionMove};

    #[test]
    fn tracks_ranges_and_clears_back_to_anchor() {
        let selection = SelectionHandle::default();

        assert_eq!(selection.extend("echo", 4, SelectionMove::CharLeft), 3);
        assert_eq!(selection.range(), Some((3, 4)));
        assert_eq!(selection.extend("echo", 3, SelectionMove::CharRight), 4);
        assert_eq!(selection.range(), None);
    }

    #[test]
    fn computes_delete_direction_from_cursor_side() {
        let left = SelectionHandle::default();
        left.extend("echo hello", 5, SelectionMove::WordRight);
        assert_eq!(
            left.delete_action("echo hello"),
            Some(SelectionDelete::Backward(5))
        );

        let right = SelectionHandle::default();
        right.extend("echo hello", 10, SelectionMove::WordLeft);
        assert_eq!(
            right.delete_action("echo hello"),
            Some(SelectionDelete::Forward(5))
        );
    }

    #[test]
    fn uses_word_and_big_word_boundaries() {
        let selection = SelectionHandle::default();

        assert_eq!(
            selection.extend("say hello-world", 4, SelectionMove::WordRight),
            9
        );
        selection.clear();
        assert_eq!(
            selection.extend("say hello-world", 4, SelectionMove::BigWordRight),
            15
        );
    }
}
