use std::fmt::Write as _;

use super::{
    super::common::{BLUE_BOLD, BOLD, CYAN_BOLD, YELLOW_BOLD, badge, dim, paint, pluralize},
    model::{GrepGroup, GrepMatch, GrepMatchRow, GrepOptions, GrepReport},
};

pub(super) fn parse_grep_output(stdout: &str) -> GrepReport {
    let mut report = GrepReport::default();
    for line in stdout.lines() {
        let Some((source, rest)) = line.split_once(':') else {
            if !line.is_empty() {
                report.notes.push(line.to_string());
            }
            continue;
        };
        let Some((line_number, text)) = rest.split_once(':') else {
            report.notes.push(line.to_string());
            continue;
        };
        let Ok(line_number) = line_number.parse() else {
            report.notes.push(line.to_string());
            continue;
        };
        push_grep_match(
            &mut report.groups,
            GrepMatch {
                source: normalize_grep_source(source),
                line_number,
                text: text.to_string(),
            },
        );
    }
    report
}

pub(super) fn render_grep_no_matches(options: &GrepOptions) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "grep"),
        paint(CYAN_BOLD, grep_query_label(options))
    );
    let _ = writeln!(
        out,
        "{} {}",
        badge("no matches", YELLOW_BOLD),
        dim("pattern not found")
    );
    out
}

pub(super) fn render_grep_report(options: &GrepOptions, report: &GrepReport) -> String {
    let match_count = report
        .groups
        .iter()
        .map(|group| group.rows.len())
        .sum::<usize>();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "grep"),
        paint(CYAN_BOLD, grep_query_label(options))
    );

    let mut meta = vec![pluralize(match_count, "match", "matches")];
    if !report.groups.is_empty() {
        meta.push(pluralize(report.groups.len(), "source", "sources"));
    }
    let _ = writeln!(out, "{}", dim(meta.join(", ")));
    for note in &report.notes {
        let _ = writeln!(out, "{} {}", badge("note", YELLOW_BOLD), dim(note));
    }
    for (index, group) in report.groups.iter().enumerate() {
        if index > 0 || !report.notes.is_empty() {
            out.push('\n');
        }
        render_grep_group(&mut out, group);
    }
    out
}

fn push_grep_match(groups: &mut Vec<GrepGroup>, row: GrepMatch) {
    if let Some(group) = groups.last_mut()
        && group.source == row.source
    {
        group.rows.push(GrepMatchRow {
            line_number: row.line_number,
            text: row.text,
        });
        return;
    }
    groups.push(GrepGroup {
        source: row.source,
        rows: vec![GrepMatchRow {
            line_number: row.line_number,
            text: row.text,
        }],
    });
}

fn normalize_grep_source(source: &str) -> String {
    match source {
        "(standard input)" => "stdin".to_string(),
        _ => source.to_string(),
    }
}

fn grep_query_label(options: &GrepOptions) -> String {
    if options.patterns.len() == 1 && options.pattern_files.is_empty() {
        return options.patterns[0].clone();
    }
    if options.patterns.is_empty() && options.pattern_files.len() == 1 {
        return format!("patterns from {}", options.pattern_files[0]);
    }
    pluralize(
        options.patterns.len() + options.pattern_files.len(),
        "pattern",
        "patterns",
    )
}

fn render_grep_group(out: &mut String, group: &GrepGroup) {
    let _ = writeln!(
        out,
        "{} {}",
        paint(BOLD, &group.source),
        badge(pluralize(group.rows.len(), "match", "matches"), BLUE_BOLD)
    );
    for row in &group.rows {
        let _ = writeln!(
            out,
            "  {} {}",
            badge(format!("line {}", row.line_number), CYAN_BOLD),
            paint(BOLD, &row.text)
        );
    }
}
