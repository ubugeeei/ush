use super::common::{COMMAND_OPTIONS, ECHO_OPTIONS, STOP_OPTIONS, UNSET_OPTIONS};
use super::types::{ArgKind, BuiltinSpec};

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

pub(super) fn resolve(name: &str) -> Option<&'static BuiltinSpec> {
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
        "env" => Some(&SPEC_ENV),
        "command" => Some(&SPEC_COMMAND),
        _ => None,
    }
}
