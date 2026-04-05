use std::sync::{Arc, Mutex};

use rustyline::completion::Pair;

use super::syntax;

#[derive(Clone, Default)]
pub struct CompletionState(Arc<Mutex<Option<ActiveCompletion>>>);

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
        *self.0.lock().expect("completion state lock poisoned") = Some(ActiveCompletion {
            context: line[..start].to_string(),
            candidates: pairs.iter().map(|pair| pair.replacement.clone()).collect(),
        });
    }

    pub fn clear(&self) {
        *self.0.lock().expect("completion state lock poisoned") = None;
    }

    pub fn accept(&self, line: &str, pos: usize) -> bool {
        if self.matches_current_word(line, pos) {
            self.clear();
            return true;
        }
        false
    }

    pub fn hint(&self, _line: &str, _pos: usize) -> Option<String> {
        None
    }

    fn matches_current_word(&self, line: &str, pos: usize) -> bool {
        let active = self
            .0
            .lock()
            .expect("completion state lock poisoned")
            .clone();
        let Some(active) = active else {
            return false;
        };
        let start = syntax::word_start(line, pos);
        let Some(word) = line.get(start..pos) else {
            return false;
        };
        start == active.context.len()
            && line.starts_with(&active.context)
            && active.candidates.iter().any(|candidate| candidate == word)
    }
}

#[cfg(test)]
mod tests {
    use rustyline::completion::Pair;

    use super::CompletionState;

    #[test]
    fn accept_clears_active_completion_without_changing_line() {
        let state = CompletionState::default();
        state.update(
            "source scr",
            10,
            7,
            &[
                Pair {
                    display: "script.ush".to_string(),
                    replacement: "script.ush".to_string(),
                },
                Pair {
                    display: "scratch/".to_string(),
                    replacement: "scratch/".to_string(),
                },
            ],
        );

        assert!(state.accept("source scratch/", 15));
        assert_eq!(state.hint("source scratch/", 15), None);
    }
}
