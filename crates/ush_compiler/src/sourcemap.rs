use alloc::vec::Vec;
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
    pub fn source_line(&self, generated_line: usize) -> Option<usize> {
        self.lines
            .get(generated_line.saturating_sub(1))
            .and_then(|line| line.source_line)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapLine {
    pub generated_line: usize,
    pub source_line: Option<usize>,
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

    pub(crate) fn into_compiled(mut self) -> CompiledScript {
        self.finish_partial_line();
        let sourcemap = SourceMap {
            lines: self
                .line_origins
                .into_iter()
                .enumerate()
                .map(|(index, source_line)| SourceMapLine {
                    generated_line: index + 1,
                    source_line,
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
