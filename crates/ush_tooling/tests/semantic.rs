use ush_tooling::{SemanticTokenKind, semantic_tokens};

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

#[test]
fn tokenizes_inline_shell_prefix_as_operator() {
    let tokens = semantic_tokens("$ printf '%s\\n' hi");
    assert_eq!(tokens[0].kind, SemanticTokenKind::Operator);
    assert_eq!(tokens[1].kind, SemanticTokenKind::Function);
}
