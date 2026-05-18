use ush_compiler::UshCompiler;

#[test]
fn rejects_non_exhaustive_match_on_enum() {
    let err = UshCompiler::default()
        .compile_source(
            r#"
            enum Colour {
              Red,
              Green,
              Blue,
            }
            let value = Colour::Red
            match value {
              Colour::Red => print "red"
              Colour::Green => print "green"
            }
        "#,
        )
        .expect_err("non-exhaustive match must be rejected");
    let message = format!("{err:#}");
    assert!(
        message.contains("non-exhaustive"),
        "expected non-exhaustive error, got: {message}",
    );
    assert!(
        message.contains("Colour::Blue"),
        "expected missing-variant hint, got: {message}",
    );
}

#[test]
fn wildcard_arm_makes_match_exhaustive() {
    UshCompiler::default()
        .compile_source(
            r#"
            enum Colour {
              Red,
              Green,
              Blue,
            }
            let value = Colour::Red
            match value {
              Colour::Red => print "red"
              _ => print "other"
            }
        "#,
        )
        .expect("wildcard arm must satisfy exhaustiveness");
}

#[test]
fn rejects_non_exhaustive_match_on_bool() {
    let err = UshCompiler::default()
        .compile_source(
            r#"
            let flag = true
            match flag {
              true => print "yes"
            }
        "#,
        )
        .expect_err("missing false arm must be rejected");
    let message = format!("{err:#}");
    assert!(
        message.contains("non-exhaustive"),
        "expected non-exhaustive error, got: {message}",
    );
}
