#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::repl::builtin_completion) enum ArgKind {
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
    pub(in crate::repl::builtin_completion) fn placeholder(self) -> &'static str {
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

    pub(in crate::repl::builtin_completion) fn path_like(self) -> bool {
        matches!(self, Self::Path)
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::repl::builtin_completion) struct BuiltinOptionSpec {
    pub(in crate::repl::builtin_completion) names: &'static [&'static str],
    pub(in crate::repl::builtin_completion) summary: &'static str,
    pub(in crate::repl::builtin_completion) value: Option<ArgKind>,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::repl::builtin_completion) struct BuiltinSpec {
    pub(in crate::repl::builtin_completion) summary: &'static str,
    pub(in crate::repl::builtin_completion) usage: &'static str,
    pub(in crate::repl::builtin_completion) options: &'static [BuiltinOptionSpec],
    pub(in crate::repl::builtin_completion) positionals: &'static [ArgKind],
    pub(in crate::repl::builtin_completion) trailing: Option<ArgKind>,
    pub(in crate::repl::builtin_completion) after_double_dash: Option<ArgKind>,
}

impl BuiltinSpec {
    pub(in crate::repl::builtin_completion) fn positional_kind(
        self,
        index: usize,
    ) -> Option<ArgKind> {
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
