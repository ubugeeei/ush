use ush_compiler::UshCompiler;

fn generated_line(output: &str, needle: &str) -> usize {
    output
        .lines()
        .position(|line| line.contains(needle))
        .map(|index| index + 1)
        .unwrap_or_else(|| panic!("missing generated line: {needle}"))
}

#[test]
fn sourcemap_tracks_top_level_statements() {
    let compiled = UshCompiler::default()
        .compile_source_with_sourcemap("let greeting = \"hello\"\nprint greeting\n")
        .expect("compile");

    let assign_line = generated_line(&compiled.shell, "greeting='hello'");
    let print_line = generated_line(&compiled.shell, "printf '%s\\n' \"${greeting}\"");

    assert_eq!(compiled.sourcemap.source_line(1), None);
    assert_eq!(compiled.sourcemap.source_line(assign_line), Some(1));
    assert_eq!(compiled.sourcemap.source_line(print_line), Some(2));
}

#[test]
fn sourcemap_tracks_nested_function_body_lines() {
    let compiled = UshCompiler::default()
        .compile_source_with_sourcemap(concat!(
            "fn greet(name: String) -> String {\n",
            "  print name;\n",
            "  \"hi \" + name\n",
            "}\n",
        ))
        .expect("compile");

    let header_line = generated_line(&compiled.shell, "ush_fn_greet()");
    let body_line = generated_line(
        &compiled.shell,
        "printf '%s\\n' \"${__ush_fn_greet_arg_0}\"",
    );
    let return_line = generated_line(
        &compiled.shell,
        "printf '%s' \"$(printf '%s' 'hi ' \"${__ush_fn_greet_arg_0}\")\"",
    );

    assert_eq!(compiled.sourcemap.source_line(header_line), Some(1));
    assert_eq!(compiled.sourcemap.source_line(body_line), Some(2));
    assert_eq!(compiled.sourcemap.source_line(return_line), Some(3));
}
