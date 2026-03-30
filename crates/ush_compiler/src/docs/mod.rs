mod parse;
mod render;

use crate::types::{HeapVec as Vec, OutputString as String};

#[derive(Debug, Clone, Default)]
pub struct ScriptDocs {
    summary: Option<String>,
    details: Vec<String>,
    usage: Option<String>,
    notes: Vec<String>,
    warnings: Vec<String>,
    errors: Vec<String>,
    examples: Vec<String>,
    see_also: Vec<String>,
    items: Vec<DocItem>,
}

#[derive(Debug, Clone)]
pub struct DocItem {
    kind: DocItemKind,
    name: String,
    signature: String,
    summary: Option<String>,
    details: Vec<String>,
    params: Vec<DocParam>,
    returns: Option<String>,
    notes: Vec<String>,
    warnings: Vec<String>,
    errors: Vec<String>,
    examples: Vec<String>,
    see_also: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocItemKind {
    Function,
    Enum,
    Trait,
}

#[derive(Debug, Clone)]
pub struct DocParam {
    name: String,
    description: String,
}

impl ScriptDocs {
    pub fn parse(source: &str) -> Self {
        parse::parse_source_docs(source)
    }

    pub fn is_empty(&self) -> bool {
        self.summary.is_none()
            && self.details.is_empty()
            && self.usage.is_none()
            && self.notes.is_empty()
            && self.warnings.is_empty()
            && self.errors.is_empty()
            && self.examples.is_empty()
            && self.see_also.is_empty()
            && self.items.is_empty()
    }

    pub fn render_help(&self, script_name: &str) -> String {
        render::render_help(self, script_name)
    }

    pub fn render_man(&self, script_name: &str, item: Option<&str>) -> String {
        render::render_man(self, script_name, item)
    }

    pub fn render_completion(&self, prefix: &str) -> String {
        render::render_completion(self, prefix)
    }

    pub(crate) fn completion_candidates(&self) -> Vec<String> {
        render::completion_candidates(self)
    }

    pub(crate) fn items(&self) -> &[DocItem] {
        &self.items
    }
}

impl DocItem {
    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }
}
