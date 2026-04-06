use alloc::vec::Vec;
use core::mem;

use crate::types::OutputString;

use super::{CompiledScript, SourceMap, SourceMapLine, SourceMapSection};

#[derive(Debug, Clone)]
pub(crate) struct OutputBuffer {
    text: OutputString,
    line_origins: Vec<Option<usize>>,
    line_sections: Vec<SourceMapSection>,
    current_origin: Option<usize>,
    current_section: SourceMapSection,
}

impl Default for OutputBuffer {
    fn default() -> Self {
        Self {
            text: OutputString::new(),
            line_origins: Vec::new(),
            line_sections: Vec::new(),
            current_origin: None,
            current_section: SourceMapSection::UserCode,
        }
    }
}

impl OutputBuffer {
    pub(crate) fn with_section(section: SourceMapSection) -> Self {
        Self {
            current_section: section,
            ..Self::default()
        }
    }

    pub(crate) fn push_str(&mut self, value: &str) {
        self.text.push_str(value);
        for ch in value.chars() {
            if ch == '\n' {
                self.line_origins.push(self.current_origin);
                self.line_sections.push(self.current_section);
            }
        }
    }

    pub(crate) fn push(&mut self, ch: char) {
        self.text.push(ch);
        if ch == '\n' {
            self.line_origins.push(self.current_origin);
            self.line_sections.push(self.current_section);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub(crate) fn set_origin(&mut self, line: Option<usize>) -> Option<usize> {
        mem::replace(&mut self.current_origin, line)
    }

    pub(crate) fn set_section(&mut self, section: SourceMapSection) -> SourceMapSection {
        mem::replace(&mut self.current_section, section)
    }

    pub(crate) fn append_buffer(&mut self, other: &Self, indent: usize) {
        let metadata = other.completed_line_metadata();
        for (line, (origin, section)) in other.text.lines().zip(metadata.into_iter()) {
            let previous_origin = self.set_origin(origin);
            let previous_section = self.set_section(section);
            if indent > 0 {
                self.push_str(&" ".repeat(indent));
            }
            self.push_str(line);
            self.push('\n');
            self.set_origin(previous_origin);
            self.set_section(previous_section);
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
        debug_assert_eq!(generated_lines.len(), self.line_sections.len());
        let source_lines = source
            .map(|value| value.lines().map(OutputString::from).collect::<Vec<_>>())
            .unwrap_or_default();
        let sourcemap = SourceMap {
            lines: self
                .line_origins
                .into_iter()
                .zip(self.line_sections)
                .enumerate()
                .map(|(index, (source_line, section))| {
                    let generated_text = generated_lines.get(index).cloned().unwrap_or_default();
                    let source_text = source_line
                        .and_then(|line| source_lines.get(line.saturating_sub(1)))
                        .cloned();
                    SourceMapLine {
                        generated_line: index + 1,
                        section,
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

    fn completed_line_metadata(&self) -> Vec<(Option<usize>, SourceMapSection)> {
        let mut metadata = self
            .line_origins
            .iter()
            .copied()
            .zip(self.line_sections.iter().copied())
            .collect::<Vec<_>>();
        if !self.text.is_empty() && !self.text.ends_with('\n') {
            metadata.push((self.current_origin, self.current_section));
        }
        metadata
    }

    fn finish_partial_line(&mut self) {
        if !self.text.is_empty() && !self.text.ends_with('\n') {
            self.line_origins.push(self.current_origin);
            self.line_sections.push(self.current_section);
        }
    }
}
