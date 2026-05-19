//! LSP `textDocument/foldingRange` provider.
//!
//! Reports folding regions for matched `{ … }` blocks. Strings,
//! triple-strings, line comments (`# …`), and attribute brackets
//! (`#[…]`) are ignored so a brace inside a quoted string does not
//! get treated as a fold boundary.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoldingRange {
    /// Zero-based first line of the fold.
    pub start_line: u32,
    /// Zero-based last line of the fold (the closing brace's line).
    pub end_line: u32,
}

pub fn folding_ranges(source: &str) -> Vec<FoldingRange> {
    let mut stack: Vec<u32> = Vec::new();
    let mut ranges = Vec::new();
    let mut in_multiline_string = false;

    for (line_no, raw) in source.lines().enumerate() {
        let line_no = line_no as u32;
        let mut chars = raw.chars().peekable();
        let mut in_single = false;
        let mut in_double = false;
        let mut just_toggled_triple = false;

        while let Some(ch) = chars.next() {
            if in_multiline_string {
                // Stay inside the multiline until we see another `"""`.
                if ch == '"' && chars.peek() == Some(&'"') {
                    chars.next();
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        in_multiline_string = false;
                        just_toggled_triple = true;
                    }
                }
                continue;
            }
            if ch == '"' && !in_single {
                if chars.peek() == Some(&'"') {
                    let saved = chars.clone();
                    chars.next();
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        in_multiline_string = true;
                        just_toggled_triple = true;
                        continue;
                    }
                    // It was just `""` (empty string), restore.
                    chars = saved;
                }
                in_double = !in_double;
                continue;
            }
            if ch == '\'' && !in_double {
                in_single = !in_single;
                continue;
            }
            if in_single || in_double {
                continue;
            }
            if ch == '#' && !just_toggled_triple {
                if chars.peek() == Some(&'[') {
                    // `#[…]` attribute — skip up to the matching `]`.
                    chars.next();
                    let mut depth = 1isize;
                    for inner in chars.by_ref() {
                        match inner {
                            '[' => depth += 1,
                            ']' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    continue;
                }
                // Line comment: ignore the rest of the line.
                break;
            }
            if ch == '{' {
                stack.push(line_no);
            } else if ch == '}'
                && let Some(start_line) = stack.pop()
                && start_line < line_no
            {
                ranges.push(FoldingRange {
                    start_line,
                    end_line: line_no,
                });
            }
        }
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::folding_ranges;

    #[test]
    fn one_function_one_fold() {
        let source = "fn greet(name: String) -> String {\n  return name\n}\n";
        let ranges = folding_ranges(source);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);
    }

    #[test]
    fn nested_blocks() {
        let source = "fn a() {\n  match x {\n    Some(v) => print v,\n  }\n}\n";
        let ranges = folding_ranges(source);
        // Outer fn fold + inner match fold.
        assert_eq!(ranges.len(), 2);
    }

    #[test]
    fn braces_inside_strings_are_ignored() {
        let source = "let s = \"{ nope }\"\nfn f() {\n  print s\n}\n";
        let ranges = folding_ranges(source);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 1);
        assert_eq!(ranges[0].end_line, 3);
    }

    #[test]
    fn line_comment_braces_are_ignored() {
        let source = "fn f() {\n  # { not a brace }\n}\n";
        let ranges = folding_ranges(source);
        assert_eq!(ranges.len(), 1);
    }

    #[test]
    fn single_line_block_does_not_fold() {
        let source = "fn empty() {}\n";
        let ranges = folding_ranges(source);
        assert!(ranges.is_empty());
    }
}
