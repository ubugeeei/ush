use super::common::{CONFIRM_OPTIONS, INPUT_OPTIONS, RM_OPTIONS, SAMMARY_OPTIONS, SELECT_OPTIONS};
use super::types::{ArgKind, BuiltinSpec};

const SPEC_SAMMARY: BuiltinSpec = BuiltinSpec {
    summary: "summarize files and line counts recursively",
    usage: "[--include-lock] <glob|path>...",
    options: SAMMARY_OPTIONS,
    positionals: &[ArgKind::Path],
    trailing: Some(ArgKind::Path),
    after_double_dash: None,
};

const SPEC_GLOB: BuiltinSpec = BuiltinSpec {
    summary: "expand glob patterns or read them from stdin",
    usage: "<pattern>...",
    options: &[],
    positionals: &[ArgKind::Path],
    trailing: Some(ArgKind::Path),
    after_double_dash: None,
};

const SPEC_CONFIRM: BuiltinSpec = BuiltinSpec {
    summary: "ask for a yes or no answer",
    usage: "[--default yes|no] [--prompt <text>] [<prompt>...]",
    options: CONFIRM_OPTIONS,
    positionals: &[ArgKind::Text],
    trailing: Some(ArgKind::Text),
    after_double_dash: None,
};

const SPEC_INPUT: BuiltinSpec = BuiltinSpec {
    summary: "read one line of user input",
    usage: "[--default <value>] [--prompt <text>] [<prompt>...]",
    options: INPUT_OPTIONS,
    positionals: &[ArgKind::Text],
    trailing: Some(ArgKind::Text),
    after_double_dash: None,
};

const SPEC_SELECT: BuiltinSpec = BuiltinSpec {
    summary: "choose one option interactively or from defaults",
    usage: "[--prompt <text>] [--default <value>] [--] [<option>...]",
    options: SELECT_OPTIONS,
    positionals: &[ArgKind::Text],
    trailing: Some(ArgKind::Text),
    after_double_dash: Some(ArgKind::Text),
};

const SPEC_WHICH: BuiltinSpec = BuiltinSpec {
    summary: "show every resolution candidate and mark the current match",
    usage: "<command>...",
    options: &[],
    positionals: &[ArgKind::Command],
    trailing: Some(ArgKind::Command),
    after_double_dash: None,
};

const SPEC_TYPE: BuiltinSpec = BuiltinSpec {
    summary: "describe how a command resolves",
    usage: "<command>...",
    options: &[],
    positionals: &[ArgKind::Command],
    trailing: Some(ArgKind::Command),
    after_double_dash: None,
};

const SPEC_TEST: BuiltinSpec = BuiltinSpec {
    summary: "evaluate shell test expressions",
    usage: "<expr>  or  [ <expr> ]",
    options: &[],
    positionals: &[ArgKind::Text],
    trailing: Some(ArgKind::Text),
    after_double_dash: None,
};

const SPEC_HELP: BuiltinSpec = BuiltinSpec {
    summary: "show builtin help",
    usage: "[<builtin>...]",
    options: &[],
    positionals: &[ArgKind::Builtin],
    trailing: Some(ArgKind::Builtin),
    after_double_dash: None,
};

const SPEC_HISTORY: BuiltinSpec = BuiltinSpec {
    summary: "print recent shell history",
    usage: "[<n>]",
    options: &[],
    positionals: &[ArgKind::Number],
    trailing: None,
    after_double_dash: None,
};

const SPEC_EXIT: BuiltinSpec = BuiltinSpec {
    summary: "exit the shell with an optional status code",
    usage: "[<status>]",
    options: &[],
    positionals: &[ArgKind::Number],
    trailing: None,
    after_double_dash: None,
};

const SPEC_RM: BuiltinSpec = BuiltinSpec {
    summary: "guarded wrapper around rm for recursive deletes",
    usage: "[--yes] [<path>...]",
    options: RM_OPTIONS,
    positionals: &[ArgKind::Path],
    trailing: Some(ArgKind::Path),
    after_double_dash: None,
};

pub(super) fn resolve(name: &str) -> Option<&'static BuiltinSpec> {
    match name {
        "fsam" | "sammary" => Some(&SPEC_SAMMARY),
        "glob" => Some(&SPEC_GLOB),
        "confirm" => Some(&SPEC_CONFIRM),
        "input" => Some(&SPEC_INPUT),
        "select" => Some(&SPEC_SELECT),
        "which" => Some(&SPEC_WHICH),
        "type" => Some(&SPEC_TYPE),
        "[" | "test" => Some(&SPEC_TEST),
        "help" => Some(&SPEC_HELP),
        "history" => Some(&SPEC_HISTORY),
        "exit" => Some(&SPEC_EXIT),
        "rm" => Some(&SPEC_RM),
        _ => None,
    }
}
