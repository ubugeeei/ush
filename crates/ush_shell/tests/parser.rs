use std::collections::BTreeMap;

use ush_shell::{ParsedLine, parse_line};

#[test]
fn parses_pipeline_with_helper_stage() {
    let parsed = parse_line("echo hello | len", &BTreeMap::new()).expect("parse");
    match parsed {
        ParsedLine::Pipeline(pipeline) => {
            assert_eq!(pipeline.stages.len(), 2);
        }
        _ => panic!("expected pipeline"),
    }
}

#[test]
fn parses_legacy_length_helper_alias() {
    let parsed = parse_line("echo hello | length", &BTreeMap::new()).expect("parse");
    assert!(matches!(parsed, ParsedLine::Pipeline(_)));
}

#[test]
fn falls_back_for_posix_control_flow_and_grouping() {
    for source in [
        "! true",
        "(echo ok)",
        "if true; then echo ok; fi",
        "while false; do echo ok; done",
        "case x in x) echo ok ;; esac",
    ] {
        assert!(matches!(
            parse_line(source, &BTreeMap::new()).expect("parse"),
            ParsedLine::Fallback(_)
        ));
    }
}

#[test]
fn does_not_fall_back_for_commands_containing_shell_keyword_substrings() {
    for source in [
        "confirm proceed?",
        "input name?",
        "printf 'red\nblue\n' | select red blue",
    ] {
        assert!(matches!(
            parse_line(source, &BTreeMap::new()).expect("parse"),
            ParsedLine::Pipeline(_)
        ));
    }
}
