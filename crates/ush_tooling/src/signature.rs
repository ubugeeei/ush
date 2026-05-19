//! LSP `textDocument/signatureHelp` provider.
//!
//! Scans the source for `fn name(parameters)` declarations and,
//! when the cursor is inside a call `name(arg, arg, …)`, returns
//! the matching signature with the currently-active parameter
//! index.
//!
//! Scope:
//!
//! - Only `fn` declarations whose opening `(` and matching `)` are
//!   on the same line are recognised. Multi-line signatures and
//!   `bin(...)` magic functions can land later.
//! - There is no scope resolution: if a name shadows itself, the
//!   first declaration found wins. The active-parameter counter is
//!   tracked by simple comma counting at depth-1.
//! - When the cursor is inside a call to a function we have not
//!   declared in this file (e.g. a stdlib helper), the provider
//!   returns `None` and the editor falls back to showing nothing.

/// One `fn name(p1: T1, p2: T2)` declaration extracted from source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    pub name: String,
    /// Each parameter's label, e.g. `"name: String"`. Empty when the
    /// function takes no arguments.
    pub parameters: Vec<String>,
    /// Full signature label without the leading `fn ` keyword,
    /// e.g. `"greet(name: String) -> String"`. Editors render this
    /// at the top of the popup.
    pub label: String,
}

/// What `textDocument/signatureHelp` should return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureHelp {
    pub signature: FunctionSignature,
    pub active_parameter: u32,
}

/// Walk the source line by line, collecting every `fn name(...)`
/// declaration that closes on the same line.
pub fn function_signatures(source: &str) -> Vec<FunctionSignature> {
    let mut out = Vec::new();
    for line in source.lines() {
        if let Some(sig) = parse_fn_line(line) {
            out.push(sig);
        }
    }
    out
}

/// Resolve which signature (if any) applies to the cursor.
pub fn signature_help(source: &str, line: u32, character: u32) -> Option<SignatureHelp> {
    let prefix = prefix_up_to_cursor(source, line, character)?;
    let (call_name, active) = call_at_end_of(&prefix)?;
    let sig = function_signatures(source)
        .into_iter()
        .find(|s| s.name == call_name)?;
    let bounded = active.min(sig.parameters.len().saturating_sub(1) as u32);
    Some(SignatureHelp {
        signature: sig,
        active_parameter: bounded,
    })
}

/// Return everything up to (but not including) the character offset
/// on the requested line. Used so we can do a simple right-to-left
/// scan to find the enclosing call.
fn prefix_up_to_cursor(source: &str, line: u32, character: u32) -> Option<String> {
    let mut out = String::new();
    for (idx, src_line) in source.lines().enumerate() {
        match (idx as u32).cmp(&line) {
            std::cmp::Ordering::Less => {
                out.push_str(src_line);
                out.push('\n');
            }
            std::cmp::Ordering::Equal => {
                let cap = (character as usize).min(src_line.len());
                out.push_str(&src_line[..cap]);
                return Some(out);
            }
            std::cmp::Ordering::Greater => break,
        }
    }
    None
}

/// Walk `prefix` from the end looking for an unbalanced `(`. When
/// found, scan backwards to read the identifier immediately before
/// it and forward (inside the same call) to count top-level commas.
fn call_at_end_of(prefix: &str) -> Option<(String, u32)> {
    let bytes = prefix.as_bytes();
    let mut depth = 0i32;
    let mut commas = 0u32;
    let mut open_at: Option<usize> = None;
    let mut in_string: Option<char> = None;

    for (i, ch) in prefix.char_indices().rev() {
        match (ch, in_string) {
            ('"', None) => in_string = Some('"'),
            ('"', Some('"')) => in_string = None,
            ('\'', None) => in_string = Some('\''),
            ('\'', Some('\'')) => in_string = None,
            (_, Some(_)) => {}
            (')', _) => depth += 1,
            ('(', _) => {
                if depth == 0 {
                    open_at = Some(i);
                    break;
                }
                depth -= 1;
            }
            (',', _) if depth == 0 => commas += 1,
            _ => {}
        }
    }

    let open_at = open_at?;
    let name_end = bytes[..open_at]
        .iter()
        .rposition(|b| !b.is_ascii_whitespace())
        .map(|p| p + 1)?;
    let name_start = bytes[..name_end]
        .iter()
        .rposition(|b| !is_identifier_byte(*b))
        .map(|p| p + 1)
        .unwrap_or(0);
    if name_start >= name_end {
        return None;
    }
    let name = &prefix[name_start..name_end];
    if !is_identifier_start(name.chars().next()?) {
        return None;
    }
    Some((name.to_string(), commas))
}

fn parse_fn_line(line: &str) -> Option<FunctionSignature> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("fn ")?;
    let paren = rest.find('(')?;
    let name = rest[..paren].trim();
    if name.is_empty() || !is_identifier_start(name.chars().next()?) {
        return None;
    }
    let close = rest.find(')')?;
    if close <= paren {
        return None;
    }
    let params_raw = &rest[paren + 1..close];
    let parameters: Vec<String> = if params_raw.trim().is_empty() {
        Vec::new()
    } else {
        params_raw
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    };
    Some(FunctionSignature {
        name: name.to_string(),
        parameters,
        label: rest[..=close].trim_end().to_string()
            + rest[close + 1..]
                .split('{')
                .next()
                .map(str::trim_end)
                .unwrap_or(""),
    })
}

fn is_identifier_start(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}

fn is_identifier_byte(b: u8) -> bool {
    b == b'_' || (b as char).is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::{function_signatures, signature_help};

    #[test]
    fn parses_a_simple_fn() {
        let sigs = function_signatures("fn greet(name: String, mood: String) -> String {\n}\n");
        assert_eq!(sigs.len(), 1);
        assert_eq!(sigs[0].name, "greet");
        assert_eq!(sigs[0].parameters, vec!["name: String", "mood: String"]);
    }

    #[test]
    fn signature_help_finds_the_active_parameter() {
        let source = "fn greet(name: String, mood: String) -> String {\n  return name\n}\n\nlet x = greet(\"hi\", \"warm\")\n";
        // cursor right after the first argument's closing quote
        let help = signature_help(source, 4, "let x = greet(\"hi\"".len() as u32).expect("help");
        assert_eq!(help.signature.name, "greet");
        assert_eq!(help.active_parameter, 0);

        // cursor after the comma — second parameter active
        let help = signature_help(source, 4, "let x = greet(\"hi\", ".len() as u32).expect("help");
        assert_eq!(help.active_parameter, 1);
    }

    #[test]
    fn cursor_outside_a_call_returns_none() {
        assert!(signature_help("let x = 1\n", 0, 5).is_none());
    }

    #[test]
    fn unknown_function_returns_none() {
        // No `fn does_not_exist` declared in this file.
        assert!(signature_help("let x = does_not_exist(1)\n", 0, 23).is_none());
    }

    #[test]
    fn no_parameter_fn_still_resolves() {
        let source = "fn noop() {\n}\n\nlet _ = noop()\n";
        let help = signature_help(source, 3, "let _ = noop(".len() as u32).expect("help");
        assert_eq!(help.signature.parameters.len(), 0);
        assert_eq!(help.active_parameter, 0);
    }

    #[test]
    fn nested_call_uses_inner_signature() {
        let source = "fn outer(x: Int) {}\nfn inner(y: Int) {}\n\nlet _ = outer(inner(42))\n";
        let help = signature_help(source, 3, "let _ = outer(inner(".len() as u32).expect("help");
        assert_eq!(help.signature.name, "inner");
    }
}
