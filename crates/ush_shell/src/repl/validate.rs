use rustyline::validate::ValidationResult;

use super::syntax;

pub fn validate_input(input: &str) -> ValidationResult {
    if syntax::has_unclosed_quotes(input) || syntax::has_trailing_escape(input) {
        return ValidationResult::Incomplete;
    }

    let tokens = syntax::tokenize(input);
    if tokens.is_empty() {
        return ValidationResult::Valid(None);
    }
    if ends_with_continuation(&tokens) || has_open_blocks(&tokens) {
        return ValidationResult::Incomplete;
    }

    ValidationResult::Valid(None)
}

fn ends_with_continuation(tokens: &[String]) -> bool {
    matches!(
        tokens.last().map(String::as_str),
        Some("|" | "||" | "&&" | "\\" | "do" | "then" | "else" | "elif" | "in")
    )
}

fn has_open_blocks(tokens: &[String]) -> bool {
    let mut stack = Vec::new();
    for token in tokens {
        match token.as_str() {
            "if" => stack.push("fi"),
            "case" => stack.push("esac"),
            "do" => stack.push("done"),
            "{" => stack.push("}"),
            "(" => stack.push(")"),
            "fi" | "done" | "esac" | "}" | ")" => {
                if stack.pop() != Some(token.as_str()) {
                    return false;
                }
            }
            _ => {}
        }
    }
    !stack.is_empty()
}

#[cfg(test)]
mod tests {
    use rustyline::validate::ValidationResult;

    use crate::repl::validate::validate_input;

    #[test]
    fn keeps_multiline_posix_constructs_open() {
        assert!(matches!(
            validate_input("echo hi |"),
            ValidationResult::Incomplete
        ));
        assert!(matches!(
            validate_input("if true"),
            ValidationResult::Incomplete
        ));
        assert!(matches!(
            validate_input("if true\nthen"),
            ValidationResult::Incomplete
        ));
    }

    #[test]
    fn accepts_complete_shell_lines() {
        assert!(matches!(
            validate_input("if true\nthen echo ok\nfi"),
            ValidationResult::Valid(_)
        ));
        assert!(matches!(
            validate_input("echo ok"),
            ValidationResult::Valid(_)
        ));
    }
}
