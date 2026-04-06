use std::path::Path;

use compact_str::CompactString;

use super::{
    candidates::typed_candidate_pairs,
    discover::load_npm_scripts,
    options::{OptionSpec, pending_value_kind, positional_args},
    tool_catalog::{
        BUN_COMMANDS, BUN_OPTION_SPECS, BUN_OPTIONS, CARGO_COMMANDS, CARGO_OPTION_SPECS,
        CARGO_OPTIONS, CLAUDE_COMMANDS, CLAUDE_OPTION_SPECS, CLAUDE_OPTIONS, CODEX_COMMANDS,
        CODEX_OPTION_SPECS, CODEX_OPTIONS, GO_COMMANDS, GO_HELP_TOPICS, GO_MOD_COMMANDS,
        GO_WORK_COMMANDS, MOON_COMMANDS, MOON_OPTION_SPECS, MOON_OPTIONS, NODE_COMMANDS,
        NODE_OPTION_SPECS, NODE_OPTIONS, PNPM_COMMANDS, PNPM_OPTIONS, YARN_COMMANDS,
        YARN_OPTION_SPECS, YARN_OPTIONS, ZIG_COMMANDS,
    },
    types::ContextualCompletion,
};
use crate::repl::descriptions;

struct ScriptRunnerSpec {
    commands: &'static [&'static str],
    options: &'static [&'static str],
    option_specs: &'static [OptionSpec],
    command_detail: &'static str,
    option_detail: &'static str,
    script_detail: &'static str,
}

const BUN_RUNNER: ScriptRunnerSpec = ScriptRunnerSpec {
    commands: BUN_COMMANDS,
    options: BUN_OPTIONS,
    option_specs: BUN_OPTION_SPECS,
    command_detail: descriptions::BUN_COMMAND,
    option_detail: descriptions::OPTION,
    script_detail: descriptions::BUN_SCRIPT,
};

const PNPM_RUNNER: ScriptRunnerSpec = ScriptRunnerSpec {
    commands: PNPM_COMMANDS,
    options: PNPM_OPTIONS,
    option_specs: &[],
    command_detail: descriptions::PNPM_COMMAND,
    option_detail: descriptions::OPTION,
    script_detail: descriptions::PNPM_SCRIPT,
};

const YARN_RUNNER: ScriptRunnerSpec = ScriptRunnerSpec {
    commands: YARN_COMMANDS,
    options: YARN_OPTIONS,
    option_specs: YARN_OPTION_SPECS,
    command_detail: descriptions::YARN_COMMAND,
    option_detail: descriptions::OPTION,
    script_detail: descriptions::YARN_SCRIPT,
};

pub(crate) fn complete_cargo(args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    static_completion(
        args,
        word,
        CARGO_COMMANDS,
        CARGO_OPTIONS,
        CARGO_OPTION_SPECS,
        descriptions::CARGO_COMMAND,
        descriptions::OPTION,
    )
}

pub(crate) fn complete_moon(args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    static_completion(
        args,
        word,
        MOON_COMMANDS,
        MOON_OPTIONS,
        MOON_OPTION_SPECS,
        descriptions::MOON_COMMAND,
        descriptions::OPTION,
    )
}

pub(crate) fn complete_go(args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    let positionals = positional_args(args, &[]);
    let items = match positionals.first().map(CompactString::as_str) {
        Some("help") => {
            return Some(ContextualCompletion::Pairs(typed_candidate_pairs(
                word,
                GO_HELP_TOPICS.iter().copied(),
                descriptions::GO_HELP,
            )));
        }
        Some("mod") => {
            return Some(ContextualCompletion::Pairs(typed_candidate_pairs(
                word,
                GO_MOD_COMMANDS.iter().copied(),
                descriptions::GO_MOD,
            )));
        }
        Some("work") => {
            return Some(ContextualCompletion::Pairs(typed_candidate_pairs(
                word,
                GO_WORK_COMMANDS.iter().copied(),
                descriptions::GO_WORK,
            )));
        }
        _ => GO_COMMANDS.iter().copied(),
    };
    Some(ContextualCompletion::Pairs(typed_candidate_pairs(
        word,
        items,
        descriptions::GO_COMMAND,
    )))
}

pub(crate) fn complete_zig(word: &str) -> Option<ContextualCompletion> {
    Some(ContextualCompletion::Pairs(typed_candidate_pairs(
        word,
        ZIG_COMMANDS.iter().copied(),
        descriptions::ZIG_COMMAND,
    )))
}

pub(crate) fn complete_node(args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    if matches!(pending_value_kind(args, NODE_OPTION_SPECS), Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if word.starts_with('-') {
        return Some(ContextualCompletion::Pairs(typed_candidate_pairs(
            word,
            NODE_OPTIONS.iter().copied(),
            descriptions::NODE_OPTION,
        )));
    }
    Some(ContextualCompletion::Pairs(typed_candidate_pairs(
        word,
        NODE_COMMANDS.iter().copied(),
        descriptions::NODE_COMMAND,
    )))
}

pub(crate) fn complete_bun(
    cwd: &Path,
    args: &[CompactString],
    word: &str,
) -> Option<ContextualCompletion> {
    script_runner_completion(cwd, args, word, &BUN_RUNNER)
}

pub(crate) fn complete_pnpm(
    cwd: &Path,
    args: &[CompactString],
    word: &str,
) -> Option<ContextualCompletion> {
    script_runner_completion(cwd, args, word, &PNPM_RUNNER)
}

pub(crate) fn complete_yarn(
    cwd: &Path,
    args: &[CompactString],
    word: &str,
) -> Option<ContextualCompletion> {
    script_runner_completion(cwd, args, word, &YARN_RUNNER)
}

pub(crate) fn complete_claude(args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    static_completion(
        args,
        word,
        CLAUDE_COMMANDS,
        CLAUDE_OPTIONS,
        CLAUDE_OPTION_SPECS,
        descriptions::CLAUDE_COMMAND,
        descriptions::OPTION,
    )
}

pub(crate) fn complete_codex(args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    static_completion(
        args,
        word,
        CODEX_COMMANDS,
        CODEX_OPTIONS,
        CODEX_OPTION_SPECS,
        descriptions::CODEX_COMMAND,
        descriptions::OPTION,
    )
}

fn static_completion(
    args: &[CompactString],
    word: &str,
    commands: &[&str],
    options: &[&str],
    specs: &[OptionSpec],
    detail: &'static str,
    option_detail: &'static str,
) -> Option<ContextualCompletion> {
    let pending = pending_value_kind(args, specs);
    if matches!(pending, Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending.is_some() {
        return None;
    }
    let (items, label) = if word.starts_with('-') {
        (options, option_detail)
    } else {
        (commands, detail)
    };
    Some(ContextualCompletion::Pairs(typed_candidate_pairs(
        word,
        items.iter().copied(),
        label,
    )))
}

fn script_runner_completion(
    cwd: &Path,
    args: &[CompactString],
    word: &str,
    spec: &ScriptRunnerSpec,
) -> Option<ContextualCompletion> {
    let pending = pending_value_kind(args, spec.option_specs);
    if matches!(pending, Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending.is_some() {
        return None;
    }

    let positionals = positional_args(args, spec.option_specs);
    if matches!(positionals.first().map(CompactString::as_str), Some("run")) {
        return Some(ContextualCompletion::Pairs(typed_candidate_pairs(
            word,
            load_npm_scripts(cwd),
            spec.script_detail,
        )));
    }

    let (items, detail) = if word.starts_with('-') {
        (spec.options, spec.option_detail)
    } else {
        (spec.commands, spec.command_detail)
    };
    Some(ContextualCompletion::Pairs(typed_candidate_pairs(
        word,
        items.iter().copied(),
        detail,
    )))
}
