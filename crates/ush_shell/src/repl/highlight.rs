use crate::parser::is_builtin;

use super::{UshHelper, syntax};

const COMMAND: &str = "\u{1b}[1;38;5;111m";
const KEYWORD: &str = "\u{1b}[1;38;5;180m";
const VARIABLE: &str = "\u{1b}[1;38;5;151m";
const STRING: &str = "\u{1b}[38;5;114m";
const ASSIGNMENT: &str = "\u{1b}[38;5;145m";
const OPERATOR: &str = "\u{1b}[38;5;109m";
const COMMENT: &str = "\u{1b}[2;38;5;244m";
const SELECTION: &str = "\u{1b}[48;5;239;38;5;255m";
const PROMPT_NAME: &str = "\u{1b}[38;5;109m";
const PROMPT_PATH: &str = "\u{1b}[1;38;5;223m";
const PROMPT_OK: &str = "\u{1b}[1;38;5;150m";
const PROMPT_ERR: &str = "\u{1b}[1;38;5;203m";
const PROMPT_FALLBACK: &str = "\u{1b}[1;38;5;250m";
const HINT: &str = "\u{1b}[2;38;5;245m";
const RESET: &str = "\u{1b}[0m";

pub fn highlight_line(helper: &UshHelper, line: &str) -> String {
    if let Some((start, end)) = helper
        .selection_range()
        .filter(|(start, end)| *start < *end && *end <= line.len())
    {
        return format!(
            "{}{}{}",
            highlight_segments(helper, &line[..start]),
            color(SELECTION, &line[start..end]),
            highlight_segments(helper, &line[end..])
        );
    }
    highlight_segments(helper, line)
}

fn highlight_segments(helper: &UshHelper, line: &str) -> String {
    let mut result = String::new();
    let mut token = String::new();
    let mut chars = line.chars().peekable();
    let mut command_position = true;

    while let Some(ch) = chars.next() {
        if ch == '#' && token.is_empty() {
            result.push_str(COMMENT);
            result.push('#');
            result.extend(chars);
            result.push_str(RESET);
            break;
        }
        if ch.is_whitespace() {
            flush_token(helper, &mut result, &mut token, &mut command_position);
            result.push(ch);
            continue;
        }
        if matches!(ch, '|' | '&' | ';' | '(' | ')' | '{' | '}' | '<' | '>') {
            flush_token(helper, &mut result, &mut token, &mut command_position);
            let mut op = ch.to_string();
            if matches!(ch, '|' | '&' | '<' | '>') && chars.peek().copied() == Some(ch) {
                op.push(chars.next().unwrap_or(ch));
            }
            result.push_str(OPERATOR);
            result.push_str(&op);
            result.push_str(RESET);
            command_position = matches!(op.as_str(), "|" | "||" | "&&" | ";" | "&");
            continue;
        }
        token.push(ch);
        if matches!(ch, '\'' | '"') {
            let quote = ch;
            let mut escaped = false;
            while let Some(next) = chars.next() {
                token.push(next);
                if quote == '"' && !escaped && next == '\\' {
                    escaped = true;
                    continue;
                }
                if next == quote && !(quote == '"' && escaped) {
                    break;
                }
                escaped = false;
            }
        }
    }

    flush_token(helper, &mut result, &mut token, &mut command_position);
    result
}

pub fn highlight_candidate(helper: &UshHelper, candidate: &str) -> String {
    if helper.commands.contains(candidate) || is_builtin(candidate) {
        return color(COMMAND, candidate);
    }
    if syntax::is_keyword(candidate) {
        return color(KEYWORD, candidate);
    }
    candidate.to_string()
}

pub fn highlight_prompt(prompt: &str) -> String {
    let Some(path) = prompt.strip_prefix("ush ") else {
        return color(PROMPT_FALLBACK, prompt);
    };
    let (path, marker_color) = if let Some(path) = path.strip_suffix(" $ ") {
        (path, PROMPT_OK)
    } else if let Some(path) = path.strip_suffix(" ! ") {
        (path, PROMPT_ERR)
    } else {
        return color(PROMPT_FALLBACK, prompt);
    };

    format!(
        "{PROMPT_NAME}ush{RESET} {PROMPT_PATH}{path}{RESET} {marker_color}{}{RESET} ",
        if marker_color == PROMPT_OK { "$" } else { "!" }
    )
}

pub fn highlight_hint(hint: &str) -> String {
    color(HINT, hint)
}

fn flush_token(
    helper: &UshHelper,
    result: &mut String,
    token: &mut String,
    command_position: &mut bool,
) {
    if token.is_empty() {
        return;
    }
    let styled = style_token(helper, token, *command_position);
    result.push_str(&styled);
    if !syntax::is_assignment(token) {
        *command_position = false;
    }
    token.clear();
}

fn style_token(helper: &UshHelper, token: &str, command_position: bool) -> String {
    if command_position
        && (helper.commands.contains(token) || is_builtin(token) || token.contains('/'))
    {
        return color(COMMAND, token);
    }
    if syntax::is_keyword(token) {
        return color(KEYWORD, token);
    }
    if syntax::is_assignment(token) {
        return color(ASSIGNMENT, token);
    }
    if token.starts_with('$') {
        return color(VARIABLE, token);
    }
    if (token.starts_with('"') && token.ends_with('"'))
        || (token.starts_with('\'') && token.ends_with('\''))
    {
        return color(STRING, token);
    }
    token.to_string()
}

fn color(code: &str, value: &str) -> String {
    format!("{code}{value}{RESET}")
}

#[cfg(test)]
mod tests {
    use crate::repl::selection::SelectionMove;
    use crate::repl::{
        UshHelper,
        highlight::{highlight_hint, highlight_line, highlight_prompt},
    };

    #[test]
    fn highlights_commands_variables_and_comments() {
        let helper = UshHelper::new(
            vec!["echo".to_string(), "grep".to_string()],
            vec!["PATH".to_string()],
        );
        let line = highlight_line(&helper, "echo $PATH # note");

        assert!(line.contains("\u{1b}[1;38;5;111mecho\u{1b}[0m"));
        assert!(line.contains("\u{1b}[1;38;5;151m$PATH\u{1b}[0m"));
        assert!(line.contains("\u{1b}[2;38;5;244m# note\u{1b}[0m"));
    }

    #[test]
    fn highlights_default_prompt_by_segment() {
        let prompt = highlight_prompt("ush ~/.../ubugeeei/ubshell $ ");

        assert!(prompt.contains("\u{1b}[38;5;109mush\u{1b}[0m"));
        assert!(prompt.contains("\u{1b}[1;38;5;223m~/.../ubugeeei/ubshell\u{1b}[0m"));
        assert!(prompt.contains("\u{1b}[1;38;5;150m$\u{1b}[0m"));
    }

    #[test]
    fn uses_soft_hint_color() {
        assert_eq!(
            highlight_hint("suffix"),
            "\u{1b}[2;38;5;245msuffix\u{1b}[0m"
        );
    }

    #[test]
    fn highlights_selected_region() {
        let helper = UshHelper::new(vec!["echo".to_string()], vec![]);
        helper
            .selection_handle()
            .extend("echo hello", 5, SelectionMove::WordRight);

        let line = highlight_line(&helper, "echo hello");

        assert!(line.contains("\u{1b}[48;5;239;38;5;255mhello\u{1b}[0m"));
    }
}
