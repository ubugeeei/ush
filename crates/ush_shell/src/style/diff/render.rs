use std::fmt::Write as _;

use super::{
    model::{DiffLine, DiffLineKind, DiffOptions, DiffSection},
    parse::parse_unified_diff,
};
use crate::style::common::{
    BLUE_BOLD, BOLD, CYAN_BOLD, GREEN_BOLD, MAGENTA_BOLD, RED_BOLD, YELLOW_BOLD, badge, dim, paint,
    pluralize,
};

pub(super) fn render_diff_clean(options: &DiffOptions) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {} {}",
        paint(BLUE_BOLD, "diff"),
        paint(CYAN_BOLD, &options.targets[0]),
        paint(MAGENTA_BOLD, &options.targets[1])
    );
    let _ = writeln!(
        out,
        "{} {}",
        badge("same", GREEN_BOLD),
        dim("no differences")
    );
    out
}

pub(super) fn render_diff_report(options: &DiffOptions, stdout: &str) -> String {
    let report = parse_unified_diff(stdout);
    let total_hunks = report
        .sections
        .iter()
        .map(|section| section.hunks.len())
        .sum::<usize>();
    let total_additions = report
        .sections
        .iter()
        .map(|section| section.additions)
        .sum::<usize>();
    let total_deletions = report
        .sections
        .iter()
        .map(|section| section.deletions)
        .sum::<usize>();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {} {}",
        paint(BLUE_BOLD, "diff"),
        paint(CYAN_BOLD, &options.targets[0]),
        paint(MAGENTA_BOLD, &options.targets[1])
    );

    let mut meta = Vec::new();
    if !report.sections.is_empty() {
        meta.push(pluralize(
            report.sections.len(),
            "changed file",
            "changed files",
        ));
    }
    if total_hunks > 0 {
        meta.push(pluralize(total_hunks, "hunk", "hunks"));
    }
    if total_additions > 0 {
        meta.push(format!("+{total_additions}"));
    }
    if total_deletions > 0 {
        meta.push(format!("-{total_deletions}"));
    }
    if meta.is_empty() {
        meta.push("differences detected".to_string());
    }
    let _ = writeln!(out, "{}", dim(meta.join(", ")));
    for note in &report.notes {
        let _ = writeln!(out, "{} {}", badge("note", YELLOW_BOLD), dim(note));
    }
    for (index, section) in report.sections.iter().enumerate() {
        if index > 0 || !report.notes.is_empty() {
            out.push('\n');
        }
        render_diff_section(&mut out, section);
    }
    out
}

fn render_diff_section(out: &mut String, section: &DiffSection) {
    let mut parts = vec![
        paint(BOLD, &section.old_label),
        paint(CYAN_BOLD, "->"),
        paint(BOLD, &section.new_label),
    ];
    if !section.hunks.is_empty() {
        parts.push(badge(
            pluralize(section.hunks.len(), "hunk", "hunks"),
            BLUE_BOLD,
        ));
    }
    if section.additions > 0 {
        parts.push(badge(format!("+{}", section.additions), GREEN_BOLD));
    }
    if section.deletions > 0 {
        parts.push(badge(format!("-{}", section.deletions), RED_BOLD));
    }
    let _ = writeln!(out, "{}", parts.join(" "));
    for note in &section.notes {
        let _ = writeln!(out, "{} {}", badge("note", YELLOW_BOLD), dim(note));
    }
    for (index, hunk) in section.hunks.iter().enumerate() {
        if index > 0 || !section.notes.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(out, "{}", paint(CYAN_BOLD, &hunk.header));
        for line in &hunk.lines {
            render_diff_line(out, line);
        }
    }
}

fn render_diff_line(out: &mut String, line: &DiffLine) {
    let rendered = match line.kind {
        DiffLineKind::Context | DiffLineKind::Note => dim(&line.text),
        DiffLineKind::Added => paint(GREEN_BOLD, &line.text),
        DiffLineKind::Removed => paint(RED_BOLD, &line.text),
    };
    let _ = writeln!(out, "{rendered}");
}
