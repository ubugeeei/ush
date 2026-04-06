use super::types::{ArgKind, BuiltinOptionSpec};

const YES_NO_CHOICES: &[&str] = &["yes", "no"];
pub(in crate::repl::builtin_completion) const SIGNAL_CHOICES: &[&str] = &[
    "TERM", "KILL", "INT", "HUP", "QUIT", "USR1", "USR2", "STOP", "CONT",
];

pub(in crate::repl::builtin_completion) const ECHO_OPTIONS: &[BuiltinOptionSpec] =
    &[BuiltinOptionSpec {
        names: &["-n"],
        summary: "omit the trailing newline",
        value: None,
    }];

pub(in crate::repl::builtin_completion) const UNSET_OPTIONS: &[BuiltinOptionSpec] =
    &[BuiltinOptionSpec {
        names: &["-v"],
        summary: "remove shell variables",
        value: None,
    }];

pub(in crate::repl::builtin_completion) const STOP_OPTIONS: &[BuiltinOptionSpec] =
    &[BuiltinOptionSpec {
        names: &["--signal"],
        summary: "choose the signal to send",
        value: Some(ArgKind::Signal),
    }];

pub(in crate::repl::builtin_completion) const SAMMARY_OPTIONS: &[BuiltinOptionSpec] =
    &[BuiltinOptionSpec {
        names: &["--include-lock"],
        summary: "include lockfiles in the report",
        value: None,
    }];

pub(in crate::repl::builtin_completion) const CONFIRM_OPTIONS: &[BuiltinOptionSpec] = &[
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

pub(in crate::repl::builtin_completion) const INPUT_OPTIONS: &[BuiltinOptionSpec] = &[
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

pub(in crate::repl::builtin_completion) const SELECT_OPTIONS: &[BuiltinOptionSpec] = &[
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

pub(in crate::repl::builtin_completion) const RM_OPTIONS: &[BuiltinOptionSpec] =
    &[BuiltinOptionSpec {
        names: &["--yes"],
        summary: "skip the recursive delete confirmation",
        value: None,
    }];

pub(in crate::repl::builtin_completion) const COMMAND_OPTIONS: &[BuiltinOptionSpec] = &[
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
