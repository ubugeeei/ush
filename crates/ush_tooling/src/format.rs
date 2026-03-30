pub fn format_source(source: &str) -> String {
    let mut out = String::new();
    let mut indent = 0usize;
    let mut last_blank = false;

    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() {
            if !last_blank && !out.is_empty() {
                out.push('\n');
            }
            last_blank = true;
            continue;
        }

        let dedent = usize::from(line.starts_with('}'));
        indent = indent.saturating_sub(dedent);
        out.push_str(&"  ".repeat(indent));
        out.push_str(&normalize_line(line));
        out.push('\n');
        indent += brace_delta(line);
        last_blank = false;
    }

    out
}

fn normalize_line(line: &str) -> String {
    if let Some((left, right)) = line.split_once("=>") {
        return format!("{} => {}", left.trim(), right.trim());
    }
    if let Some((left, right)) = assignment_parts(line) {
        return format!("{left} = {right}");
    }
    if let Some((left, right)) = line.split_once("->") {
        return format!("{} -> {}", left.trim_end(), right.trim_start());
    }
    line.to_string()
}

fn assignment_parts(line: &str) -> Option<(String, String)> {
    let (left, right) = line.split_once('=')?;
    let head = left.trim_start();
    if !matches!(head.split_whitespace().next(), Some("let" | "alias")) {
        return None;
    }
    Some((left.trim().to_string(), right.trim().to_string()))
}

fn brace_delta(line: &str) -> usize {
    let mut single = false;
    let mut double = false;
    let mut delta = 0isize;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '#' if !single && !double => break,
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '\\' if double => {
                let _ = chars.next();
            }
            '{' if !single && !double => delta += 1,
            '}' if !single && !double => delta -= 1,
            _ => {}
        }
    }

    delta.max(0) as usize
}

#[cfg(test)]
mod tests {
    use super::format_source;

    #[test]
    fn formats_indentation_and_spacing() {
        let formatted = format_source(
            "fn greet(name: String)->String {\nprint name\nmatch name {\n\"x\"=>print \"ok\"\n}\n}\n",
        );

        assert_eq!(
            formatted,
            concat!(
                "fn greet(name: String) -> String {\n",
                "  print name\n",
                "  match name {\n",
                "    \"x\" => print \"ok\"\n",
                "  }\n",
                "}\n"
            )
        );
    }

    #[test]
    fn collapses_repeated_blank_lines() {
        let formatted = format_source("print \"a\"\n\n\nprint \"b\"\n");
        assert_eq!(formatted, "print \"a\"\n\nprint \"b\"\n");
    }
}
