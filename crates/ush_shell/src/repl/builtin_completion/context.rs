use crate::{commands::is_builtin, repl::syntax};

use super::specs::{ArgKind, BuiltinOptionSpec};

#[derive(Debug, Default)]
pub(super) struct ParsedOptions {
    pub(super) positionals: Vec<String>,
    pub(super) expected_value: Option<ArgKind>,
    pub(super) after_double_dash: bool,
}

#[derive(Debug)]
pub(super) struct BuiltinContext {
    pub(super) start: usize,
    pub(super) word: String,
    pub(super) command: String,
    pub(super) args: Vec<String>,
}

pub(super) fn parse_context(line: &str, pos: usize) -> Option<BuiltinContext> {
    let prefix = &line[..pos];
    let start = syntax::word_start(line, pos);
    let word = prefix[start..].to_string();
    let tokens = syntax::tokenize(&prefix[..start]);
    let segment_start = tokens
        .iter()
        .rposition(|token| matches!(token.as_str(), "|" | "||" | "&&" | ";" | "&"))
        .map_or(0, |index| index + 1);
    let segment = &tokens[segment_start..];
    let command_index = segment.iter().position(|token| {
        !(syntax::is_assignment(token) || matches!(token.as_str(), "!" | "command"))
    })?;
    let command = segment.get(command_index)?.clone();
    if !is_builtin(&command) {
        return None;
    }

    Some(BuiltinContext {
        start,
        word,
        command,
        args: segment[command_index + 1..].to_vec(),
    })
}

pub(super) fn parse_options(tokens: &[String], options: &[BuiltinOptionSpec]) -> ParsedOptions {
    let mut parsed = ParsedOptions::default();
    let mut index = 0usize;

    while let Some(token) = tokens.get(index) {
        if parsed.after_double_dash {
            parsed.positionals.push(token.clone());
            index += 1;
            continue;
        }
        if token == "--" {
            parsed.after_double_dash = true;
            index += 1;
            continue;
        }
        if let Some(option) = find_option(token, options) {
            if option_consumes_inline_value(token, option) {
                index += 1;
                continue;
            }
            if let Some(kind) = option.value {
                if let Some(value) = tokens.get(index + 1)
                    && value != "--"
                {
                    index += 2;
                    continue;
                }
                parsed.expected_value = Some(kind);
                break;
            }
            index += 1;
            continue;
        }
        parsed.positionals.push(token.clone());
        index += 1;
    }

    parsed
}

fn find_option<'a>(token: &str, options: &'a [BuiltinOptionSpec]) -> Option<&'a BuiltinOptionSpec> {
    let exact = options.iter().find(|option| option.names.contains(&token));
    if exact.is_some() {
        return exact;
    }
    options
        .iter()
        .find(|option| option.value.is_some() && option_consumes_inline_value(token, option))
}

fn option_consumes_inline_value(token: &str, option: &BuiltinOptionSpec) -> bool {
    option.value.is_some()
        && option.names.iter().any(|name| {
            token
                .strip_prefix(&format!("{name}="))
                .is_some_and(|value| !value.is_empty())
                || (name.starts_with('-')
                    && !name.starts_with("--")
                    && token.starts_with(name)
                    && token.len() > name.len())
        })
}
