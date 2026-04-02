use std::{collections::BTreeMap, path::Path};

use ush_compiler::{CompiledScript, SourceMapLine, SourceMapSection};

pub fn instrument_compiled_script(origin: &Path, compiled: &CompiledScript) -> String {
    let mut out = String::new();
    let mut quote_state = ShellQuoteState::default();
    let generated_groups = compiled
        .sourcemap
        .source_index()
        .into_iter()
        .map(|line| (line.source_line, format_generated_lines(&line.generated_lines)))
        .collect::<BTreeMap<_, _>>();
    out.push_str("__ush_runtime_map_origin=");
    out.push_str(&shell_quote(&origin.display().to_string()));
    out.push('\n');
    out.push_str("__ush_runtime_map_generated=''\n");
    out.push_str("__ush_runtime_map_section=''\n");
    out.push_str("__ush_runtime_map_source_line=''\n");
    out.push_str("__ush_runtime_map_source=''\n");
    out.push_str("__ush_runtime_map_shell=''\n");
    out.push_str("__ush_runtime_map_mapped=''\n");
    out.push('\n');
    out.push_str("__ush_runtime_map_track() {\n");
    out.push_str("  __ush_runtime_map_generated=\"$1\"\n");
    out.push_str("  __ush_runtime_map_section=\"$2\"\n");
    out.push_str("  __ush_runtime_map_source_line=\"$3\"\n");
    out.push_str("  __ush_runtime_map_source=\"$4\"\n");
    out.push_str("  __ush_runtime_map_shell=\"$5\"\n");
    out.push_str("  __ush_runtime_map_mapped=\"$6\"\n");
    out.push_str("}\n");
    out.push('\n');
    out.push_str("__ush_runtime_map_report() {\n");
    out.push_str("  __ush_runtime_map_status=\"$1\"\n");
    out.push_str("  [ \"$__ush_runtime_map_status\" -eq 0 ] && return\n");
    out.push_str("  if [ -n \"$__ush_runtime_map_source_line\" ]; then\n");
    out.push_str(
        "    printf '\\nush runtime map: %s:%s\\n' \"$__ush_runtime_map_origin\" \"$__ush_runtime_map_source_line\" >&2\n",
    );
    out.push_str(
        "    printf '  section: %s\\n' \"$__ush_runtime_map_section\" >&2\n",
    );
    out.push_str(
        "    printf '  shell  : G%04d | %s\\n' \"$__ush_runtime_map_generated\" \"$__ush_runtime_map_shell\" >&2\n",
    );
    out.push_str("    printf '  source : %s\\n' \"$__ush_runtime_map_source\" >&2\n");
    out.push_str("    printf '  mapped : %s\\n' \"$__ush_runtime_map_mapped\" >&2\n");
    out.push_str("  elif [ -n \"$__ush_runtime_map_generated\" ]; then\n");
    out.push_str("    printf '\\nush runtime map: %s\\n' \"$__ush_runtime_map_origin\" >&2\n");
    out.push_str(
        "    printf '  section: %s\\n' \"$__ush_runtime_map_section\" >&2\n",
    );
    out.push_str(
        "    printf '  shell  : G%04d | %s\\n' \"$__ush_runtime_map_generated\" \"$__ush_runtime_map_shell\" >&2\n",
    );
    out.push_str("    printf '  source : (no direct source mapping)\\n' >&2\n");
    out.push_str("  fi\n");
    out.push_str("}\n");
    out.push('\n');
    out.push_str("trap '__ush_runtime_map_report \"$?\"' 0\n");
    out.push('\n');

    for line in &compiled.sourcemap.lines {
        let started_inside_multiline_literal = quote_state.is_open();
        quote_state.observe(&line.generated_text);
        let touches_multiline_literal = started_inside_multiline_literal || quote_state.is_open();

        if line.section == SourceMapSection::UserCode
            && !touches_multiline_literal
            && should_inline_track(&line.generated_text)
        {
            let mapped = line
                .source_line
                .and_then(|source_line| generated_groups.get(&source_line))
                .map(String::as_str)
                .unwrap_or("");
            append_tracking_prefix(&mut out, line, mapped);
            out.push_str("; ");
        }
        out.push_str(&line.generated_text);
        out.push('\n');
    }
    out
}

fn append_tracking_prefix(out: &mut String, line: &SourceMapLine, mapped: &str) {
    out.push_str("__ush_runtime_map_track ");
    out.push_str(&shell_quote(&line.generated_line.to_string()));
    out.push(' ');
    out.push_str(&shell_quote(line.section.label()));
    out.push(' ');
    out.push_str(&shell_quote(
        &line
            .source_line
            .map(|value| value.to_string())
            .unwrap_or_default(),
    ));
    out.push(' ');
    out.push_str(&shell_quote(line.source_text.as_deref().unwrap_or("")));
    out.push(' ');
    out.push_str(&shell_quote(&line.generated_text));
    out.push(' ');
    out.push_str(&shell_quote(mapped));
}

fn should_inline_track(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }
    if matches!(
        trimmed,
        "then" | "do" | "else" | "fi" | "done" | "esac" | "}" | ";;"
    ) {
        return false;
    }
    if trimmed.starts_with("elif ") || trimmed == "elif" {
        return false;
    }
    if trimmed.starts_with('}') {
        return false;
    }
    if trimmed.contains(')') && trimmed.ends_with(";;") {
        return false;
    }
    if trimmed.ends_with(')') && !trimmed.contains('(') {
        return false;
    }
    true
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn format_generated_lines(lines: &[usize]) -> String {
    lines
        .iter()
        .map(|line| format!("G{line:04}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Clone, Copy, Debug, Default)]
struct ShellQuoteState {
    in_single: bool,
    in_double: bool,
    escaped: bool,
}

impl ShellQuoteState {
    fn is_open(self) -> bool {
        self.in_single || self.in_double
    }

    fn observe(&mut self, line: &str) {
        for ch in line.chars() {
            if self.in_single {
                if ch == '\'' {
                    self.in_single = false;
                }
                continue;
            }

            if self.escaped {
                self.escaped = false;
                continue;
            }

            match ch {
                '\\' if self.in_double => self.escaped = true,
                '"' if self.in_double => self.in_double = false,
                '"' => self.in_double = true,
                '\'' => self.in_single = true,
                _ => {}
            }
        }
        self.escaped = false;
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::{format_generated_lines, instrument_compiled_script, should_inline_track};
    use ush_compiler::UshCompiler;

    #[test]
    fn instrumented_script_reports_mapped_source_lines() {
        let compiled = UshCompiler
            .compile_source_with_sourcemap("print \"hello\"\n")
            .expect("compile");

        let script = compact_runtime_snapshot(&instrument_compiled_script("example.ush".as_ref(), &compiled));

        assert_eq!(script, include_str!("fixtures/runtime_diagnostics_simple.sh"));
    }

    #[test]
    fn control_join_lines_are_not_instrumented() {
        assert!(!should_inline_track("}; do"));
        assert!(!should_inline_track("}; then"));
        assert!(!should_inline_track("} && {"));
        assert!(!should_inline_track("done"));
        assert!(!should_inline_track("'0') value='ok'; ;;"));
        assert!(!should_inline_track("'--name') shift; value=\"$1\"; ;;"));
        assert!(should_inline_track("[ \"$(printf '%s' true)\" = 'true' ]"));
        assert!(should_inline_track("count=$((count + 1))"));
    }

    #[test]
    fn multiline_shell_literals_are_left_uninstrumented() {
        let compiled = UshCompiler
            .compile_source_with_sourcemap(
                "let article = \"\"\"\n  <article>\n    hello\n  </article>\n\"\"\"\nprint article\n",
            )
            .expect("compile");

        let script =
            compact_runtime_snapshot(&instrument_compiled_script("example.ush".as_ref(), &compiled));
        assert_eq!(script, include_str!("fixtures/runtime_diagnostics_multiline.sh"));
    }

    #[test]
    fn generated_line_groups_render_as_shell_line_spans() {
        assert_eq!(format_generated_lines(&[7]), "G0007");
        assert_eq!(format_generated_lines(&[7, 8, 11]), "G0007, G0008, G0011");
    }

    fn compact_runtime_snapshot(script: &str) -> String {
        let lines = script.lines().collect::<Vec<_>>();
        let shell_start = lines
            .iter()
            .position(|line| *line == "#!/bin/sh")
            .expect("shell start");
        let tail_start = lines.len().saturating_sub(12);
        let mut out = String::new();
        let _ = writeln!(out, "[runtime prelude]");
        out.push_str(&lines[..shell_start].join("\n"));
        out.push('\n');
        let _ = writeln!(out);
        let _ = writeln!(out, "[tail]");
        out.push_str(&lines[tail_start..].join("\n"));
        out.push('\n');
        out
    }
}
