use phf::phf_map;

use super::options::{OptionSpec, option_spec};

pub(crate) const GIT_GLOBAL_OPTIONS: &[&str] = &[
    "--help", "--version", "--paginate", "--no-pager", "-C", "-c", "--git-dir", "--work-tree",
    "--namespace",
];
pub(crate) const GIT_SUBCOMMANDS: &[&str] = &[
    "add", "am", "archive", "bisect", "branch", "bundle", "checkout", "cherry-pick", "clean",
    "clone", "commit", "config", "describe", "diff", "fetch", "format-patch", "grep", "help",
    "init", "log", "merge", "mv", "pull", "push", "rebase", "reflog", "remote", "reset",
    "restore", "revert", "rm", "show", "shortlog", "stash", "status", "submodule", "switch",
    "tag", "worktree",
];
pub(crate) const GIT_REMOTE_SUBCOMMANDS: &[&str] = &[
    "add", "get-url", "prune", "remove", "rename", "set-branches", "set-head", "set-url", "show",
    "update",
];
pub(crate) static GIT_SUBCOMMAND_OPTIONS: phf::Map<&'static str, &'static [&'static str]> =
    phf_map! {
        "add" => &["-A", "-u", "-p", "--all", "--patch", "--update"],
        "branch" => &["-a", "-d", "-D", "-m", "-M", "--all", "--list"],
        "checkout" => &["-b", "-B", "--detach", "--ours", "--theirs", "--track", "--"],
        "commit" => &["-a", "-m", "--amend", "--fixup", "--no-edit"],
        "diff" => &["--cached", "--name-only", "--staged", "--stat", "--"],
        "fetch" => &["--all", "--prune", "--tags"],
        "log" => &["--graph", "--name-only", "--oneline", "--stat"],
        "merge" => &["--abort", "--continue", "--ff-only", "--no-ff"],
        "pull" => &["--ff-only", "--rebase", "--tags"],
        "push" => &["-f", "-u", "--force", "--set-upstream", "--tags"],
        "rebase" => &["-i", "--abort", "--continue", "--onto"],
        "reset" => &["--hard", "--mixed", "--soft"],
        "restore" => &["--source", "--staged", "--worktree", "--"],
        "show" => &["--name-only", "--stat"],
        "status" => &["-b", "-s", "--branch", "--short"],
        "switch" => &["-c", "-C", "--detach", "--guess", "--track"],
    };

pub(crate) const MAKE_OPTIONS: &[&str] = &[
    "-B", "--always-make", "-C", "--directory", "-f", "--file", "--makefile", "-j", "--jobs",
    "-k", "--keep-going", "-n", "--dry-run", "-q", "--question", "-s", "--silent",
    "--warn-undefined-variables",
];
pub(crate) const JUST_OPTIONS: &[&str] = &[
    "--allow-missing", "--check", "--choose", "--command", "--dump", "--dump-format", "--edit",
    "--evaluate", "--fmt", "--global-justfile", "--groups", "--help", "--init", "--justfile",
    "--list", "--man", "--no-deps", "--quiet", "--set", "--show", "--summary", "--unsorted",
    "--usage", "--variables", "--working-directory", "-c", "-d", "-f", "-l", "-q", "-s", "-u",
    "-v",
];
pub(crate) const MISE_TOP_LEVEL: &[&str] = &[
    "activate", "backends", "bin-paths", "cache", "completion", "config", "doctor", "env",
    "exec", "fmt", "generate", "install", "latest", "lock", "ls", "outdated", "plugins",
    "prepare", "prune", "registry", "reshim", "run", "search", "set", "settings", "shell",
    "sync", "tasks", "trust", "uninstall", "upgrade", "use", "watch", "where", "which",
];
pub(crate) const MISE_TASKS_SUBCOMMANDS: &[&str] =
    &["add", "deps", "edit", "info", "ls", "run", "validate"];
pub(crate) const NPM_COMMANDS: &[&str] = &[
    "access", "audit", "cache", "ci", "completion", "config", "dedupe", "diff", "doctor",
    "exec", "help", "init", "install", "link", "ls", "outdated", "pack", "prefix", "publish",
    "query", "rebuild", "repo", "run", "start", "stop", "test", "uninstall", "update",
    "version", "view",
];
pub(crate) const VP_COMMANDS: &[&str] = &["build", "dev", "optimize", "preview", "serve"];
pub(crate) const VP_OPTIONS: &[&str] = &[
    "--base", "--clearScreen", "--config", "--debug", "--host", "--https", "--mode", "--open",
    "--port", "--strictPort",
];

pub(crate) const MAKE_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["-C", "--directory"], 1, true, false),
    option_spec(&["-f", "--file", "--makefile"], 1, true, true),
    option_spec(&["-I", "--include-dir", "-o", "--old-file", "--assume-old"], 1, true, true),
    option_spec(&["-W", "--what-if", "--new-file", "--assume-new"], 1, true, true),
    option_spec(&["-j", "--jobs", "-l", "--load-average", "--max-load"], 1, false, true),
    option_spec(&["--debug", "-N", "--NeXT-option"], 1, false, true),
];
pub(crate) const JUST_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["--chooser"], 1, false, false),
    option_spec(&["--command"], 1, false, false),
    option_spec(&["--dump-format"], 1, false, false),
    option_spec(&["--justfile", "-f"], 1, true, true),
    option_spec(&["--set"], 2, false, false),
    option_spec(&["--shell"], 1, false, false),
    option_spec(&["--shell-arg"], 1, false, false),
    option_spec(&["--show", "-s"], 1, false, false),
    option_spec(&["--tempdir"], 1, true, false),
    option_spec(&["--usage"], 1, false, false),
    option_spec(&["--working-directory", "-d"], 1, true, true),
];
pub(crate) const MISE_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["-C", "--cd"], 1, true, false),
    option_spec(&["-E", "--env"], 1, false, false),
    option_spec(&["-j", "--jobs"], 1, false, false),
    option_spec(&["--output"], 1, false, false),
];
pub(crate) const NPM_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["--workspace", "-w"], 1, false, false),
    option_spec(&["--prefix"], 1, true, false),
    option_spec(&["--userconfig"], 1, true, false),
];
pub(crate) const GIT_GLOBAL_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["-C"], 1, true, true),
    option_spec(&["-c"], 1, false, false),
    option_spec(&["--git-dir", "--work-tree", "--namespace"], 1, true, false),
];
