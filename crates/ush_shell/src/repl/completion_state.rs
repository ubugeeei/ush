use std::cell::RefCell;

use rustyline::completion::Pair;

use super::syntax;

#[derive(Default)]
pub struct CompletionState(RefCell<Option<ActiveCompletion>>);

#[derive(Clone)]
struct ActiveCompletion {
    context: String,
    candidates: Vec<String>,
}

impl CompletionState {
    pub fn update(&self, line: &str, _pos: usize, start: usize, pairs: &[Pair]) {
        if pairs.len() <= 1 {
            self.clear();
            return;
        }
        self.0.replace(Some(ActiveCompletion {
            context: line[..start].to_string(),
            candidates: pairs.iter().map(|pair| pair.replacement.clone()).collect(),
        }));
    }

    pub fn clear(&self) {
        self.0.take();
    }

    pub fn hint(&self, line: &str, pos: usize) -> Option<String> {
        let active = self.0.borrow().clone()?;
        let start = syntax::word_start(line, pos);
        let word = line.get(start..pos)?;

        if start != active.context.len() || !line.starts_with(&active.context) {
            self.clear();
            return None;
        }

        let index = active
            .candidates
            .iter()
            .position(|candidate| candidate == word)?;
        Some(format!(
            "  [{}/{}] tab: next  shift-tab: prev  enter: accept",
            index + 1,
            active.candidates.len()
        ))
    }
}
