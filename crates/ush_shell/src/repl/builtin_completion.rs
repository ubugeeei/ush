use std::{collections::BTreeSet, path::Path};

use rustyline::completion::Pair;

use crate::commands::{BUILTIN_COMMANDS, is_builtin};

use super::{UshHelper, syntax};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArgKind {
    Alias,
    AliasAssignment,
    Builtin,
    Choice(&'static [&'static str]),
    Command,
    EnvAssignment,
    EnvName,
    Job,
    Number,
    Path,
    Port,
    Signal,
    Text,
}

impl ArgKind {
    fn placeholder(self) -> &'static str {
        match self {
            Self::Alias => "<alias>",
            Self::AliasAssignment => "<name=value>",
            Self::Builtin => "<builtin>",
            Self::Choice(_) => "<value>",
            Self::Command => "<command>",
            Self::EnvAssignment => "<NAME=value>",
            Self::EnvName => "<NAME>",
            Self::Job => "%job",
            Self::Number => "<n>",
            Self::Path => "<path>",
            Self::Port => "<port>",
            Self::Signal => "<signal>",
            Self::Text => "<value>",
        }
    }

    fn path_like(self) -> bool {
        matches!(self, Self::Path)
    }
}

#[derive(Clone, Copy, Debug)]
struct BuiltinOptionSpec {
    names: &'static [&'static str],
    summary: &'static str,
    value: Option<ArgKind>,
}

#[derive(Clone, Copy, Debug)]
struct BuiltinSpec {
    summary: &'static str,
    usage: &'static str,
    options: &'static [BuiltinOptionSpec],
    positionals: &'static [ArgKind],
    trailing: Option<ArgKind>,
    after_double_dash: Option<ArgKind>,
}

impl BuiltinSpec {
    fn positional_kind(self, index: usize) -> Option<ArgKind> {
        self.positionals
            .get(index)
            .copied()
            .or(if index >= self.positionals.len() {
                self.trailing
            } else {
                None
            })
    }
}

#[derive(Debug, Default)]
struct ParsedOptions {
    positionals: Vec<String>,
    expected_value: Option<ArgKind>,
    after_double_dash: bool,
}

#[derive(Debug)]
struct BuiltinContext {
    start: usize,
    word: String,
    command: String,
    args: Vec<String>,
}

const YES_NO_CHOICES: &[&str] = &["yes", "no"];
const SIGNAL_CHOICES: &[&str] = &[
    "TERM", "KILL", "INT", "HUP", "QUIT", "USR1", "USR2", "STOP", "CONT",
];

const ECHO_OPTIONS: &[BuiltinOptionSpec] = &[BuiltinOptionSpec {
    names: &["-n"],
    summary: "omit the trailing newline",
    value: None,
}];

const UNSET_OPTIONS: &[BuiltinOptionSpec] = &[BuiltinOptionSpec {
    names: &["-v"],
    summary: "remove shell variables",
    value: None,
}];

const STOP_OPTIONS: &[BuiltinOptionSpec] = &[BuiltinOptionSpec {
    names: &["--signal"],
    summary: "choose the signal to send",
    value: Some(ArgKind::Signal),
}];

const SAMMARY_OPTIONS: &[BuiltinOptionSpec] = &[BuiltinOptionSpec {
    names: &["--include-lock"],
    summary: "include lockfiles in the report",
    value: None,
}];

const CONFIRM_OPTIONS: &[BuiltinOptionSpec] = &[
    BuiltinOptionSpec {
        names: &["--default"],
        summary: "fall back to yes or no when interaction is disabled",
        value: Some(ArgKind::Choice(YES_NO_CHOICES)),
    },
    BuiltinOptionSpec {
        names: &["--prompt"],
        summary: "set the prompt text explicitly",
        value: Some(ArgKind::Text),
    },
];

const INPUT_OPTIONS: &[BuiltinOptionSpec] = &[
    BuiltinOptionSpec {
        names: &["--default"],
        summary: "use this value when interaction is disabled",
        value: Some(ArgKind::Text),
    },
    BuiltinOptionSpec {
        names: &["--prompt"],
        summary: "set the prompt text explicitly",
        value: Some(ArgKind::Text),
    },
];

const SELECT_OPTIONS: &[BuiltinOptionSpec] = &[
    BuiltinOptionSpec {
        names: &["--default"],
        summary: "preselect this option when possible",
        value: Some(ArgKind::Text),
    },
    BuiltinOptionSpec {
        names: &["--prompt"],
        summary: "set the prompt text explicitly",
        value: Some(ArgKind::Text),
    },
];

const RM_OPTIONS: &[BuiltinOptionSpec] = &[BuiltinOptionSpec {
    names: &["--yes"],
    summary: "skip the recursive delete confirmation",
    value: None,
}];

const COMMAND_OPTIONS: &[BuiltinOptionSpec] = &[
    BuiltinOptionSpec {
        names: &["-v"],
        summary: "print the command path or builtin name",
        value: None,
    },
    BuiltinOptionSpec {
        names: &["-V"],
        summary: "describe how the command resolves",
        value: None,
    },
];

const SPEC_NOOP: BuiltinSpec = BuiltinSpec {
    summary: "return a successful no-op status",
    usage: "",
    options: &[],
    positionals: &[],
    trailing: None,
    after_double_dash: None,
};

const SPEC_SOURCE: BuiltinSpec = BuiltinSpec {
    summary: "read and execute commands from a file in the current shell",
    usage: "<file>",
    options: &[],
    positionals: &[ArgKind::Path],
    trailing: None,
    after_double_dash: None,
};

const SPEC_CD: BuiltinSpec = BuiltinSpec {
    summary: "change the current working directory",
    usage: "[<dir>]",
    options: &[],
    positionals: &[ArgKind::Path],
    trailing: None,
    after_double_dash: None,
};

const SPEC_PWD: BuiltinSpec = BuiltinSpec {
    summary: "print the current working directory",
    usage: "",
    options: &[],
    positionals: &[],
    trailing: None,
    after_double_dash: None,
};

const SPEC_ECHO: BuiltinSpec = BuiltinSpec {
    summary: "print arguments to stdout",
    usage: "[-n] [<text>...]",
    options: ECHO_OPTIONS,
    positionals: &[],
    trailing: Some(ArgKind::Text),
    after_double_dash: None,
};

const SPEC_TRUE: BuiltinSpec = BuiltinSpec {
    summary: "exit successfully",
    usage: "",
    options: &[],
    positionals: &[],
    trailing: None,
    after_double_dash: None,
};

const SPEC_FALSE: BuiltinSpec = BuiltinSpec {
    summary: "exit with a failure status",
    usage: "",
    options: &[],
    positionals: &[],
    trailing: None,
    after_double_dash: None,
};

const SPEC_ALIAS: BuiltinSpec = BuiltinSpec {
    summary: "define shell aliases",
    usage: "[<name=value>...]",
    options: &[],
    positionals: &[ArgKind::AliasAssignment],
    trailing: Some(ArgKind::AliasAssignment),
    after_double_dash: None,
};

const SPEC_UNALIAS: BuiltinSpec = BuiltinSpec {
    summary: "remove shell aliases",
    usage: "<alias>...",
    options: &[],
    positionals: &[ArgKind::Alias],
    trailing: Some(ArgKind::Alias),
    after_double_dash: None,
};

const SPEC_EXPORT: BuiltinSpec = BuiltinSpec {
    summary: "set environment variables in the current shell",
    usage: "<NAME=value>...",
    options: &[],
    positionals: &[ArgKind::EnvAssignment],
    trailing: Some(ArgKind::EnvAssignment),
    after_double_dash: None,
};

const SPEC_UNSET: BuiltinSpec = BuiltinSpec {
    summary: "remove shell variables",
    usage: "[-v] <NAME>...",
    options: UNSET_OPTIONS,
    positionals: &[ArgKind::EnvName],
    trailing: Some(ArgKind::EnvName),
    after_double_dash: None,
};

const SPEC_PORT: BuiltinSpec = BuiltinSpec {
    summary: "resolve listening pids from one or more ports",
    usage: "<port>...",
    options: &[],
    positionals: &[ArgKind::Port],
    trailing: Some(ArgKind::Port),
    after_double_dash: None,
};

const SPEC_STOP: BuiltinSpec = BuiltinSpec {
    summary: "send a signal to pids from args or stdin",
    usage: "[--signal <signal>] [<pid>...]",
    options: STOP_OPTIONS,
    positionals: &[ArgKind::Number],
    trailing: Some(ArgKind::Number),
    after_double_dash: None,
};

const SPEC_JOBS: BuiltinSpec = BuiltinSpec {
    summary: "list tracked background jobs",
    usage: "",
    options: &[],
    positionals: &[],
    trailing: None,
    after_double_dash: None,
};

const SPEC_WAIT: BuiltinSpec = BuiltinSpec {
    summary: "wait for background jobs to finish",
    usage: "[%job...]",
    options: &[],
    positionals: &[ArgKind::Job],
    trailing: Some(ArgKind::Job),
    after_double_dash: None,
};

const SPEC_DISOWN: BuiltinSpec = BuiltinSpec {
    summary: "remove jobs from the shell's job table",
    usage: "[%job...]",
    options: &[],
    positionals: &[ArgKind::Job],
    trailing: Some(ArgKind::Job),
    after_double_dash: None,
};

const SPEC_FG: BuiltinSpec = BuiltinSpec {
    summary: "bring a background job to the foreground",
    usage: "[%job]",
    options: &[],
    positionals: &[ArgKind::Job],
    trailing: None,
    after_double_dash: None,
};

const SPEC_BG: BuiltinSpec = BuiltinSpec {
    summary: "continue a tracked job in the background",
    usage: "[%job]",
    options: &[],
    positionals: &[ArgKind::Job],
    trailing: None,
    after_double_dash: None,
};

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

const SPEC_ENV: BuiltinSpec = BuiltinSpec {
    summary: "run a command with temporary environment overrides",
    usage: "[<NAME=value>...] [<command> ...]",
    options: &[],
    positionals: &[],
    trailing: None,
    after_double_dash: None,
};

const SPEC_COMMAND: BuiltinSpec = BuiltinSpec {
    summary: "run a command ignoring aliases or inspect lookup results",
    usage: "[-v|-V] <command>...",
    options: COMMAND_OPTIONS,
    positionals: &[ArgKind::Command],
    trailing: Some(ArgKind::Command),
    after_double_dash: None,
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

pub(super) fn command_summary(name: &str) -> Option<&'static str> {
    resolve_builtin_spec(name).map(|spec| spec.summary)
}

pub fn complete(
    helper: &UshHelper,
    line: &str,
    pos: usize,
) -> rustyline::Result<Option<(usize, Vec<Pair>)>> {
    let Some(context) = parse_context(line, pos) else {
        return Ok(None);
    };

    match context.command.as_str() {
        "env" => complete_env(helper, &context).map(Some),
        "command" => complete_command_builtin(helper, &context).map(Some),
        _ => complete_builtin_spec(helper, &context).map(Some),
    }
}

pub fn hint(helper: &UshHelper, line: &str, pos: usize) -> Option<String> {
    let context = parse_context(line, pos)?;

    match context.command.as_str() {
        "env" => hint_env(helper, &context),
        "command" => hint_command_builtin(helper, &context),
        _ => hint_builtin_spec(helper, &context),
    }
}

fn complete_builtin_spec(
    helper: &UshHelper,
    context: &BuiltinContext,
) -> rustyline::Result<(usize, Vec<Pair>)> {
    let spec = resolve_builtin_spec(&context.command).expect("builtin spec");
    let parsed = parse_options(&context.args, spec.options);

    if let Some(kind) = parsed.expected_value {
        return complete_arg(helper, context.start, &context.word, kind);
    }
    if !parsed.after_double_dash && context.word.starts_with('-') {
        return Ok((context.start, option_pairs(&context.word, spec.options)));
    }

    let expected = if parsed.after_double_dash {
        spec.after_double_dash
    } else {
        spec.positional_kind(parsed.positionals.len())
    };
    let mut pairs = if let Some(kind) = expected {
        complete_arg(helper, context.start, &context.word, kind)?.1
    } else {
        Vec::new()
    };

    if context.word.is_empty() && !parsed.after_double_dash {
        pairs.extend(option_pairs("", spec.options));
        pairs = dedupe_pairs(pairs);
    }

    Ok((context.start, pairs))
}

fn hint_builtin_spec(helper: &UshHelper, context: &BuiltinContext) -> Option<String> {
    let spec = resolve_builtin_spec(&context.command)?;
    let parsed = parse_options(&context.args, spec.options);

    if let Some(kind) = parsed.expected_value {
        return Some(command_hint(spec, Some(kind)));
    }
    if !parsed.after_double_dash && context.word.starts_with('-') {
        let options = matching_options(&context.word, spec.options);
        return option_hint(options).or_else(|| Some(command_hint(spec, None)));
    }

    let expected = if parsed.after_double_dash {
        spec.after_double_dash
    } else {
        spec.positional_kind(parsed.positionals.len())
    };

    if matches!(expected, Some(ArgKind::Job)) && helper.jobs.is_empty() {
        let mut hint = command_hint(spec, None);
        hint.push_str("  (no tracked jobs)");
        return Some(hint);
    }

    Some(command_hint(spec, expected))
}

fn complete_env(
    helper: &UshHelper,
    context: &BuiltinContext,
) -> rustyline::Result<(usize, Vec<Pair>)> {
    let mut index = 0usize;
    while let Some(token) = context.args.get(index) {
        if syntax::is_assignment(token) {
            index += 1;
            continue;
        }
        break;
    }

    if context.word.contains('=') {
        return Ok((context.start, Vec::new()));
    }
    if syntax::is_assignment(&context.word)
        || (!context.word.is_empty() && index == context.args.len())
    {
        return Ok((context.start, env_assignment_pairs(helper, &context.word)));
    }
    if index == context.args.len() {
        return Ok((context.start, command_name_pairs(helper, &context.word)));
    }
    Ok((context.start, Vec::new()))
}

fn hint_env(_helper: &UshHelper, context: &BuiltinContext) -> Option<String> {
    let mut index = 0usize;
    while let Some(token) = context.args.get(index) {
        if syntax::is_assignment(token) {
            index += 1;
            continue;
        }
        break;
    }
    if index == context.args.len() {
        return Some(
            "  run a command with temporary environment overrides  next: <NAME=value> or <command>  usage: [<NAME=value>...] [<command> ...]".to_string(),
        );
    }
    Some("  run a command with temporary environment overrides  usage: [<NAME=value>...] [<command> ...]".to_string())
}

fn complete_command_builtin(
    helper: &UshHelper,
    context: &BuiltinContext,
) -> rustyline::Result<(usize, Vec<Pair>)> {
    let option_mode = context
        .args
        .first()
        .is_some_and(|arg| matches!(arg.as_str(), "-v" | "-V"));
    if context.word.starts_with('-') || (context.args.is_empty() && context.word.is_empty()) {
        let mut pairs = option_pairs(&context.word, COMMAND_OPTIONS);
        if context.word.is_empty() && !option_mode {
            pairs.extend(command_name_pairs(helper, ""));
            pairs = dedupe_pairs(pairs);
        }
        return Ok((context.start, pairs));
    }

    if option_mode {
        return Ok((context.start, command_name_pairs(helper, &context.word)));
    }

    if context.args.is_empty() {
        return Ok((context.start, command_name_pairs(helper, &context.word)));
    }

    Ok((context.start, Vec::new()))
}

fn hint_command_builtin(_helper: &UshHelper, context: &BuiltinContext) -> Option<String> {
    if context.word.starts_with('-') {
        let options = matching_options(&context.word, COMMAND_OPTIONS);
        return option_hint(options).or_else(|| {
            Some(
                "  run a command ignoring aliases or inspect lookup results  usage: [-v|-V] <command>..."
                    .to_string(),
            )
        });
    }
    if context
        .args
        .first()
        .is_some_and(|arg| matches!(arg.as_str(), "-v" | "-V"))
    {
        return Some(
            "  run a command ignoring aliases or inspect lookup results  next: <command>  usage: [-v|-V] <command>..."
                .to_string(),
        );
    }
    if context.args.is_empty() {
        return Some(
            "  run a command ignoring aliases or inspect lookup results  next: <command>  usage: [-v|-V] <command>..."
                .to_string(),
        );
    }
    Some(
        "  run a command ignoring aliases or inspect lookup results  usage: [-v|-V] <command>..."
            .to_string(),
    )
}

fn parse_context(line: &str, pos: usize) -> Option<BuiltinContext> {
    let prefix = &line[..pos];
    let start = syntax::word_start(line, pos);
    let word = prefix[start..].to_string();
    let tokens = syntax::tokenize(&prefix[..start]);
    let segment_start = tokens
        .iter()
        .rposition(|token| matches!(token.as_str(), "|" | "||" | "&&" | ";" | "&"))
        .map_or(0, |index| index + 1);
    let segment = &tokens[segment_start..];
    let command_index = segment.iter().position(|token| {
        !(syntax::is_assignment(token) || matches!(token.as_str(), "!" | "command"))
    })?;
    let command = segment.get(command_index)?.clone();
    if !is_builtin(&command) {
        return None;
    }

    Some(BuiltinContext {
        start,
        word,
        command,
        args: segment[command_index + 1..].to_vec(),
    })
}

fn parse_options(tokens: &[String], options: &[BuiltinOptionSpec]) -> ParsedOptions {
    let mut parsed = ParsedOptions::default();
    let mut index = 0usize;

    while let Some(token) = tokens.get(index) {
        if parsed.after_double_dash {
            parsed.positionals.push(token.clone());
            index += 1;
            continue;
        }
        if token == "--" {
            parsed.after_double_dash = true;
            index += 1;
            continue;
        }
        if let Some(option) = find_option(token, options) {
            if option_consumes_inline_value(token, option) {
                index += 1;
                continue;
            }
            if let Some(kind) = option.value {
                if let Some(value) = tokens.get(index + 1)
                    && value != "--"
                {
                    index += 2;
                    continue;
                }
                parsed.expected_value = Some(kind);
                break;
            }
            index += 1;
            continue;
        }
        parsed.positionals.push(token.clone());
        index += 1;
    }

    parsed
}

fn complete_arg(
    helper: &UshHelper,
    start: usize,
    word: &str,
    kind: ArgKind,
) -> rustyline::Result<(usize, Vec<Pair>)> {
    if kind.path_like() {
        return Ok((start, complete_path_pairs(helper.cwd.as_path(), word)));
    }

    Ok((
        start,
        match kind {
            ArgKind::Alias => alias_pairs(helper, word),
            ArgKind::AliasAssignment => alias_assignment_pairs(word),
            ArgKind::Builtin => builtin_pairs(word, true),
            ArgKind::Choice(values) => choice_pairs(word, values),
            ArgKind::Command => command_name_pairs(helper, word),
            ArgKind::EnvAssignment => env_assignment_pairs(helper, word),
            ArgKind::EnvName => env_name_pairs(helper, word),
            ArgKind::Job => job_pairs(helper, word),
            ArgKind::Number | ArgKind::Port | ArgKind::Signal | ArgKind::Text => match kind {
                ArgKind::Signal => choice_pairs(word, SIGNAL_CHOICES),
                _ => Vec::new(),
            },
            ArgKind::Path => Vec::new(),
        },
    ))
}

fn command_name_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    if needle.is_empty() {
        return Vec::new();
    }
    helper
        .commands
        .iter()
        .filter(|item| item.starts_with(needle))
        .map(|item| {
            let summary = command_summary(item)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| "command".to_string());
            described_pair(item, &summary)
        })
        .collect()
}

fn builtin_pairs(needle: &str, include_empty: bool) -> Vec<Pair> {
    if needle.is_empty() && !include_empty {
        return Vec::new();
    }
    let mut seen = BTreeSet::new();
    BUILTIN_COMMANDS
        .iter()
        .copied()
        .filter(|name| name.starts_with(needle))
        .filter(|name| seen.insert(*name))
        .filter_map(|name| command_summary(name).map(|summary| described_pair(name, summary)))
        .collect()
}

fn alias_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    if needle.is_empty() {
        return Vec::new();
    }
    helper
        .alias_names
        .iter()
        .filter(|name| name.starts_with(needle))
        .map(|name| described_pair(name, "alias"))
        .collect()
}

fn alias_assignment_pairs(needle: &str) -> Vec<Pair> {
    if needle.is_empty() || needle.contains('=') {
        return Vec::new();
    }
    vec![described_pair(&format!("{needle}="), "define alias")]
}

fn env_name_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    if needle.is_empty() {
        return Vec::new();
    }
    helper
        .env_names
        .iter()
        .filter(|name| name.starts_with(needle))
        .map(|name| described_pair(name, "environment variable"))
        .collect()
}

fn env_assignment_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    if needle.is_empty() || needle.contains('=') {
        return Vec::new();
    }
    helper
        .env_names
        .iter()
        .filter(|name| name.starts_with(needle))
        .map(|name| described_pair(&format!("{name}="), "set environment variable"))
        .collect()
}

fn job_pairs(helper: &UshHelper, needle: &str) -> Vec<Pair> {
    helper
        .jobs
        .iter()
        .filter(|job| needle.is_empty() || job.spec.starts_with(needle))
        .map(|job| described_pair(&job.spec, &job.summary))
        .collect()
}

fn choice_pairs(needle: &str, choices: &'static [&'static str]) -> Vec<Pair> {
    choices
        .iter()
        .copied()
        .filter(|choice| {
            choice.starts_with(&needle.to_ascii_lowercase()) || choice.starts_with(needle)
        })
        .map(|choice| described_pair(choice, "value"))
        .collect()
}

fn option_pairs(prefix: &str, options: &[BuiltinOptionSpec]) -> Vec<Pair> {
    dedupe_pairs(
        options
            .iter()
            .flat_map(|option| {
                option
                    .names
                    .iter()
                    .filter(move |name| name.starts_with(prefix))
                    .map(move |name| described_pair(name, &option_summary(option)))
            })
            .collect(),
    )
}

fn matching_options<'a>(
    prefix: &str,
    options: &'a [BuiltinOptionSpec],
) -> Vec<&'a BuiltinOptionSpec> {
    options
        .iter()
        .filter(|option| option.names.iter().any(|name| name.starts_with(prefix)))
        .collect()
}

fn option_hint(options: Vec<&BuiltinOptionSpec>) -> Option<String> {
    if options.len() != 1 {
        return None;
    }
    let option = options[0];
    let name = option.names.iter().max_by_key(|name| name.len()).copied()?;
    let value = option
        .value
        .map(|kind| format!(" {}", kind.placeholder()))
        .unwrap_or_default();
    Some(format!("  {name}{value}  {}", option.summary))
}

fn command_hint(spec: &BuiltinSpec, expected: Option<ArgKind>) -> String {
    let mut hint = format!("  {}", spec.summary);
    if let Some(kind) = expected {
        hint.push_str("  next: ");
        hint.push_str(kind.placeholder());
    }
    if !spec.usage.is_empty() {
        hint.push_str("  usage: ");
        hint.push_str(spec.usage);
    }
    hint
}

fn resolve_builtin_spec(name: &str) -> Option<&'static BuiltinSpec> {
    match name {
        ":" => Some(&SPEC_NOOP),
        "." | "source" => Some(&SPEC_SOURCE),
        "cd" => Some(&SPEC_CD),
        "pwd" => Some(&SPEC_PWD),
        "echo" => Some(&SPEC_ECHO),
        "true" => Some(&SPEC_TRUE),
        "false" => Some(&SPEC_FALSE),
        "alias" => Some(&SPEC_ALIAS),
        "unalias" => Some(&SPEC_UNALIAS),
        "export" => Some(&SPEC_EXPORT),
        "unset" => Some(&SPEC_UNSET),
        "port" => Some(&SPEC_PORT),
        "stop" => Some(&SPEC_STOP),
        "jobs" => Some(&SPEC_JOBS),
        "wait" => Some(&SPEC_WAIT),
        "disown" => Some(&SPEC_DISOWN),
        "fg" => Some(&SPEC_FG),
        "bg" => Some(&SPEC_BG),
        "fsam" | "sammary" => Some(&SPEC_SAMMARY),
        "glob" => Some(&SPEC_GLOB),
        "confirm" => Some(&SPEC_CONFIRM),
        "input" => Some(&SPEC_INPUT),
        "select" => Some(&SPEC_SELECT),
        "env" => Some(&SPEC_ENV),
        "command" => Some(&SPEC_COMMAND),
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

fn option_summary(option: &BuiltinOptionSpec) -> String {
    match option.value {
        Some(kind) => format!("{} {}", option.summary, kind.placeholder()),
        None => option.summary.to_string(),
    }
}

fn find_option<'a>(token: &str, options: &'a [BuiltinOptionSpec]) -> Option<&'a BuiltinOptionSpec> {
    let exact = options.iter().find(|option| option.names.contains(&token));
    if exact.is_some() {
        return exact;
    }
    options
        .iter()
        .find(|option| option.value.is_some() && option_consumes_inline_value(token, option))
}

fn option_consumes_inline_value(token: &str, option: &BuiltinOptionSpec) -> bool {
    option.value.is_some()
        && option.names.iter().any(|name| {
            token
                .strip_prefix(&format!("{name}="))
                .is_some_and(|value| !value.is_empty())
                || (name.starts_with('-')
                    && !name.starts_with("--")
                    && token.starts_with(name)
                    && token.len() > name.len())
        })
}

fn described_pair(replacement: &str, summary: &str) -> Pair {
    Pair {
        display: format!("{replacement}  {summary}"),
        replacement: replacement.to_string(),
    }
}

fn dedupe_pairs(pairs: Vec<Pair>) -> Vec<Pair> {
    let mut seen = BTreeSet::new();
    pairs
        .into_iter()
        .filter(|pair| seen.insert(pair.replacement.clone()))
        .collect()
}

fn complete_path_pairs(cwd: &Path, word: &str) -> Vec<Pair> {
    let (dir_prefix, file_prefix) = split_path_prefix(word);
    let search_dir = resolve_completion_dir(cwd, &dir_prefix);
    let mut entries: Vec<_> = std::fs::read_dir(search_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if !file_name.starts_with(&file_prefix) {
                return None;
            }
            if !file_prefix.starts_with('.') && file_name.starts_with('.') {
                return None;
            }

            let metadata = entry.metadata().ok()?;
            let suffix = if metadata.is_dir() { "/" } else { "" };
            let replacement = format!("{}{}{}", dir_prefix, escape_shell_text(&file_name), suffix);
            let summary = if metadata.is_dir() {
                "directory"
            } else {
                "path"
            };
            Some(described_pair(&replacement, summary))
        })
        .collect();
    entries.sort_by(|left, right| left.replacement.cmp(&right.replacement));
    entries
}

fn split_path_prefix(word: &str) -> (String, String) {
    if word == "~" {
        return ("~/".to_string(), String::new());
    }
    if let Some(index) = word.rfind('/') {
        return (word[..=index].to_string(), word[index + 1..].to_string());
    }
    (String::new(), word.to_string())
}

fn resolve_completion_dir(cwd: &Path, dir_prefix: &str) -> std::path::PathBuf {
    if dir_prefix.is_empty() {
        return cwd.to_path_buf();
    }
    if let Some(rest) = dir_prefix.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return Path::new(&home).join(rest);
    }
    let path = Path::new(dir_prefix);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn escape_shell_text(value: &str) -> String {
    value.replace(' ', "\\ ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rustyline::{Context, completion::Completer, hint::Hinter, history::DefaultHistory};
    use tempfile::tempdir;

    use crate::repl::{ReplJobCandidate, UshHelper};

    fn helper(cwd: PathBuf) -> UshHelper {
        let mut helper = UshHelper::new(
            vec![
                "help".to_string(),
                "unset".to_string(),
                "source".to_string(),
                "rm".to_string(),
                "echo".to_string(),
                "grep".to_string(),
            ],
            vec!["HOME".to_string(), "PATH".to_string(), "PWD".to_string()],
            cwd,
        );
        helper.refresh(
            vec![
                "help".to_string(),
                "unset".to_string(),
                "source".to_string(),
                "rm".to_string(),
                "echo".to_string(),
                "grep".to_string(),
            ],
            vec!["HOME".to_string(), "PATH".to_string(), "PWD".to_string()],
            helper.cwd.clone(),
            vec!["ll".to_string()],
            vec![ReplJobCandidate {
                spec: "%1".to_string(),
                summary: "Running  sleep 10".to_string(),
            }],
        );
        helper
    }

    #[test]
    fn completes_builtin_topics_and_env_names() {
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);
        let helper = helper(PathBuf::from("."));

        let (_, help_pairs) = helper.complete("help so", 7, &ctx).expect("help complete");
        let (_, unset_pairs) = helper
            .complete("unset PA", 8, &ctx)
            .expect("unset complete");

        assert!(help_pairs.iter().any(|pair| pair.replacement == "source"));
        assert!(unset_pairs.iter().any(|pair| pair.replacement == "PATH"));
    }

    #[test]
    fn completes_job_specs_and_signal_names() {
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);
        let helper = helper(PathBuf::from("."));

        let (_, fg_pairs) = helper.complete("fg ", 3, &ctx).expect("fg complete");
        let (_, signal_pairs) = helper
            .complete("stop --signal KI", 16, &ctx)
            .expect("stop complete");

        assert!(fg_pairs.iter().any(|pair| pair.replacement == "%1"));
        assert!(signal_pairs.iter().any(|pair| pair.replacement == "KILL"));
    }

    #[test]
    fn completes_source_paths_relative_to_shell_cwd() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(dir.path().join("script.ush"), "echo ok\n").expect("write script");
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);
        let helper = helper(dir.path().to_path_buf());

        let (_, pairs) = helper
            .complete("source sc", 9, &ctx)
            .expect("source complete");

        assert!(pairs.iter().any(|pair| pair.replacement == "script.ush"));
    }

    #[test]
    fn hints_builtin_usage_and_top_level_builtin_summaries() {
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);
        let helper = helper(PathBuf::from("."));

        let (_, command_pairs) = helper.complete("he", 2, &ctx).expect("complete");

        assert!(command_pairs.iter().any(|pair| {
            pair.replacement == "help" && pair.display.contains("show builtin help")
        }));
        assert_eq!(
            helper.hint("rm ", 3, &ctx),
            Some(
                "  guarded wrapper around rm for recursive deletes  next: <path>  usage: [--yes] [<path>...]".to_string()
            )
        );
    }
}
