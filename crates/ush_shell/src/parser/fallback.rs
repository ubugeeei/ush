pub(super) fn needs_posix_fallback(line: &str) -> bool {
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
            '>' if line.get(index.saturating_sub(1)..index + 1) != Some("->") => return true,
            '$' if matches!(chars.peek(), Some((_, '('))) => return true,
            _ => {}
        }
    }

    line.contains("&&")
        || line.contains("||")
        || contains_unquoted_keyword(line, "if")
        || contains_unquoted_keyword(line, "elif")
        || contains_unquoted_keyword(line, "else")
        || contains_unquoted_keyword(line, "for")
        || contains_unquoted_keyword(line, "while")
        || contains_unquoted_keyword(line, "until")
        || contains_unquoted_keyword(line, "case")
        || contains_unquoted_keyword(line, "do")
        || contains_unquoted_keyword(line, "done")
        || contains_unquoted_keyword(line, "then")
        || contains_unquoted_keyword(line, "fi")
        || contains_unquoted_keyword(line, "esac")
}

fn contains_unquoted_keyword(line: &str, keyword: &str) -> bool {
    let mut single = false;
    let mut double = false;
    let mut escaped = false;
    let mut token = String::new();

    for ch in line.chars() {
        match ch {
            '\\' if !single => {
                escaped = !escaped;
                if !token.is_empty() && !escaped {
                    if token == keyword {
                        return true;
                    }
                    token.clear();
                }
            }
            '\'' if !double && !escaped => {
                if !token.is_empty() {
                    if token == keyword {
                        return true;
                    }
                    token.clear();
                }
                single = !single;
            }
            '"' if !single && !escaped => {
                if !token.is_empty() {
                    if token == keyword {
                        return true;
                    }
                    token.clear();
                }
                double = !double;
            }
            _ if single || double => escaped = false,
            _ if ch == '_' || ch.is_ascii_alphanumeric() => {
                token.push(ch);
                escaped = false;
            }
            _ => {
                if token == keyword {
                    return true;
                }
                token.clear();
                escaped = false;
            }
        }
    }

    token == keyword
}
