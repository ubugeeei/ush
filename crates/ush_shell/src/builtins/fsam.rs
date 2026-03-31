use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow, bail};
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};

use crate::{Shell, ValueStream};

#[derive(Clone)]
struct SummaryRow {
    path: String,
    lines: usize,
    bytes: u64,
}

impl Shell {
    pub(super) fn handle_fsam(&self, args: &[String]) -> Result<(ValueStream, i32)> {
        if args.is_empty() {
            bail!("fsam requires at least one glob or path");
        }

        let rows = summarize(&self.cwd, args)?;
        if rows.is_empty() {
            bail!("fsam: no files matched");
        }

        let total_lines = rows.iter().map(|row| row.lines).sum::<usize>();
        let total_bytes = rows.iter().map(|row| row.bytes).sum::<u64>();
        let text = if self.options.stylish {
            render_table(&rows, total_lines, total_bytes)
        } else {
            render_plain(&rows, total_lines, total_bytes)
        };
        Ok((ValueStream::Text(text), 0))
    }
}

fn summarize(cwd: &Path, args: &[String]) -> Result<Vec<SummaryRow>> {
    let mut files = BTreeSet::new();
    for arg in args {
        collect_matches(cwd, arg, &mut files)?;
    }

    let mut rows = Vec::new();
    for path in files {
        let bytes =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        rows.push(SummaryRow {
            path: display_path(cwd, &path),
            lines: count_lines(&bytes),
            bytes: bytes.len() as u64,
        });
    }
    Ok(rows)
}

fn collect_matches(cwd: &Path, raw: &str, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    if contains_glob(raw) {
        let mut matched = false;
        let pattern = glob_pattern(cwd, raw);
        for entry in glob::glob(&pattern)? {
            let path = normalize(cwd, &entry?);
            if path.is_file() {
                matched = true;
                files.insert(path);
            }
        }
        if !matched {
            bail!("fsam: no files matched `{raw}`");
        }
        return Ok(());
    }

    let path = normalize(cwd, Path::new(raw));
    if !path.exists() {
        bail!("fsam: path not found: {raw}");
    }
    if path.is_dir() {
        bail!("fsam: directories are not supported directly, use a glob: {raw}/**/*");
    }
    files.insert(path);
    Ok(())
}

fn glob_pattern(cwd: &Path, raw: &str) -> String {
    let path = Path::new(raw);
    if path.is_absolute() {
        raw.to_string()
    } else {
        cwd.join(path).display().to_string()
    }
}

fn normalize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn display_path(cwd: &Path, path: &Path) -> String {
    path.strip_prefix(cwd)
        .map(|value| value.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

fn count_lines(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }
    let line_breaks = bytes.iter().filter(|byte| **byte == b'\n').count();
    line_breaks + usize::from(*bytes.last().unwrap_or(&b'\n') != b'\n')
}

fn render_plain(rows: &[SummaryRow], total_lines: usize, total_bytes: u64) -> String {
    let mut out = String::from("lines\tbytes\tpath\n");
    for row in rows {
        out.push_str(&format!("{}\t{}\t{}\n", row.lines, row.bytes, row.path));
    }
    out.push_str(&format!(
        "{}\t{}\tTOTAL ({} files)\n",
        total_lines,
        total_bytes,
        rows.len()
    ));
    out
}

fn render_table(rows: &[SummaryRow], total_lines: usize, total_bytes: u64) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["path", "lines", "bytes"]);
    for row in rows {
        table.add_row(vec![
            Cell::new(&row.path),
            Cell::new(row.lines),
            Cell::new(row.bytes),
        ]);
    }
    table.add_row(vec![
        Cell::new(format!("TOTAL ({} files)", rows.len())),
        Cell::new(total_lines),
        Cell::new(total_bytes),
    ]);
    format!("{table}\n")
}

fn contains_glob(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}

trait ContextExt<T> {
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T>;
}

impl<T> ContextExt<T> for std::result::Result<T, std::io::Error> {
    fn with_context(self, f: impl FnOnce() -> String) -> Result<T> {
        self.map_err(|error| anyhow!("{}: {error}", f()))
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tempfile::tempdir;

    use super::{count_lines, summarize};

    #[test]
    fn counts_lines_with_and_without_trailing_newline() {
        assert_eq!(count_lines(b""), 0);
        assert_eq!(count_lines(b"one\n"), 1);
        assert_eq!(count_lines(b"one\ntwo"), 2);
    }

    #[test]
    fn summarizes_glob_matches_and_sorts_paths() {
        let dir = tempdir().expect("tempdir");
        let cwd = dir.path().to_path_buf();
        fs::write(cwd.join("b.txt"), "b\n").expect("write");
        fs::write(cwd.join("a.txt"), "a\nx\n").expect("write");

        let rows = summarize(&cwd, &[String::from("*.txt")]).expect("summarize");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].path, PathBuf::from("a.txt").display().to_string());
        assert_eq!(rows[0].lines, 2);
        assert_eq!(rows[1].path, PathBuf::from("b.txt").display().to_string());
        assert_eq!(rows[1].lines, 1);
    }
}
