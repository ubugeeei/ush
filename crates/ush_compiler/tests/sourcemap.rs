use ush_compiler::{SourceMapSection, UshCompiler};

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
    assert_eq!(
        compiled
            .sourcemap
            .line(assign_line)
            .and_then(|line| line.source_text.as_deref()),
        Some("let greeting = \"hello\"")
    );
    assert_eq!(
        compiled
            .sourcemap
            .line(print_line)
            .map(|line| line.generated_text.as_str()),
        Some("printf '%s\\n' \"${greeting}\"")
    );
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

#[test]
fn sourcemap_render_listing_pairs_generated_and_source_lines() {
    let compiled = UshCompiler::default()
        .compile_source_with_sourcemap("let greeting = \"hello\"\nprint greeting\n")
        .expect("compile");

    let listing = compiled.sourcemap.render_listing();

    assert!(listing.contains("generated "));
    assert!(listing.contains("mapped span G"));
    assert!(listing.contains("-- runtime-support --"));
    assert!(listing.contains("-- user-code --"));
    assert!(listing.contains("greeting='hello'"));
    assert!(listing.contains("<= let greeting = \"hello\""));
    assert!(listing.contains("printf '%s\\n' \"${greeting}\""));
    assert!(listing.contains("<= print greeting"));
}

#[test]
fn sourcemap_summary_and_source_index_group_related_lines() {
    let compiled = UshCompiler::default()
        .compile_source_with_sourcemap("if true {\n  print \"hi\"\n}\n")
        .expect("compile");

    let condition_lines = compiled.sourcemap.generated_lines_for_source(1);
    let print_lines = compiled.sourcemap.generated_lines_for_source(2);
    let summary = compiled.sourcemap.summary();
    let source_index = compiled.sourcemap.source_index();
    let user_code = summary
        .sections
        .iter()
        .find(|section| section.section == SourceMapSection::UserCode)
        .expect("user-code section");

    assert!(condition_lines.len() >= 3);
    assert_eq!(print_lines.len(), 1);
    assert_eq!(summary.source_line_count, 2);
    assert_eq!(
        source_index
            .iter()
            .find(|line| line.source_line == 1)
            .and_then(|line| line.source_text.as_deref()),
        Some("if true {")
    );
    assert_eq!(
        source_index
            .iter()
            .find(|line| line.source_line == 1)
            .map(|line| line.generated_lines.as_slice()),
        Some(condition_lines.as_slice())
    );
    assert_eq!(user_code.mapped_line_count, condition_lines.len() + print_lines.len());
    assert!(user_code.generated_line_count >= user_code.mapped_line_count);
}

#[test]
fn sourcemap_render_mapped_listing_skips_unmapped_sections() {
    let compiled = UshCompiler::default()
        .compile_source_with_sourcemap("print \"hello\"\n")
        .expect("compile");

    let listing = compiled.sourcemap.render_mapped_listing();

    assert!(listing.contains("mapped "));
    assert!(listing.contains("-- user-code --"));
    assert!(!listing.contains("-- runtime-support --"));
}
