pub(super) fn render_starship_format(
    format: &str,
    directory: &str,
    character: &str,
    git_branch: Option<&str>,
) -> String {
    let mut out = String::new();
    let chars = format.chars().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '$' {
            out.push(chars[index]);
            index += 1;
            continue;
        }
        let rest = chars[index..].iter().collect::<String>();
        if rest.starts_with("$directory") {
            out.push_str(directory);
            index += "$directory".len();
            continue;
        }
        if rest.starts_with("$character") {
            out.push_str(character);
            index += "$character".len();
            continue;
        }
        if rest.starts_with("$line_break") {
            out.push('\n');
            index += "$line_break".len();
            continue;
        }
        if rest.starts_with("$git_branch") {
            if let Some(git_branch) = git_branch {
                out.push_str(git_branch);
            }
            index += "$git_branch".len();
            continue;
        }
        if chars
            .get(index + 1)
            .copied()
            .is_some_and(is_starship_module_char)
        {
            index += 1;
            while chars
                .get(index)
                .copied()
                .is_some_and(is_starship_module_char)
            {
                index += 1;
            }
            continue;
        }
        out.push('$');
        index += 1;
    }

    out
}

fn is_starship_module_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

pub(super) fn strip_starship_markup(input: &str) -> String {
    let chars = input.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '['
            && let Some((inner, end)) = starship_style_group(&chars, index)
        {
            out.push_str(&unescape_starship_text(&inner));
            index = end;
            continue;
        }
        if chars[index] == '\\'
            && let Some(next) = chars.get(index + 1)
        {
            out.push(*next);
            index += 2;
            continue;
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

fn starship_style_group(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut inner = String::new();
    let mut cursor = start + 1;
    while cursor < chars.len() {
        if chars[cursor] == '\\'
            && let Some(next) = chars.get(cursor + 1)
        {
            inner.push(chars[cursor]);
            inner.push(*next);
            cursor += 2;
            continue;
        }
        if chars[cursor] == ']' && chars.get(cursor + 1).copied() == Some('(') {
            let mut style_cursor = cursor + 2;
            while style_cursor < chars.len() {
                if chars[style_cursor] == ')' {
                    return Some((inner, style_cursor + 1));
                }
                style_cursor += 1;
            }
            return None;
        }
        inner.push(chars[cursor]);
        cursor += 1;
    }
    None
}

fn unescape_starship_text(raw: &str) -> String {
    let mut out = String::new();
    let mut escaped = false;
    for ch in raw.chars() {
        if escaped {
            out.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    if escaped {
        out.push('\\');
    }
    out
}

pub(super) fn ensure_prompt_suffix(mut prompt: String) -> String {
    if prompt.ends_with('\n') || prompt.ends_with(' ') {
        return prompt;
    }
    prompt.push(' ');
    prompt
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_starship_format_tokens() {
        let prompt = super::render_starship_format(
            "$directory$git_branch$line_break$character",
            "~/repo",
            "$ ",
            Some(" |- main"),
        );

        assert_eq!(prompt, "~/repo |- main\n$ ");
    }

    #[test]
    fn strips_starship_markup_and_escapes() {
        assert_eq!(
            super::strip_starship_markup(r" [\(main\)](cyan)"),
            " (main)"
        );
    }
}
