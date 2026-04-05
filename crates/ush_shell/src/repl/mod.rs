mod bindings;
mod complete;
mod completion_state;
mod highlight;
mod selection;
mod syntax;
#[cfg(test)]
mod tests;
mod validate;

pub(crate) mod contextual;

use std::{
    borrow::Cow,
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::Result;
use rustyline::{
    CompletionType, Config, Context, EditMode, Editor, Helper,
    completion::{Completer, FilenameCompleter, Pair},
    error::ReadlineError,
    highlight::{CmdKind, Highlighter},
    hint::{Hinter, HistoryHinter},
    history::DefaultHistory,
    validate::{ValidationContext, ValidationResult, Validator},
};
use ush_config::ShellKeymap;

use self::selection::SelectionHandle;

pub(crate) use self::validate::validate_input;

pub struct UshHelper {
    commands: BTreeSet<String>,
    env_names: BTreeSet<String>,
    cwd: PathBuf,
    files: FilenameCompleter,
    hinter: HistoryHinter,
    completion: completion_state::CompletionState,
    selection: SelectionHandle,
}

impl UshHelper {
    pub fn new(commands: Vec<String>, env_names: Vec<String>, cwd: PathBuf) -> Self {
        Self {
            commands: commands.into_iter().collect(),
            env_names: env_names.into_iter().collect(),
            cwd,
            files: FilenameCompleter::new(),
            hinter: HistoryHinter::new(),
            completion: completion_state::CompletionState::default(),
            selection: SelectionHandle::default(),
        }
    }

    pub fn refresh(&mut self, commands: Vec<String>, env_names: Vec<String>, cwd: PathBuf) {
        self.commands = commands.into_iter().collect();
        self.env_names = env_names.into_iter().collect();
        self.cwd = cwd;
        self.completion.clear();
        self.selection.clear();
    }

    pub fn selection_handle(&self) -> SelectionHandle {
        self.selection.clone()
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection.range()
    }

    pub fn has_selection(&self) -> bool {
        self.selection.has_selection()
    }

    pub(crate) fn update_completion(&self, line: &str, pos: usize, start: usize, pairs: &[Pair]) {
        self.completion.update(line, pos, start, pairs);
    }

    pub(crate) fn cwd(&self) -> &Path {
        &self.cwd
    }

    fn command_pairs(&self, needle: &str) -> Vec<Pair> {
        let keywords = syntax::keywords().iter().copied().map(str::to_string);
        self.commands
            .iter()
            .cloned()
            .chain(keywords)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|item| item.starts_with(needle))
            .map(|item| Pair {
                display: item.clone(),
                replacement: item,
            })
            .collect()
    }

    fn env_pairs(&self, needle: &str, brace: bool, suffix: &str) -> Vec<Pair> {
        self.env_names
            .iter()
            .filter(|item| item.starts_with(needle))
            .map(|item| {
                let replacement = if brace {
                    format!("${{{item}}}{suffix}")
                } else {
                    format!("${item}{suffix}")
                };
                Pair {
                    display: item.clone(),
                    replacement,
                }
            })
            .collect()
    }

    fn env_name_pairs(&self, needle: &str) -> Vec<Pair> {
        self.env_names
            .iter()
            .filter(|item| item.starts_with(needle))
            .map(|item| Pair {
                display: item.clone(),
                replacement: format!("{item}="),
            })
            .collect()
    }
}

impl Helper for UshHelper {}

impl Completer for UshHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        complete::complete(self, line, pos, ctx)
    }
}

impl Hinter for UshHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        if let Some(hint) = self.completion.hint(line, pos) {
            return Some(hint);
        }
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for UshHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(highlight::highlight_line(self, line))
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Owned(highlight::highlight_prompt(prompt))
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(highlight::highlight_hint(hint))
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: CompletionType,
    ) -> Cow<'c, str> {
        Cow::Owned(highlight::highlight_candidate(self, candidate))
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        kind == CmdKind::ForcedRefresh || self.has_selection() || syntax::needs_refresh(line, pos)
    }
}

impl Validator for UshHelper {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
        Ok(validate::validate_input(ctx.input()))
    }
}

pub fn create_editor(
    history_file: &Path,
    history_size: usize,
    keymap: ShellKeymap,
    commands: Vec<String>,
    env_names: Vec<String>,
    cwd: PathBuf,
) -> Result<Editor<UshHelper, DefaultHistory>> {
    let config = Config::builder()
        .max_history_size(history_size)?
        .history_ignore_dups(true)?
        .history_ignore_space(true)
        .completion_type(CompletionType::Circular)
        .completion_show_all_if_ambiguous(false)
        .completion_prompt_limit(200)
        .keyseq_timeout(Some(300))
        .edit_mode(edit_mode(keymap))
        .auto_add_history(true)
        .build();
    let helper = UshHelper::new(commands, env_names, cwd);
    let selection = helper.selection_handle();
    let mut editor = Editor::with_config(config)?;
    editor.set_helper(Some(helper));
    bindings::configure_editor(&mut editor, selection);
    let _ = editor.load_history(history_file);
    Ok(editor)
}

fn edit_mode(keymap: ShellKeymap) -> EditMode {
    match keymap {
        ShellKeymap::Emacs => EditMode::Emacs,
        ShellKeymap::Vi => EditMode::Vi,
    }
}

pub fn classify_readline_error(error: ReadlineError) -> anyhow::Error {
    match error {
        ReadlineError::Interrupted => anyhow::anyhow!("interrupted"),
        ReadlineError::Eof => anyhow::anyhow!("eof"),
        other => anyhow::anyhow!(other),
    }
}
