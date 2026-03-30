mod bindings;
mod complete;
mod highlight;
mod selection;
mod syntax;
mod validate;

use std::{borrow::Cow, collections::BTreeSet, path::Path};

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

use self::selection::SelectionHandle;

pub struct UshHelper {
    commands: BTreeSet<String>,
    env_names: BTreeSet<String>,
    files: FilenameCompleter,
    hinter: HistoryHinter,
    selection: SelectionHandle,
}

impl UshHelper {
    pub fn new(commands: Vec<String>, env_names: Vec<String>) -> Self {
        Self {
            commands: commands.into_iter().collect(),
            env_names: env_names.into_iter().collect(),
            files: FilenameCompleter::new(),
            hinter: HistoryHinter::new(),
            selection: SelectionHandle::default(),
        }
    }

    pub fn refresh(&mut self, commands: Vec<String>, env_names: Vec<String>) {
        self.commands = commands.into_iter().collect();
        self.env_names = env_names.into_iter().collect();
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
    commands: Vec<String>,
    env_names: Vec<String>,
) -> Result<Editor<UshHelper, DefaultHistory>> {
    let config = Config::builder()
        .max_history_size(history_size)?
        .history_ignore_dups(true)?
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .completion_show_all_if_ambiguous(true)
        .completion_prompt_limit(200)
        .keyseq_timeout(Some(120))
        .edit_mode(EditMode::Emacs)
        .auto_add_history(true)
        .build();
    let helper = UshHelper::new(commands, env_names);
    let selection = helper.selection_handle();
    let mut editor = Editor::with_config(config)?;
    editor.set_helper(Some(helper));
    bindings::configure_editor(&mut editor, selection);
    let _ = editor.load_history(history_file);
    Ok(editor)
}

pub fn classify_readline_error(error: ReadlineError) -> anyhow::Error {
    match error {
        ReadlineError::Interrupted => anyhow::anyhow!("interrupted"),
        ReadlineError::Eof => anyhow::anyhow!("eof"),
        other => anyhow::anyhow!(other),
    }
}

#[cfg(test)]
mod tests {
    use rustyline::{Context, hint::Hinter, history::History};
    use tempfile::tempdir;

    use super::{UshHelper, create_editor};

    #[test]
    fn history_hint_prefers_previous_entries() {
        let dir = tempdir().expect("tempdir");
        let history_file = dir.path().join("history.txt");
        let mut editor = create_editor(
            &history_file,
            10,
            vec!["echo".to_string()],
            vec!["PATH".to_string()],
        )
        .expect("editor");
        editor.add_history_entry("echo hello").expect("history");
        let ctx = Context::new(editor.history());
        let helper = UshHelper::new(vec!["echo".to_string()], vec!["PATH".to_string()]);

        assert_eq!(helper.hint("echo h", 6, &ctx), Some("ello".to_string()));
    }

    #[test]
    fn editor_respects_history_limit() {
        let dir = tempdir().expect("tempdir");
        let history_file = dir.path().join("history.txt");
        let mut editor = create_editor(
            &history_file,
            2,
            vec!["echo".to_string()],
            vec!["PATH".to_string()],
        )
        .expect("editor");

        editor.add_history_entry("echo one").expect("history");
        editor.add_history_entry("echo two").expect("history");
        editor.add_history_entry("echo three").expect("history");

        assert_eq!(editor.history().len(), 2);
    }
}
