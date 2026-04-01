use ush_compiler::UshCompiler;

#[test]
fn expressions_allow_call_arguments_with_parentheses_inside_strings() {
    let output = UshCompiler::default()
        .compile_source(
            r#"
            use std::regex::capture
            print $ capture("release-v0.3.4", "v([0-9.]+)", 1)
            "#,
        )
        .expect("compile source");

    assert!(output.contains("ush_fn_std__regex__capture"));
}
