#[derive(Default)]
pub(super) struct GrepOptions {
    pub(super) ignore_case: bool,
    pub(super) invert_match: bool,
    pub(super) word_regexp: bool,
    pub(super) line_regexp: bool,
    pub(super) fixed_strings: bool,
    pub(super) extended_regexp: bool,
    pub(super) recursive: bool,
    pub(super) no_messages: bool,
    pub(super) text: bool,
    pub(super) max_count: Option<usize>,
    pub(super) patterns: Vec<String>,
    pub(super) pattern_files: Vec<String>,
    pub(super) targets: Vec<String>,
}

impl GrepOptions {
    pub(super) fn has_pattern_source(&self) -> bool {
        !self.patterns.is_empty() || !self.pattern_files.is_empty()
    }
}

pub(super) enum GrepPending {
    Pattern,
    PatternFile,
    MaxCount,
}

#[derive(Default)]
pub(super) struct GrepReport {
    pub(super) groups: Vec<GrepGroup>,
    pub(super) notes: Vec<String>,
}

pub(super) struct GrepGroup {
    pub(super) source: String,
    pub(super) rows: Vec<GrepMatchRow>,
}

pub(super) struct GrepMatchRow {
    pub(super) line_number: usize,
    pub(super) text: String,
}

pub(super) struct GrepMatch {
    pub(super) source: String,
    pub(super) line_number: usize,
    pub(super) text: String,
}
