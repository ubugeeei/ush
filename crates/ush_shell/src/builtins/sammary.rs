mod report;

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use self::report::{
    SummaryRow, count_lines, is_lock_file, render_plain, render_table, summarize_types, type_name,
};
use crate::{Shell, ValueStream};

struct SammaryOptions {
    include_lock: bool,
    targets: Vec<String>,
}

impl Shell {
    pub(super) fn handle_sammary(&self, args: &[String]) -> Result<(ValueStream, i32)> {
        let options = parse_args(args)?;
        let rows = summarize(&self.cwd, &options.targets, options.include_lock)?;
        if rows.is_empty() {
            bail!("sammary: no files matched");
        }

        let total_lines = rows.iter().map(|row| row.lines).sum::<usize>();
        let total_bytes = rows.iter().map(|row| row.bytes).sum::<u64>();
        let types = summarize_types(&rows);
        let text = if self.options.stylish {
            render_table(&rows, &types, total_lines, total_bytes)
        } else {
            render_plain(&rows, &types, total_lines, total_bytes)
        };
        Ok((ValueStream::Text(text), 0))
    }
}

fn parse_args(args: &[String]) -> Result<SammaryOptions> {
    let mut include_lock = false;
    let mut targets = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--include-lock" => include_lock = true,
            _ if arg.starts_with('-') => bail!("sammary: unsupported flag: {arg}"),
            _ => targets.push(arg.clone()),
        }
    }
    if targets.is_empty() {
        bail!("sammary requires at least one glob or path");
    }
    Ok(SammaryOptions {
        include_lock,
        targets,
    })
}

fn summarize(cwd: &Path, args: &[String], include_lock: bool) -> Result<Vec<SummaryRow>> {
    let mut files = BTreeSet::new();
    for arg in args {
        collect_matches(cwd, arg, &mut files)?;
    }

    files
        .into_iter()
        .filter(|path| include_lock || !is_lock_file(path))
        .map(|path| {
            let bytes =
                fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
            Ok(SummaryRow {
                path: display_path(cwd, &path),
                kind: type_name(&path),
                lines: count_lines(&bytes),
                bytes: bytes.len() as u64,
            })
        })
        .collect()
}

fn collect_matches(cwd: &Path, raw: &str, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    let matched = if contains_glob(raw) {
        glob::glob(&glob_pattern(cwd, raw))?.collect::<std::result::Result<Vec<_>, _>>()?
    } else {
        vec![normalize(cwd, Path::new(raw))]
    };

    if matched.is_empty() {
        bail!("sammary: no files matched `{raw}`");
    }

    let before = files.len();
    for path in matched {
        if !contains_glob(raw) && !path.exists() {
            bail!("sammary: path not found: {raw}");
        }
        collect_path(&path, files)?;
    }
    if files.len() == before {
        bail!("sammary: no files matched `{raw}`");
    }
    Ok(())
}

fn collect_path(path: &Path, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    if path.is_file() {
        files.insert(path.to_path_buf());
        return Ok(());
    }
    if path.is_dir() {
        for entry in
            fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))?
        {
            collect_path(&entry?.path(), files)?;
        }
    }
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

fn contains_glob(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}
