use alloc::collections::BTreeSet;

use crate::types::{HeapVec as Vec, OutputString as String};

use super::{DocItem, DocItemKind, ScriptDocs};

pub(super) fn render_help(docs: &ScriptDocs, script_name: &str) -> String {
    let mut out = String::new();
    if let Some(summary) = &docs.summary {
        push_line(&mut out, &format!("{script_name} - {summary}"));
    } else {
        push_line(&mut out, script_name);
    }
    push_blank(&mut out);
    push_line(&mut out, "Usage:");
    push_line(&mut out, &format!("  {script_name} [--help]"));
    push_line(&mut out, &format!("  {script_name} [--man [ITEM]]"));
    push_line(&mut out, &format!("  {script_name} [--complete [PREFIX]]"));
    if let Some(usage) = &docs.usage {
        push_line(&mut out, &format!("  {usage}"));
    }
    if !docs.details.is_empty() {
        push_blank(&mut out);
        push_line(&mut out, "Description:");
        push_indented_lines(&mut out, &docs.details, "  ");
    }
    if !docs.notes.is_empty() {
        push_blank(&mut out);
        push_line(&mut out, "Notes:");
        push_bullets(&mut out, &docs.notes, "  ");
    }
    if !docs.warnings.is_empty() {
        push_blank(&mut out);
        push_line(&mut out, "Warnings:");
        push_bullets(&mut out, &docs.warnings, "  ");
    }
    if !docs.errors.is_empty() {
        push_blank(&mut out);
        push_line(&mut out, "Errors:");
        push_bullets(&mut out, &docs.errors, "  ");
    }
    if !docs.examples.is_empty() {
        push_blank(&mut out);
        push_line(&mut out, "Examples:");
        push_indented_lines(&mut out, &docs.examples, "  ");
    }
    if !docs.see_also.is_empty() {
        push_blank(&mut out);
        push_line(&mut out, "See also:");
        push_bullets(&mut out, &docs.see_also, "  ");
    }
    if !docs.items.is_empty() {
        push_blank(&mut out);
        push_line(&mut out, "Documented items:");
        for item in &docs.items {
            push_line(&mut out, &format!("  {}", item.signature));
            if let Some(summary) = &item.summary {
                push_line(&mut out, &format!("      {summary}"));
            }
            if !item.errors.is_empty() {
                push_line(
                    &mut out,
                    &format!("      errors: {}", item.errors.join(", ")),
                );
            }
        }
    }
    out
}

pub(super) fn render_man(docs: &ScriptDocs, script_name: &str, item: Option<&str>) -> String {
    if let Some(name) = item {
        return docs
            .items
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| render_item_man(script_name, entry))
            .unwrap_or_else(|| format!("No documented item named `{name}`.\n"));
    }

    let mut out = String::new();
    push_section(&mut out, "NAME");
    if let Some(summary) = &docs.summary {
        push_line(&mut out, &format!("{script_name} - {summary}"));
    } else {
        push_line(&mut out, script_name);
    }
    push_section(&mut out, "SYNOPSIS");
    push_line(&mut out, &format!("{script_name} [--help]"));
    push_line(&mut out, &format!("{script_name} [--man [ITEM]]"));
    push_line(&mut out, &format!("{script_name} [--complete [PREFIX]]"));
    if let Some(usage) = &docs.usage {
        push_line(&mut out, usage);
    }
    if docs.summary.is_some() || !docs.details.is_empty() {
        push_section(&mut out, "DESCRIPTION");
        if let Some(summary) = &docs.summary {
            push_line(&mut out, summary);
        }
        push_lines(&mut out, &docs.details);
    }
    push_list_section(&mut out, "NOTES", &docs.notes);
    push_list_section(&mut out, "WARNINGS", &docs.warnings);
    push_list_section(&mut out, "ERRORS", &docs.errors);
    if !docs.examples.is_empty() {
        push_section(&mut out, "EXAMPLES");
        push_lines(&mut out, &docs.examples);
    }
    push_items(&mut out, "FUNCTIONS", docs, DocItemKind::Function);
    push_items(&mut out, "ENUMS", docs, DocItemKind::Enum);
    push_items(&mut out, "TRAITS", docs, DocItemKind::Trait);
    push_list_section(&mut out, "SEE ALSO", &docs.see_also);
    out
}

pub(super) fn completion_candidates(docs: &ScriptDocs) -> Vec<String> {
    let mut words = BTreeSet::new();
    for word in ["--help", "-h", "--man", "man", "--complete", "complete"] {
        words.insert(String::from(word));
    }
    for item in &docs.items {
        words.insert(item.name.clone());
    }
    words.into_iter().collect()
}

pub(super) fn render_completion(docs: &ScriptDocs, prefix: &str) -> String {
    let mut out = String::new();
    for item in completion_candidates(docs) {
        if item.starts_with(prefix) {
            push_line(&mut out, &item);
        }
    }
    out
}

fn render_item_man(script_name: &str, item: &DocItem) -> String {
    let mut out = String::new();
    push_section(&mut out, "NAME");
    push_line(&mut out, &format!("{script_name} {}", item.name));
    push_section(&mut out, "SIGNATURE");
    push_line(&mut out, &item.signature);
    if item.summary.is_some() || !item.details.is_empty() {
        push_section(&mut out, "DESCRIPTION");
        if let Some(summary) = &item.summary {
            push_line(&mut out, summary);
        }
        push_lines(&mut out, &item.details);
    }
    if !item.params.is_empty() {
        push_section(&mut out, "PARAMETERS");
        for param in &item.params {
            push_line(&mut out, &format!("{} - {}", param.name, param.description));
        }
    }
    if let Some(returns) = &item.returns {
        push_section(&mut out, "RETURNS");
        push_line(&mut out, returns);
    }
    push_list_section(&mut out, "NOTES", &item.notes);
    push_list_section(&mut out, "WARNINGS", &item.warnings);
    push_list_section(&mut out, "ERRORS", &item.errors);
    if !item.examples.is_empty() {
        push_section(&mut out, "EXAMPLES");
        push_lines(&mut out, &item.examples);
    }
    push_list_section(&mut out, "SEE ALSO", &item.see_also);
    out
}

fn push_items(out: &mut String, heading: &str, docs: &ScriptDocs, kind: DocItemKind) {
    let items: Vec<_> = docs.items.iter().filter(|item| item.kind == kind).collect();
    if items.is_empty() {
        return;
    }
    push_section(out, heading);
    for item in items {
        push_line(out, &item.name);
        push_line(out, &format!("  {}", item.signature));
        if let Some(summary) = &item.summary {
            push_line(out, &format!("  {summary}"));
        }
        if !item.errors.is_empty() {
            push_line(out, &format!("  errors: {}", item.errors.join(", ")));
        }
    }
}

fn push_indented_lines(out: &mut String, lines: &[String], indent: &str) {
    for line in lines {
        if line.is_empty() {
            push_blank(out);
        } else {
            push_line(out, &format!("{indent}{line}"));
        }
    }
}

fn push_lines(out: &mut String, lines: &[String]) {
    for line in lines {
        if line.is_empty() {
            push_blank(out);
        } else {
            push_line(out, line);
        }
    }
}

fn push_bullets(out: &mut String, lines: &[String], indent: &str) {
    for line in lines {
        push_line(out, &format!("{indent}- {line}"));
    }
}

fn push_list_section(out: &mut String, heading: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    push_section(out, heading);
    for line in lines {
        push_line(out, line);
    }
}

fn push_section(out: &mut String, title: &str) {
    push_blank(out);
    push_line(out, title);
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

fn push_blank(out: &mut String) {
    if !out.is_empty() && !out.ends_with("\n\n") {
        out.push('\n');
    }
}
