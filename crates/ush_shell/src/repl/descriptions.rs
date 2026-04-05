use phf::phf_map;

pub(crate) const BUILTIN: &str = "shell builtin";
pub(crate) const KEYWORD: &str = "shell keyword";
pub(crate) const COMMAND: &str = "command";
pub(crate) const OPTION: &str = "option";
pub(crate) const ENV_VAR: &str = "environment variable";
pub(crate) const ENV_BINDING: &str = "set environment variable";
pub(crate) const GIT_COMMAND: &str = "git command";
pub(crate) const GIT_REMOTE_COMMAND: &str = "git remote command";
pub(crate) const GIT_REF: &str = "git ref";
pub(crate) const GIT_REMOTE: &str = "git remote";
pub(crate) const GIT_STASH: &str = "git stash command";
pub(crate) const MAKE_TARGET: &str = "make target";
pub(crate) const JUST_RECIPE: &str = "just recipe";
pub(crate) const MISE_COMMAND: &str = "mise command";
pub(crate) const MISE_TASK: &str = "mise task";
pub(crate) const NPM_COMMAND: &str = "npm command";
pub(crate) const NPM_SCRIPT: &str = "npm script";
pub(crate) const VP_COMMAND: &str = "vp command";
pub(crate) const CARGO_COMMAND: &str = "cargo command";
pub(crate) const MOON_COMMAND: &str = "moon command";
pub(crate) const GO_COMMAND: &str = "go command";
pub(crate) const GO_HELP: &str = "go help topic";
pub(crate) const GO_MOD: &str = "go mod command";
pub(crate) const GO_WORK: &str = "go work command";
pub(crate) const ZIG_COMMAND: &str = "zig command";
pub(crate) const NODE_COMMAND: &str = "node command";
pub(crate) const NODE_OPTION: &str = "node option";
pub(crate) const BUN_COMMAND: &str = "bun command";
pub(crate) const BUN_SCRIPT: &str = "bun script";
pub(crate) const PNPM_COMMAND: &str = "pnpm command";
pub(crate) const PNPM_SCRIPT: &str = "pnpm script";
pub(crate) const YARN_COMMAND: &str = "yarn command";
pub(crate) const YARN_SCRIPT: &str = "yarn script";
pub(crate) const CLAUDE_COMMAND: &str = "claude command";
pub(crate) const CODEX_COMMAND: &str = "codex command";

static COMMAND_HELP: phf::Map<&'static str, &'static str> = phf_map! {
    "bun" => "JavaScript runtime and package manager",
    "cargo" => "Rust package manager",
    "claude" => "Anthropic coding agent",
    "codex" => "OpenAI coding agent",
    "git" => "distributed version control",
    "go" => "Go toolchain",
    "just" => "command runner",
    "make" => "build automation tool",
    "mise" => "tool and task manager",
    "moon" => "MoonBit build tool",
    "node" => "JavaScript runtime",
    "npm" => "Node package manager",
    "pnpm" => "workspace-aware package manager",
    "vp" => "Vite command alias",
    "vite" => "frontend build tool",
    "yarn" => "package manager",
    "zig" => "Zig toolchain",
};

pub(crate) fn command(name: &str, builtin: bool, keyword: bool) -> &'static str {
    if builtin {
        return BUILTIN;
    }
    if keyword {
        return KEYWORD;
    }
    COMMAND_HELP.get(name).copied().unwrap_or(COMMAND)
}
