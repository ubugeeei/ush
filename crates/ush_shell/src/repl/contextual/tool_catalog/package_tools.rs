use super::super::options::{OptionSpec, option_spec};

pub(crate) const BUN_COMMANDS: &[&str] = &[
    "run", "test", "x", "repl", "exec", "install", "add", "remove", "update", "audit", "outdated",
    "link", "unlink", "publish", "patch", "pm", "info", "build", "init", "create", "upgrade",
];
pub(crate) const BUN_OPTIONS: &[&str] = &[
    "-c",
    "-e",
    "-h",
    "-i",
    "-p",
    "-r",
    "--help",
    "--watch",
    "--hot",
    "--smol",
    "--preload",
    "--inspect",
    "--install",
    "--no-install",
    "--env-file",
    "--cwd",
    "--config",
];
pub(crate) const BUN_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["--env-file", "--cwd", "--config"], 1, true, false),
    option_spec(&["--preload", "--inspect", "--install"], 1, false, false),
];

pub(crate) const PNPM_COMMANDS: &[&str] = &[
    "add",
    "install",
    "link",
    "remove",
    "unlink",
    "update",
    "audit",
    "list",
    "outdated",
    "why",
    "create",
    "dlx",
    "exec",
    "run",
    "config",
    "init",
    "publish",
    "self-update",
];
pub(crate) const PNPM_OPTIONS: &[&str] = &["--help", "-r", "--recursive", "--filter"];

pub(crate) const YARN_COMMANDS: &[&str] = &[
    "add",
    "config",
    "create",
    "dlx",
    "exec",
    "init",
    "install",
    "npm",
    "remove",
    "run",
    "set",
    "test",
    "up",
    "why",
    "workspaces",
];
pub(crate) const YARN_OPTIONS: &[&str] = &["-h", "--help", "--cwd", "--top-level", "--verbose"];
pub(crate) const YARN_OPTION_SPECS: &[OptionSpec] = &[option_spec(&["--cwd"], 1, true, false)];
