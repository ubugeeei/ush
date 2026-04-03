use std::sync::{Arc, Mutex};

use rustyline::completion::Pair;

use super::syntax;

#[derive(Clone, Default)]
pub struct CompletionState(Arc<Mutex<Option<ActiveCompletion>>>);

#[derive(Clone)]
struct ActiveCompletion {
    context: String,
    candidates: Vec<CompletionCandidate>,
}

#[derive(Clone)]
struct CompletionCandidate {
    replacement: String,
    summary: Option<String>,
}

impl CompletionState {
    pub fn update(&self, line: &str, _pos: usize, start: usize, pairs: &[Pair]) {
        if pairs.len() <= 1 {
            self.clear();
            return;
        }
        *self.0.lock().expect("completion state lock poisoned") = Some(ActiveCompletion {
            context: line[..start].to_string(),
            candidates: pairs
                .iter()
                .map(|pair| CompletionCandidate {
                    replacement: pair.replacement.clone(),
                    summary: completion_summary(pair),
                })
                .collect(),
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

    pub fn hint(&self, line: &str, pos: usize) -> Option<String> {
        let (active, index) = self.current_candidate(line, pos)?;

        let mut hint = format!("  [{}/{}]", index + 1, active.candidates.len());
        if let Some(summary) = &active.candidates[index].summary {
            hint.push(' ');
            hint.push_str(summary);
        }
        hint.push_str(" tab: next  shift-tab: prev  enter: accept");
        Some(hint)
    }

    fn matches_current_word(&self, line: &str, pos: usize) -> bool {
        self.current_candidate(line, pos).is_some()
    }

    fn current_candidate(&self, line: &str, pos: usize) -> Option<(ActiveCompletion, usize)> {
        let active = self
            .0
            .lock()
            .expect("completion state lock poisoned")
            .clone()?;
        let start = syntax::word_start(line, pos);
        let word = line.get(start..pos)?;

        if start != active.context.len() || !line.starts_with(&active.context) {
            self.clear();
            return None;
        }

        let index = active
            .candidates
            .iter()
            .position(|candidate| candidate.replacement == word)?;

        Some((active, index))
    }
}

fn completion_summary(pair: &Pair) -> Option<String> {
    pair.display
        .strip_prefix(&pair.replacement)
        .map(str::trim)
        .filter(|summary| !summary.is_empty())
        .map(ToOwned::to_owned)
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
                    display: "script.ush  path".to_string(),
                    replacement: "script.ush".to_string(),
                },
                Pair {
                    display: "scratch/  directory".to_string(),
                    replacement: "scratch/".to_string(),
                },
            ],
        );

        assert!(state.accept("source scratch/", 15));
        assert_eq!(state.hint("source scratch/", 15), None);
    }
}
