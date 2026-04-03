mod fallback;

use std::collections::BTreeMap;

use anyhow::{Result, bail};

use crate::commands;
use crate::helpers::HelperInvocation;
use fallback::needs_posix_fallback;

#[derive(Debug, Clone)]
pub enum ParsedLine {
    Empty,
    Background(String),
    Fallback(String),
    Pipeline(Pipeline),
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub raw: String,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone)]
pub enum Stage {
    Builtin(CommandSpec),
    External(CommandSpec),
    Helper(HelperInvocation),
    Assignments(Vec<(String, String)>),
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub raw: String,
    pub command: String,
    pub args: Vec<String>,
    pub assignments: Vec<(String, String)>,
}

pub fn parse_line(line: &str, aliases: &BTreeMap<String, String>) -> Result<ParsedLine> {
    let stripped = strip_comment(line).trim().to_string();
    if stripped.is_empty() {
        return Ok(ParsedLine::Empty);
    }

    if let Some(background) = split_background_job(&stripped) {
        return Ok(ParsedLine::Background(background));
    }

    if needs_posix_fallback(&stripped) {
        return Ok(ParsedLine::Fallback(stripped));
    }

    let mut stages = Vec::new();
    for raw_stage in split_unquoted(&stripped, '|')? {
        let expanded = expand_alias(raw_stage.trim(), aliases)?;
        if let Some(helper) = HelperInvocation::parse(&expanded) {
            stages.push(Stage::Helper(helper?));
            continue;
        }

        let tokens = shell_words::split(&expanded)?;
        if tokens.is_empty() {
            continue;
        }

        let (assignments, rest) = split_assignments(tokens);
        if rest.is_empty() {
            stages.push(Stage::Assignments(assignments));
            continue;
        }

        let command = rest[0].clone();
        let args = rest[1..].to_vec();
        let spec = CommandSpec {
            raw: expanded,
            command: command.clone(),
            args,
            assignments,
        };

        if commands::is_builtin(&command) {
            stages.push(Stage::Builtin(spec));
        } else {
            stages.push(Stage::External(spec));
        }
    }

    if stages.is_empty() {
        return Ok(ParsedLine::Empty);
    }

    Ok(ParsedLine::Pipeline(Pipeline {
        raw: stripped,
        stages,
    }))
}

fn split_assignments(tokens: Vec<String>) -> (Vec<(String, String)>, Vec<String>) {
    let mut assignments = Vec::new();
    let mut rest = Vec::new();
    let mut assigning = true;

    for token in tokens {
        if assigning && is_assignment(&token) {
            if let Some((name, value)) = token.split_once('=') {
                assignments.push((name.to_string(), value.to_string()));
            }
            continue;
        }

        assigning = false;
        rest.push(token);
    }

    (assignments, rest)
}

fn is_assignment(token: &str) -> bool {
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    is_identifier(name)
}

fn expand_alias(stage: &str, aliases: &BTreeMap<String, String>) -> Result<String> {
    let mut current = stage.to_string();
    for _ in 0..8 {
        let Some(target) = alias_expansion_target(&current)? else {
            return Ok(current);
        };
        let Some(alias) = aliases.get(&target.word) else {
            return Ok(current);
        };
        current = format!(
            "{}{}{}",
            &current[..target.start],
            alias,
            &current[target.end..]
        );
    }
    Ok(current)
}

#[derive(Debug)]
struct AliasExpansionTarget {
    start: usize,
    end: usize,
    word: String,
}

#[derive(Clone, Copy, Debug)]
struct ShellWordSpan {
    start: usize,
    end: usize,
    quoted: bool,
}

fn alias_expansion_target(stage: &str) -> Result<Option<AliasExpansionTarget>> {
    let mut cursor = skip_whitespace(stage, 0);
    while let Some(span) = next_shell_word_span(stage, cursor)? {
        let raw = &stage[span.start..span.end];
        let word = parse_single_shell_word(raw)?;
        if is_assignment(&word) {
            cursor = skip_whitespace(stage, span.end);
            continue;
        }
        if span.quoted {
            return Ok(None);
        }
        return Ok(Some(AliasExpansionTarget {
            start: span.start,
            end: span.end,
            word,
        }));
    }
    Ok(None)
}

fn skip_whitespace(source: &str, from: usize) -> usize {
    source[from..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(from + offset))
        .unwrap_or(source.len())
}

fn next_shell_word_span(source: &str, from: usize) -> Result<Option<ShellWordSpan>> {
    let start = skip_whitespace(source, from);
    if start >= source.len() {
        return Ok(None);
    }

    let mut single = false;
    let mut double = false;
    let mut escaped = false;
    let mut quoted = false;

    for (offset, ch) in source[start..].char_indices() {
        let index = start + offset;
        match ch {
            '\\' if !single => {
                escaped = !escaped;
                quoted = true;
            }
            '\'' if !double && !escaped => {
                single = !single;
                escaped = false;
                quoted = true;
            }
            '"' if !single && !escaped => {
                double = !double;
                escaped = false;
                quoted = true;
            }
            _ if ch.is_whitespace() && !single && !double && !escaped => {
                return Ok(Some(ShellWordSpan {
                    start,
                    end: index,
                    quoted,
                }));
            }
            _ => escaped = false,
        }
    }

    if single || double {
        bail!("unterminated quoted string");
    }

    Ok(Some(ShellWordSpan {
        start,
        end: source.len(),
        quoted,
    }))
}

fn parse_single_shell_word(raw: &str) -> Result<String> {
    let mut tokens = shell_words::split(raw)?;
    match tokens.len() {
        1 => Ok(tokens.remove(0)),
        _ => bail!("invalid shell word: {raw}"),
    }
}

fn strip_comment(line: &str) -> String {
    let mut single = false;
    let mut double = false;
    for (index, ch) in line.char_indices() {
        match ch {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '#' if !single
                && !double
                && (index == 0 || line[..index].ends_with(char::is_whitespace)) =>
            {
                return line[..index].to_string();
            }
            _ => {}
        }
    }
    line.to_string()
}

fn split_unquoted(source: &str, separator: char) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut start = 0usize;
    let mut single = false;
    let mut double = false;
    let mut escaped = false;

    for (index, ch) in source.char_indices() {
        match ch {
            '\\' if !single => escaped = !escaped,
            '\'' if !double && !escaped => single = !single,
            '"' if !single && !escaped => double = !double,
            _ if ch == separator && !single && !double && !escaped => {
                result.push(source[start..index].trim().to_string());
                start = index + ch.len_utf8();
            }
            _ => escaped = false,
        }
    }

    if single || double {
        bail!("unterminated quoted string");
    }

    result.push(source[start..].trim().to_string());
    Ok(result)
}

fn is_identifier(source: &str) -> bool {
    let mut chars = source.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn split_background_job(line: &str) -> Option<String> {
    let mut single = false;
    let mut double = false;
    let mut escaped = false;
    let mut background_index = None;

    for (index, ch) in line.char_indices() {
        match ch {
            '\\' if !single => escaped = !escaped,
            '\'' if !double && !escaped => {
                single = !single;
                background_index = None;
                escaped = false;
            }
            '"' if !single && !escaped => {
                double = !double;
                background_index = None;
                escaped = false;
            }
            _ if single || double => escaped = false,
            '&' => {
                background_index = Some(index);
                escaped = false;
            }
            _ if ch.is_whitespace() => escaped = false,
            _ => {
                background_index = None;
                escaped = false;
            }
        }
    }

    let index = background_index?;
    let command = line[..index].trim_end();
    if command.is_empty() {
        return None;
    }
    if command.ends_with('&') {
        return None;
    }
    Some(command.to_string())
}

pub fn is_builtin(command: &str) -> bool {
    commands::is_builtin(command)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{ParsedLine, Stage, parse_line};

    #[test]
    fn parses_trailing_background_jobs_before_fallback() {
        let parsed = parse_line("sleep 1 &", &BTreeMap::new()).expect("parse");

        match parsed {
            ParsedLine::Background(source) => assert_eq!(source, "sleep 1"),
            other => panic!("expected background line, got {other:?}"),
        }
    }

    #[test]
    fn keeps_boolean_and_as_posix_fallback() {
        let parsed = parse_line("true && false", &BTreeMap::new()).expect("parse");

        match parsed {
            ParsedLine::Fallback(source) => assert_eq!(source, "true && false"),
            other => panic!("expected fallback line, got {other:?}"),
        }
    }

    #[test]
    fn alias_expansion_preserves_quoted_suffix_arguments() {
        let aliases = BTreeMap::from([("gm".to_string(), "git commit -m".to_string())]);
        let parsed = parse_line("gm 'simplify readme'", &aliases).expect("parse");

        match parsed {
            ParsedLine::Pipeline(pipeline) => match &pipeline.stages[0] {
                Stage::External(spec) => {
                    assert_eq!(spec.raw, "git commit -m 'simplify readme'");
                    assert_eq!(spec.command, "git");
                    assert_eq!(
                        spec.args,
                        vec![
                            "commit".to_string(),
                            "-m".to_string(),
                            "simplify readme".to_string()
                        ]
                    );
                }
                other => panic!("expected external stage, got {other:?}"),
            },
            other => panic!("expected pipeline, got {other:?}"),
        }
    }

    #[test]
    fn alias_expansion_runs_after_leading_assignments() {
        let aliases = BTreeMap::from([("gm".to_string(), "git commit -m".to_string())]);
        let parsed = parse_line("EDITOR=vim gm 'simplify readme'", &aliases).expect("parse");

        match parsed {
            ParsedLine::Pipeline(pipeline) => match &pipeline.stages[0] {
                Stage::External(spec) => {
                    assert_eq!(spec.raw, "EDITOR=vim git commit -m 'simplify readme'");
                    assert_eq!(spec.assignments, vec![("EDITOR".to_string(), "vim".to_string())]);
                    assert_eq!(spec.command, "git");
                    assert_eq!(
                        spec.args,
                        vec![
                            "commit".to_string(),
                            "-m".to_string(),
                            "simplify readme".to_string()
                        ]
                    );
                }
                other => panic!("expected external stage, got {other:?}"),
            },
            other => panic!("expected pipeline, got {other:?}"),
        }
    }

    #[test]
    fn quoted_command_word_does_not_trigger_alias_expansion() {
        let aliases = BTreeMap::from([("gm".to_string(), "git commit -m".to_string())]);
        let parsed = parse_line("'gm' 'simplify readme'", &aliases).expect("parse");

        match parsed {
            ParsedLine::Pipeline(pipeline) => match &pipeline.stages[0] {
                Stage::External(spec) => {
                    assert_eq!(spec.raw, "'gm' 'simplify readme'");
                    assert_eq!(spec.command, "gm");
                    assert_eq!(spec.args, vec!["simplify readme".to_string()]);
                }
                other => panic!("expected external stage, got {other:?}"),
            },
            other => panic!("expected pipeline, got {other:?}"),
        }
    }
}
