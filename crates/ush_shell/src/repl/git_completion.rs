mod context;
mod pairs;
mod specs;
#[cfg(test)]
mod tests;

use super::UshHelper;
use context::{parse_context, parse_options};
use pairs::{
    command_hint, command_pairs, complete_arg, dedupe_pairs, matching_commands, matching_options,
    option_hint, option_pairs,
};
use specs::GLOBAL_OPTIONS;

pub fn complete(
    helper: &UshHelper,
    line: &str,
    pos: usize,
) -> rustyline::Result<Option<(usize, Vec<rustyline::completion::Pair>)>> {
    let Some(context) = parse_context(line, pos) else {
        return Ok(None);
    };

    if let Some(kind) = context.global_expected_value {
        return Ok(Some(complete_arg(
            helper,
            context.start,
            &context.word,
            kind,
        )));
    }

    let Some(spec) = context.subcommand else {
        if context.raw_subcommand.is_some() {
            return Ok(Some((context.start, Vec::new())));
        }

        let mut pairs = command_pairs(&context.word);
        if context.word.is_empty() || context.word.starts_with('-') {
            pairs.extend(option_pairs(&context.word, GLOBAL_OPTIONS));
        }
        return Ok(Some((context.start, dedupe_pairs(pairs))));
    };

    let parsed = parse_options(&context.subcommand_args, spec.options);
    if let Some(kind) = parsed.expected_value {
        return Ok(Some(complete_arg(
            helper,
            context.start,
            &context.word,
            kind,
        )));
    }

    if !parsed.after_double_dash && context.word.starts_with('-') {
        return Ok(Some((
            context.start,
            option_pairs(&context.word, spec.options),
        )));
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
        pairs = dedupe_pairs(pairs);
    }

    Ok(Some((context.start, pairs)))
}

pub fn hint(_helper: &UshHelper, line: &str, pos: usize) -> Option<String> {
    let context = parse_context(line, pos)?;

    if let Some(kind) = context.global_expected_value {
        return Some(format!("  global option value: {}", kind.placeholder()));
    }

    let Some(spec) = context.subcommand else {
        if context.raw_subcommand.is_some() {
            return None;
        }
        if context.word.starts_with('-') {
            let options = matching_options(&context.word, GLOBAL_OPTIONS);
            return option_hint(options);
        }
        let matches = matching_commands(&context.word);
        return if matches.len() == 1 {
            Some(command_hint(matches[0], None))
        } else {
            Some(
                "  next: <subcommand>  common: status, add, commit, switch, checkout, branch, push, pull, fetch"
                    .to_string(),
            )
        };
    };

    let parsed = parse_options(&context.subcommand_args, spec.options);
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
    Some(command_hint(spec, expected))
}
