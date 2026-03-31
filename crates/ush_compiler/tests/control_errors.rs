use ush_compiler::UshCompiler;

#[test]
fn heterogeneous_tuple_iteration_is_rejected() {
    let error = UshCompiler::default()
        .compile_source(
            r#"
            let pair = (1, "two")
            for item in pair {
              print item
            }
        "#,
        )
        .expect_err("tuple iteration should fail");

    assert!(
        error
            .to_string()
            .contains("for-in over tuples requires all items to share one type")
    );
}
