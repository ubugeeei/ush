use ush_compiler::UshCompiler;

fn compile_error(source: &str) -> String {
    UshCompiler::default()
        .compile_source(source)
        .expect_err("program should fail to compile")
        .to_string()
}

#[test]
fn missing_struct_fields_are_rejected() {
    let error = compile_error(
        r#"
        enum Response {
          Http { code: Int, body: String },
        }
        let response = Response::Http { code: 200 }
    "#,
    );

    assert!(error.contains("missing field body"));
}

#[test]
fn duplicate_enums_are_rejected() {
    let error = compile_error(
        r#"
        enum Option {
          None,
        }
        enum Option {
          Some(String),
        }
    "#,
    );

    assert!(error.contains("duplicate enum"));
}

#[test]
fn tuple_shape_mismatches_are_rejected() {
    let error = compile_error(
        r#"
        enum Option {
          None,
          Some(String),
        }
        let value = Option::Some("ok", "extra")
    "#,
    );

    assert!(error.contains("variant field shape mismatch"));
}

#[test]
fn unknown_functions_are_rejected() {
    let error = compile_error(
        r#"
        async worker()
    "#,
    );

    assert!(error.contains("unknown function"));
}

#[test]
fn function_argument_type_mismatches_are_rejected() {
    let error = compile_error(
        r#"
        fn greet(message: String, count: Int) {
          print message + ":" + count
        }
        greet(1, "two")
    "#,
    );

    assert!(error.contains("type mismatch"));
}

#[test]
fn function_argument_count_mismatches_are_rejected() {
    let error = compile_error(
        r#"
        fn greet(message: String) {
          print message
        }
        greet("hello", "extra")
    "#,
    );

    assert!(error.contains("expects at most 1 arguments"));
}

#[test]
fn async_bindings_require_declared_return_types() {
    let error = compile_error(
        r#"
        fn worker(message: String) {
          print message
        }
        let task = async worker("hello")
    "#,
    );

    assert!(error.contains("async bindings require a return type"));
}

#[test]
fn await_requires_a_task_handle() {
    let error = compile_error(
        r#"
        let value = "hello"
        let result = await value
    "#,
    );

    assert!(error.contains("await expects a task handle"));
}

#[test]
fn return_outside_functions_is_rejected() {
    let error = compile_error(
        r#"
        return "nope"
    "#,
    );

    assert!(error.contains("return is only valid inside functions"));
}

#[test]
fn call_expressions_require_return_types() {
    let error = compile_error(
        r#"
        fn greet(message: String) {
          print message
        }
        let value = greet "hello"
    "#,
    );

    assert!(error.contains("does not return a value"));
}

#[test]
fn impl_requires_a_known_trait() {
    let error = compile_error(
        r#"
        impl Named for Int {}
    "#,
    );

    assert!(error.contains("unknown trait"));
}

#[test]
fn duplicate_impls_are_rejected() {
    let error = compile_error(
        r#"
        trait Named {}
        impl Named for String {}
        impl Named for String {}
    "#,
    );

    assert!(error.contains("duplicate impl"));
}

#[test]
fn unknown_named_arguments_are_rejected() {
    let error = compile_error(
        r#"
        fn greet(name: String) {
          print name
        }
        greet value: "ush"
    "#,
    );

    assert!(error.contains("unknown argument label"));
}

#[test]
fn default_value_types_must_match_parameter_types() {
    let error = compile_error(
        r#"
        fn greet(#[default("oops")] count: Int) {
          print count
        }
        greet()
    "#,
    );

    assert!(error.contains("type mismatch for `count`"));
}

#[test]
fn duplicate_parameter_aliases_are_rejected() {
    let error = compile_error(
        r#"
        fn bin(#[alias("n")] name: String, #[alias("n")] count: Int) {
          print name + ":" + count
        }
    "#,
    );

    assert!(error.contains("duplicate parameter alias"));
}
