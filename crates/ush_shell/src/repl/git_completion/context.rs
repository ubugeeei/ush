use crate::repl::syntax;

use super::specs::{ArgKind, GIT_COMMANDS, GLOBAL_OPTIONS, GitCommandSpec, GitOptionSpec};

#[derive(Debug, Default)]
pub(super) struct ParsedOptions {
    pub(super) positionals: Vec<String>,
    pub(super) expected_value: Option<ArgKind>,
    pub(super) after_double_dash: bool,
}

#[derive(Debug)]
pub(super) struct GitContext {
    pub(super) start: usize,
    pub(super) word: String,
    pub(super) subcommand: Option<&'static GitCommandSpec>,
    pub(super) raw_subcommand: Option<String>,
    pub(super) global_expected_value: Option<ArgKind>,
    pub(super) subcommand_args: Vec<String>,
}

pub(super) fn parse_context(line: &str, pos: usize) -> Option<GitContext> {
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
    if segment.get(command_index).map(String::as_str) != Some("git") {
        return None;
    }

    let mut index = command_index + 1;
    let mut global_expected_value = None;
    while let Some(token) = segment.get(index) {
        if let Some(option) = find_option(token, GLOBAL_OPTIONS) {
            if option_consumes_inline_value(token, option) {
                index += 1;
                continue;
            }
            if let Some(kind) = option.value {
                if segment.get(index + 1).is_some() {
                    index += 2;
                } else {
                    global_expected_value = Some(kind);
                    index += 1;
                    break;
                }
            } else {
                index += 1;
            }
            continue;
        }
        break;
    }

    let raw_subcommand = segment.get(index).cloned();
    let subcommand = raw_subcommand
        .as_deref()
        .and_then(|name| GIT_COMMANDS.iter().find(|spec| spec.name == name));
    let subcommand_args = if subcommand.is_some() {
        segment[index + 1..].to_vec()
    } else {
        Vec::new()
    };

    Some(GitContext {
        start,
        word,
        subcommand,
        raw_subcommand,
        global_expected_value,
        subcommand_args,
    })
}

pub(super) fn parse_options(tokens: &[String], options: &[GitOptionSpec]) -> ParsedOptions {
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

fn find_option<'a>(token: &str, options: &'a [GitOptionSpec]) -> Option<&'a GitOptionSpec> {
    let exact = options.iter().find(|option| option.names.contains(&token));
    if exact.is_some() {
        return exact;
    }
    options
        .iter()
        .find(|option| option.value.is_some() && option_consumes_inline_value(token, option))
}

fn option_consumes_inline_value(token: &str, option: &GitOptionSpec) -> bool {
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
