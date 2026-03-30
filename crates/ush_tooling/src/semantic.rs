use self::KeywordContext::{Decl, Func, Let, NoKeyword};
use crate::token::{SemanticToken, SemanticTokenKind};

const KEYWORDS: &[&str] = &[
    "alias", "async", "enum", "fn", "impl", "let", "match", "print", "return", "shell", "trait",
    "type",
];

#[derive(Clone, Copy)]
enum KeywordContext {
    NoKeyword,
    Decl,
    Func,
    Let,
}

pub fn semantic_tokens(source: &str) -> Vec<SemanticToken> {
    let mut out = Vec::new();
    for (line_no, line) in source.lines().enumerate() {
        tokenize_line(line_no as u32, line, &mut out);
    }
    out
}

fn tokenize_line(line_no: u32, line: &str, out: &mut Vec<SemanticToken>) {
    let bytes = line.as_bytes();
    let mut index = 0usize;
    let mut context = NoKeyword;

    while index < bytes.len() {
        let ch = bytes[index] as char;
        if ch.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if line[index..].starts_with("#[") {
            let end = line[index..]
                .find(']')
                .map_or(line.len(), |offset| index + offset + 1);
            push(
                out,
                line_no,
                index,
                end - index,
                SemanticTokenKind::Decorator,
            );
            break;
        }
        if ch == '#' {
            push(
                out,
                line_no,
                index,
                line.len() - index,
                SemanticTokenKind::Comment,
            );
            break;
        }
        if matches!(ch, '"' | '\'') {
            let end = string_end(line, index);
            push(out, line_no, index, end - index, SemanticTokenKind::String);
            index = end;
            context = NoKeyword;
            continue;
        }
        if ch.is_ascii_digit() {
            let end = take_while(line, index, |value| value.is_ascii_digit());
            push(out, line_no, index, end - index, SemanticTokenKind::Number);
            index = end;
            context = NoKeyword;
            continue;
        }
        if ch == '$' {
            let end = variable_end(line, index);
            push(
                out,
                line_no,
                index,
                end - index,
                SemanticTokenKind::Variable,
            );
            index = end;
            context = NoKeyword;
            continue;
        }
        if is_ident_start(ch) {
            let end = take_while(line, index, is_ident);
            let ident = &line[index..end];
            let kind = classify_ident(ident, context, next_non_space(line, end));
            push(out, line_no, index, end - index, kind);
            context = match ident {
                "fn" => Func,
                "let" => Let,
                "enum" | "type" | "trait" | "impl" => Decl,
                _ => NoKeyword,
            };
            index = end;
            continue;
        }
        let end = operator_end(line, index);
        push(
            out,
            line_no,
            index,
            end - index,
            SemanticTokenKind::Operator,
        );
        index = end;
        context = NoKeyword;
    }
}

fn classify_ident(ident: &str, context: KeywordContext, next: Option<char>) -> SemanticTokenKind {
    if KEYWORDS.contains(&ident) {
        return SemanticTokenKind::Keyword;
    }
    if matches!(context, Func) || next == Some('(') {
        return SemanticTokenKind::Function;
    }
    if matches!(context, Decl) || ident.chars().next().is_some_and(char::is_uppercase) {
        return SemanticTokenKind::Type;
    }
    if next == Some(':') {
        return SemanticTokenKind::Property;
    }
    if matches!(context, Let) {
        return SemanticTokenKind::Variable;
    }
    SemanticTokenKind::Variable
}

fn push(
    out: &mut Vec<SemanticToken>,
    line: u32,
    start: usize,
    len: usize,
    kind: SemanticTokenKind,
) {
    out.push(SemanticToken {
        line,
        start: start as u32,
        length: len as u32,
        kind,
    });
}

fn string_end(line: &str, start: usize) -> usize {
    let quote = line.as_bytes()[start] as char;
    let mut escaped = false;
    for (offset, ch) in line[start + 1..].char_indices() {
        if quote == '"' && !escaped && ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote && !(quote == '"' && escaped) {
            return start + offset + 2;
        }
        escaped = false;
    }
    line.len()
}

fn variable_end(line: &str, start: usize) -> usize {
    if line[start..].starts_with("${") {
        return line[start..]
            .find('}')
            .map_or(line.len(), |offset| start + offset + 1);
    }
    take_while(line, start + 1, is_ident)
}

fn operator_end(line: &str, start: usize) -> usize {
    let pair = line.get(start..start + 2).unwrap_or("");
    if matches!(
        pair,
        "->" | "=>" | "::" | "==" | "!=" | "<=" | ">=" | "&&" | "||"
    ) {
        return start + 2;
    }
    start + 1
}

fn next_non_space(line: &str, start: usize) -> Option<char> {
    line[start..].chars().find(|ch| !ch.is_ascii_whitespace())
}

fn take_while(line: &str, start: usize, pred: impl Fn(char) -> bool) -> usize {
    let mut end = start;
    for (offset, ch) in line[start..].char_indices() {
        if !pred(ch) {
            break;
        }
        end = start + offset + ch.len_utf8();
    }
    end.max(start)
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use crate::{SemanticTokenKind, semantic_tokens};

    #[test]
    fn tokenizes_keywords_and_calls() {
        let tokens = semantic_tokens("fn greet(name: String) -> String {");

        assert_eq!(tokens[0].kind, SemanticTokenKind::Keyword);
        assert_eq!(tokens[1].kind, SemanticTokenKind::Function);
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == SemanticTokenKind::Type)
        );
    }

    #[test]
    fn tokenizes_comments_and_strings() {
        let tokens = semantic_tokens("print \"ok\" # note");

        assert!(
            tokens
                .iter()
                .any(|token| token.kind == SemanticTokenKind::String)
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == SemanticTokenKind::Comment)
        );
    }
}
