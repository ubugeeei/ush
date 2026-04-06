#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::repl::git_completion) enum ArgKind {
    Branch,
    Commit,
    Config,
    LocalBranch,
    Message,
    Number,
    Path,
    Pathspec,
    Ref,
    Remote,
    RemoteBranch,
    Stash,
    Tag,
    Url,
}

impl ArgKind {
    pub(in crate::repl::git_completion) fn placeholder(self) -> &'static str {
        match self {
            Self::Branch | Self::LocalBranch => "<branch>",
            Self::Commit => "<commit>",
            Self::Config => "<name=value>",
            Self::Message => "<message>",
            Self::Number => "<n>",
            Self::Path => "<path>",
            Self::Pathspec => "<pathspec>",
            Self::Ref => "<ref>",
            Self::Remote => "<remote>",
            Self::RemoteBranch => "<remote/branch>",
            Self::Stash => "<stash>",
            Self::Tag => "<tag>",
            Self::Url => "<url>",
        }
    }

    pub(in crate::repl::git_completion) fn path_like(self) -> bool {
        matches!(self, Self::Path | Self::Pathspec)
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::repl::git_completion) struct GitOptionSpec {
    pub(in crate::repl::git_completion) names: &'static [&'static str],
    pub(in crate::repl::git_completion) summary: &'static str,
    pub(in crate::repl::git_completion) value: Option<ArgKind>,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::repl::git_completion) struct GitCommandSpec {
    pub(in crate::repl::git_completion) name: &'static str,
    pub(in crate::repl::git_completion) summary: &'static str,
    pub(in crate::repl::git_completion) usage: &'static str,
    pub(in crate::repl::git_completion) options: &'static [GitOptionSpec],
    pub(in crate::repl::git_completion) positionals: &'static [ArgKind],
    pub(in crate::repl::git_completion) trailing: Option<ArgKind>,
    pub(in crate::repl::git_completion) after_double_dash: Option<ArgKind>,
}

impl GitCommandSpec {
    pub(in crate::repl::git_completion) fn positional_kind(self, index: usize) -> Option<ArgKind> {
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
