use alloc::vec::Vec;
use core::fmt::Write as _;
use core::mem;

use crate::types::OutputString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledScript {
    pub shell: OutputString,
    pub sourcemap: SourceMap,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceMap {
    pub lines: Vec<SourceMapLine>,
}

impl SourceMap {
    pub fn line(&self, generated_line: usize) -> Option<&SourceMapLine> {
        self.lines.get(generated_line.saturating_sub(1))
    }

    pub fn source_line(&self, generated_line: usize) -> Option<usize> {
        self.line(generated_line).and_then(|line| line.source_line)
    }

    pub fn render_listing(&self) -> OutputString {
        self.render_listing_inner(false)
    }

    pub fn render_mapped_listing(&self) -> OutputString {
        self.render_listing_inner(true)
    }

    fn render_listing_inner(&self, mapped_only: bool) -> OutputString {
        let mut out = OutputString::new();
        for line in &self.lines {
            if mapped_only && line.source_line.is_none() {
                continue;
            }
            let source = line
                .source_line
                .map(|value| format!("S{value:04}"))
                .unwrap_or_else(|| "----".to_string());
            let _ = writeln!(
                out,
                "G{:04} -> {} | {}",
                line.generated_line, source, line.generated_text
            );
            if let Some(source_text) = &line.source_text {
                let _ = writeln!(out, "             <= {}", source_text);
            }
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapLine {
    pub generated_line: usize,
    pub source_line: Option<usize>,
    pub generated_text: OutputString,
    pub source_text: Option<OutputString>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct OutputBuffer {
    text: OutputString,
    line_origins: Vec<Option<usize>>,
    current_origin: Option<usize>,
}

impl OutputBuffer {
    pub(crate) fn from_text(text: &str) -> Self {
        let mut out = Self::default();
        out.push_str(text);
        out
    }

    pub(crate) fn push_str(&mut self, value: &str) {
        self.text.push_str(value);
        for ch in value.chars() {
            if ch == '\n' {
                self.line_origins.push(self.current_origin);
            }
        }
    }

    pub(crate) fn push(&mut self, ch: char) {
        self.text.push(ch);
        if ch == '\n' {
            self.line_origins.push(self.current_origin);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub(crate) fn set_origin(&mut self, line: Option<usize>) -> Option<usize> {
        mem::replace(&mut self.current_origin, line)
    }

    pub(crate) fn append_buffer(&mut self, other: &Self, indent: usize) {
        let origins = other.completed_line_origins();
        for (line, origin) in other.text.lines().zip(origins.into_iter()) {
            let previous = self.set_origin(origin);
            if indent > 0 {
                self.push_str(&" ".repeat(indent));
            }
            self.push_str(line);
            self.push('\n');
            self.set_origin(previous);
        }
    }

    pub(crate) fn into_compiled(mut self, source: Option<&str>) -> CompiledScript {
        self.finish_partial_line();
        let generated_lines = self
            .text
            .lines()
            .map(OutputString::from)
            .collect::<Vec<_>>();
        debug_assert_eq!(generated_lines.len(), self.line_origins.len());
        let source_lines = source
            .map(|value| value.lines().map(OutputString::from).collect::<Vec<_>>())
            .unwrap_or_default();
        let sourcemap = SourceMap {
            lines: self
                .line_origins
                .into_iter()
                .enumerate()
                .map(|(index, source_line)| {
                    let generated_text = generated_lines.get(index).cloned().unwrap_or_default();
                    let source_text = source_line
                        .and_then(|line| source_lines.get(line.saturating_sub(1)))
                        .cloned();
                    SourceMapLine {
                        generated_line: index + 1,
                        source_line,
                        generated_text,
                        source_text,
                    }
                })
                .collect(),
        };
        CompiledScript {
            shell: self.text,
            sourcemap,
        }
    }

    fn completed_line_origins(&self) -> Vec<Option<usize>> {
        let mut origins = self.line_origins.clone();
        if !self.text.is_empty() && !self.text.ends_with('\n') {
            origins.push(self.current_origin);
        }
        origins
    }

    fn finish_partial_line(&mut self) {
        if !self.text.is_empty() && !self.text.ends_with('\n') {
            self.line_origins.push(self.current_origin);
        }
    }
}
