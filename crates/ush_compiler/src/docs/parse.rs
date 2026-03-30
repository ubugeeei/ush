use crate::types::{HeapVec as Vec, OutputString as String};

use super::{DocItem, DocItemKind, DocParam, ScriptDocs};

pub(super) fn parse_source_docs(source: &str) -> ScriptDocs {
    let mut docs = ScriptDocs::default();
    let mut pending = Vec::new();
    let mut depth = 0usize;
    let mut separated = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if depth == 0 && trimmed.starts_with("#|") {
            if separated && !pending.is_empty() && docs.is_empty() {
                apply_script_docs(&mut pending, &mut docs);
            }
            pending.push(trimmed[2..].trim_start().into());
            separated = false;
            continue;
        }
        if trimmed.is_empty() {
            if depth == 0 && !pending.is_empty() {
                separated = true;
            }
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }

        if depth == 0 {
            if let Some(item) = parse_item(trimmed) {
                if !pending.is_empty() {
                    apply_item_docs(&mut pending, item, &mut docs);
                }
            } else if !pending.is_empty() && docs.is_empty() {
                apply_script_docs(&mut pending, &mut docs);
            }
            separated = false;
        }

        depth = update_depth(depth, trimmed);
    }

    if !pending.is_empty() && docs.is_empty() {
        apply_script_docs(&mut pending, &mut docs);
    }
    docs
}

fn parse_item(trimmed: &str) -> Option<DocItem> {
    if let Some(rest) = trimmed.strip_prefix("fn ") {
        let header = rest.strip_suffix('{')?.trim();
        let name = header.split_once('(')?.0.trim();
        return Some(empty_item(
            DocItemKind::Function,
            name,
            format!("fn {header}"),
        ));
    }
    if let Some(rest) = trimmed.strip_prefix("enum ") {
        let name = rest.strip_suffix('{')?.trim();
        return Some(empty_item(DocItemKind::Enum, name, format!("enum {name}")));
    }
    if let Some(rest) = trimmed.strip_prefix("trait ") {
        let name = rest.strip_suffix('{')?.trim();
        return Some(empty_item(
            DocItemKind::Trait,
            name,
            format!("trait {name}"),
        ));
    }
    None
}

fn empty_item(kind: DocItemKind, name: &str, signature: String) -> DocItem {
    DocItem {
        kind,
        name: name.into(),
        signature,
        summary: None,
        details: Vec::new(),
        params: Vec::new(),
        returns: None,
        examples: Vec::new(),
    }
}

fn apply_script_docs(pending: &mut Vec<String>, docs: &mut ScriptDocs) {
    let block = parse_block(pending);
    docs.summary = block.summary;
    docs.details = block.details;
    docs.usage = block.usage;
    docs.examples = block.examples;
    pending.clear();
}

fn apply_item_docs(pending: &mut Vec<String>, mut item: DocItem, docs: &mut ScriptDocs) {
    let block = parse_block(pending);
    item.summary = block.summary;
    item.details = block.details;
    item.params = block.params;
    item.returns = block.returns;
    item.examples = block.examples;
    docs.items.push(item);
    pending.clear();
}

fn parse_block(lines: &[String]) -> ParsedBlock {
    let mut block = ParsedBlock::default();
    let mut prose = Vec::new();

    for line in lines {
        if let Some(rest) = line.strip_prefix("@usage ") {
            block.usage = Some(rest.trim().into());
        } else if let Some(rest) = line.strip_prefix("@return ") {
            block.returns = Some(rest.trim().into());
        } else if let Some(rest) = line.strip_prefix("@example ") {
            block.examples.push(rest.trim().into());
        } else if let Some(rest) = line.strip_prefix("@param ") {
            let mut parts = rest.trim().splitn(2, char::is_whitespace);
            let name = parts.next().unwrap_or_default().trim();
            let description = parts.next().unwrap_or_default().trim();
            if !name.is_empty() {
                block.params.push(DocParam {
                    name: name.into(),
                    description: description.into(),
                });
            }
        } else if !line.is_empty() {
            prose.push(line.clone());
        }
    }

    if let Some((head, tail)) = prose.split_first() {
        block.summary = Some(head.clone());
        block.details = tail.to_vec();
    }
    block
}

fn update_depth(mut depth: usize, line: &str) -> usize {
    let (mut single, mut double, mut escaped) = (false, false, false);
    for ch in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '#' if !single && !double => break,
            '\\' if double => escaped = true,
            '\'' if !double => single = !single,
            '"' if !single => double = !double,
            '{' if !single && !double => depth += 1,
            '}' if !single && !double && depth > 0 => depth -= 1,
            _ => {}
        }
    }
    depth
}

#[derive(Default)]
struct ParsedBlock {
    summary: Option<String>,
    details: Vec<String>,
    usage: Option<String>,
    params: Vec<DocParam>,
    returns: Option<String>,
    examples: Vec<String>,
}
