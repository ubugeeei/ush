mod context;
mod pairs;
mod specs;
#[cfg(test)]
mod tests;

use super::UshHelper;
use context::{parse_context, parse_options};
use pairs::{command_hint, complete_arg, matching_options, option_hint, option_pairs};
use specs::resolve_builtin_spec;

pub fn complete(
    helper: &UshHelper,
    line: &str,
    pos: usize,
) -> rustyline::Result<Option<(usize, Vec<rustyline::completion::Pair>)>> {
    let Some(context) = parse_context(line, pos) else {
        return Ok(None);
    };

    match context.command.as_str() {
        "env" => Ok(Some(complete_env(helper, &context))),
        "command" => Ok(Some(complete_command_builtin(helper, &context))),
        _ => Ok(Some(complete_builtin_spec(helper, &context))),
    }
}

pub fn hint(helper: &UshHelper, line: &str, pos: usize) -> Option<String> {
    let context = parse_context(line, pos)?;

    match context.command.as_str() {
        "env" => hint_env(&context),
        "command" => hint_command_builtin(&context),
        _ => hint_builtin_spec(helper, &context),
    }
}

pub(super) fn command_summary(name: &str) -> Option<&'static str> {
    specs::command_summary(name)
}

fn complete_builtin_spec(
    helper: &UshHelper,
    context: &context::BuiltinContext,
) -> (usize, Vec<rustyline::completion::Pair>) {
    let spec = resolve_builtin_spec(&context.command).expect("builtin spec");
    let parsed = parse_options(&context.args, spec.options);

    if let Some(kind) = parsed.expected_value {
        return complete_arg(helper, context.start, &context.word, kind);
    }
    if !parsed.after_double_dash && context.word.starts_with('-') {
        return (context.start, option_pairs(&context.word, spec.options));
    }

    let expected = if parsed.after_double_dash {
        spec.after_double_dash
    } else {
        spec.positional_kind(parsed.positionals.len())
    };
    let mut pairs = if let Some(kind) = expected {
        complete_arg(helper, context.start, &context.word, kind).1
    } else {
        Vec::new()
    };

    if context.word.is_empty() && !parsed.after_double_dash {
        pairs.extend(option_pairs("", spec.options));
        pairs = pairs::dedupe_pairs(pairs);
    }

    (context.start, pairs)
}

fn hint_builtin_spec(helper: &UshHelper, context: &context::BuiltinContext) -> Option<String> {
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

    if matches!(expected, Some(specs::ArgKind::Job)) && helper.jobs.is_empty() {
        let mut hint = command_hint(spec, None);
        hint.push_str("  (no tracked jobs)");
        return Some(hint);
    }

    Some(command_hint(spec, expected))
}

fn complete_env(
    helper: &UshHelper,
    context: &context::BuiltinContext,
) -> (usize, Vec<rustyline::completion::Pair>) {
    let mut index = 0usize;
    while let Some(token) = context.args.get(index) {
        if super::syntax::is_assignment(token) {
            index += 1;
            continue;
        }
        break;
    }

    if context.word.contains('=') {
        return (context.start, Vec::new());
    }
    if super::syntax::is_assignment(&context.word)
        || (!context.word.is_empty() && index == context.args.len())
    {
        return (
            context.start,
            pairs::env_assignment_pairs(helper, &context.word),
        );
    }
    if index == context.args.len() {
        return (
            context.start,
            pairs::command_name_pairs(helper, &context.word),
        );
    }
    (context.start, Vec::new())
}

fn hint_env(context: &context::BuiltinContext) -> Option<String> {
    let mut index = 0usize;
    while let Some(token) = context.args.get(index) {
        if super::syntax::is_assignment(token) {
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
    context: &context::BuiltinContext,
) -> (usize, Vec<rustyline::completion::Pair>) {
    let option_mode = context
        .args
        .first()
        .is_some_and(|arg| matches!(arg.as_str(), "-v" | "-V"));
    if context.word.starts_with('-') || (context.args.is_empty() && context.word.is_empty()) {
        let mut pairs = option_pairs(&context.word, specs::COMMAND_OPTIONS);
        if context.word.is_empty() && !option_mode {
            pairs.extend(pairs::command_name_pairs(helper, ""));
            pairs = pairs::dedupe_pairs(pairs);
        }
        return (context.start, pairs);
    }

    if option_mode || context.args.is_empty() {
        return (
            context.start,
            pairs::command_name_pairs(helper, &context.word),
        );
    }

    (context.start, Vec::new())
}

fn hint_command_builtin(context: &context::BuiltinContext) -> Option<String> {
    if context.word.starts_with('-') {
        let options = matching_options(&context.word, specs::COMMAND_OPTIONS);
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
