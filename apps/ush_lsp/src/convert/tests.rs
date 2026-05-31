use super::{diagnostics, full_document_edit};

#[test]
fn creates_a_full_document_edit() {
    let edit = full_document_edit("print \"a\"\n", "print \"b\"\n");
    assert_eq!(edit.range.end.line, 1);
    assert_eq!(edit.new_text, "print \"b\"\n");
}

#[test]
fn maps_diagnostics_to_ranges() {
    let diagnostics = diagnostics(
        "print \"ok\"\nprint missing\n",
        &[ush_tooling::UshDiagnostic {
            line: 1,
            message: "unknown variable".to_string(),
        }],
    );

    assert_eq!(diagnostics[0].range.start.line, 1);
}
