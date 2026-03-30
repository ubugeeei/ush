use std::collections::BTreeMap;

use anyhow::{Result, bail};

use crate::helpers::HelperInvocation;

#[derive(Debug, Clone)]
pub enum ParsedLine {
    Empty,
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

        if is_builtin(&command) {
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
        let tokens = shell_words::split(&current)?;
        let Some(first) = tokens.first() else {
            return Ok(current);
        };
        let Some(alias) = aliases.get(first) else {
            return Ok(current);
        };
        let suffix = if tokens.len() > 1 {
            format!(" {}", tokens[1..].join(" "))
        } else {
            String::new()
        };
        current = format!("{alias}{suffix}");
    }
    Ok(current)
}

fn needs_posix_fallback(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with('!') || trimmed.starts_with('(') || trimmed.starts_with('{') {
        return true;
    }

    let mut chars = line.char_indices().peekable();
    let mut single = false;
    let mut double = false;

    while let Some((index, ch)) = chars.next() {
        match ch {
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            _ if single || double => {}
            ';' | '`' | '&' | '<' => return true,
            '>' => {
                if line.get(index.saturating_sub(1)..index + 1) != Some("->") {
                    return true;
                }
            }
            '$' if matches!(chars.peek(), Some((_, '('))) => return true,
            _ => {}
        }
    }

    [
        "&&", "||", "if ", "elif ", "else", "for ", "while ", "until ", "case ", "do ", "done",
        "then", "fi", "esac",
    ]
    .into_iter()
    .any(|pattern| line.contains(pattern))
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

pub fn is_builtin(command: &str) -> bool {
    matches!(
        command,
        "cd" | "pwd"
            | "exit"
            | "alias"
            | "unalias"
            | "history"
            | "export"
            | "help"
            | "source"
            | "rm"
    )
}
