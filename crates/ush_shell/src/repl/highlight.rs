use crate::parser::is_builtin;
use rustyline::CompletionType;

use super::{UshHelper, display, syntax};

const COMMAND: &str = "\u{1b}[1;38;5;111m";
const KEYWORD: &str = "\u{1b}[1;38;5;180m";
const VARIABLE: &str = "\u{1b}[1;38;5;151m";
const STRING: &str = "\u{1b}[38;5;114m";
const ASSIGNMENT: &str = "\u{1b}[38;5;145m";
const OPERATOR: &str = "\u{1b}[38;5;109m";
const COMMENT: &str = "\u{1b}[2;38;5;244m";
const SELECTION: &str = "\u{1b}[48;5;239;38;5;255m";
const ACTIVE_CANDIDATE: &str = "\u{1b}[1;48;5;111;38;5;235m";
const ACTIVE_CANDIDATE_DETAIL: &str = "\u{1b}[48;5;111;38;5;236m";
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
            for next in chars.by_ref() {
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

pub fn highlight_candidate(
    helper: &UshHelper,
    candidate: &str,
    completion: CompletionType,
    active: bool,
) -> String {
    let (name, detail) = display::split(candidate);
    let mut styled = if active && matches!(completion, CompletionType::Circular) {
        color(ACTIVE_CANDIDATE, name)
    } else if helper.commands.contains(name) || is_builtin(name) {
        color(COMMAND, name)
    } else if syntax::is_keyword(name) {
        color(KEYWORD, name)
    } else {
        name.to_string()
    };

    if let Some(detail) = detail {
        push_detail(
            &mut styled,
            detail,
            active && matches!(completion, CompletionType::Circular),
        );
    }

    styled
}

fn push_detail(out: &mut String, detail: &str, active: bool) {
    out.push_str(if active {
        ACTIVE_CANDIDATE_DETAIL
    } else {
        HINT
    });
    out.push_str(display::DETAIL_SEPARATOR);
    out.push_str(detail);
    out.push_str(RESET);
}

pub fn highlight_prompt(prompt: &str) -> String {
    let Some((body, marker_color)) = parse_prompt_body(prompt) else {
        return color(PROMPT_FALLBACK, prompt);
    };
    if let Some(path) = body.strip_prefix("ush ") {
        return format!(
            "{PROMPT_NAME}ush{RESET} {PROMPT_PATH}{path}{RESET} {marker_color}{}{RESET} ",
            if marker_color == PROMPT_OK { "$" } else { "!" }
        );
    }
    if let Some((head, tail)) = body.rsplit_once('\n') {
        let marker = if marker_color == PROMPT_OK { "$" } else { "!" };
        let head = if tail.is_empty() { head } else { body };
        return format!("{PROMPT_PATH}{head}{RESET}\n{marker_color}{marker}{RESET} ");
    }
    format!(
        "{PROMPT_PATH}{body}{RESET} {marker_color}{}{RESET} ",
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

fn parse_prompt_body(prompt: &str) -> Option<(&str, &str)> {
    if let Some(body) = prompt.strip_suffix(" $ ") {
        return Some((body, PROMPT_OK));
    }
    if let Some(body) = prompt.strip_suffix(" ! ") {
        return Some((body, PROMPT_ERR));
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::repl::selection::SelectionMove;
    use crate::repl::{
        UshHelper,
        highlight::{highlight_hint, highlight_line, highlight_prompt},
    };
    use rustyline::CompletionType;

    #[test]
    fn highlights_commands_variables_and_comments() {
        let helper = UshHelper::new(
            vec!["echo".to_string(), "grep".to_string()],
            vec!["PATH".to_string()],
            std::env::temp_dir(),
        );
        let line = highlight_line(&helper, "echo $PATH # note");

        assert!(line.contains("\u{1b}[1;38;5;111mecho\u{1b}[0m"));
        assert!(line.contains("\u{1b}[1;38;5;151m$PATH\u{1b}[0m"));
        assert!(line.contains("\u{1b}[2;38;5;244m# note\u{1b}[0m"));
    }

    #[test]
    fn highlights_default_prompt_by_segment() {
        let prompt = highlight_prompt("~/.../ubugeeei/ush $ ");

        assert!(prompt.contains("\u{1b}[1;38;5;223m~/.../ubugeeei/ush\u{1b}[0m"));
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
        let helper = UshHelper::new(vec!["echo".to_string()], vec![], std::env::temp_dir());
        helper
            .selection_handle()
            .extend("echo hello", 5, SelectionMove::WordRight);

        let line = highlight_line(&helper, "echo hello");

        assert!(line.contains("\u{1b}[48;5;239;38;5;255mhello\u{1b}[0m"));
    }

    #[test]
    fn highlights_active_circular_candidate() {
        let helper = UshHelper::new(vec!["echo".to_string()], vec![], std::env::temp_dir());
        let candidate = super::highlight_candidate(&helper, "echo", CompletionType::Circular, true);

        assert!(candidate.contains("\u{1b}[1;48;5;111;38;5;235m"));
    }
}
