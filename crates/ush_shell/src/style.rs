use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};

use crate::helpers::ValueStream;

pub fn render_ls(cwd: &Path, args: &[String]) -> Result<Option<ValueStream>> {
    let mut show_hidden = false;
    let mut targets = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-a" | "--all" => show_hidden = true,
            "-l" | "--long" => {}
            _ if arg.starts_with('-') => return Ok(None),
            _ => targets.push(arg.clone()),
        }
    }

    if targets.is_empty() {
        targets.push(".".to_string());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["name", "kind", "size", "modified"]);

    for target in targets {
        let path = normalize_path(cwd, &target);
        let mut entries = ls_entries(&path, show_hidden)
            .with_context(|| format!("failed to read {}", path.display()))?;
        entries.sort_by(|left, right| left.0.cmp(&right.0));

        for (file_name, entry_path) in entries {
            let metadata = entry_path.metadata()?;
            let kind = if metadata.is_dir() {
                ("dir", Color::Blue)
            } else if metadata.permissions().mode() & 0o111 != 0 {
                ("exec", Color::Green)
            } else {
                ("file", Color::White)
            };

            let modified = metadata
                .modified()
                .ok()
                .map(|time| {
                    DateTime::<Local>::from(time)
                        .format("%Y-%m-%d %H:%M")
                        .to_string()
                })
                .unwrap_or_else(|| "-".to_string());

            table.add_row(vec![
                Cell::new(file_name).fg(kind.1),
                Cell::new(kind.0).fg(kind.1),
                Cell::new(metadata.len()),
                Cell::new(modified),
            ]);
        }
    }

    Ok(Some(ValueStream::Text(format!("{table}\n"))))
}

fn ls_entries(path: &Path, show_hidden: bool) -> Result<Vec<(String, PathBuf)>> {
    if path.is_dir() {
        let mut entries = fs::read_dir(path)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter_map(|entry| {
                let file_name = entry.file_name().to_string_lossy().to_string();
                (!file_name.starts_with('.') || show_hidden).then_some((file_name, entry.path()))
            })
            .collect::<Vec<_>>();
        if show_hidden {
            entries.push((".".to_string(), path.to_path_buf()));
            entries.push((
                "..".to_string(),
                path.parent().unwrap_or(path).to_path_buf(),
            ));
        }
        return Ok(entries);
    }

    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());
    Ok(vec![(file_name, path.to_path_buf())])
}

pub fn render_cat(cwd: &Path, args: &[String], input: &ValueStream) -> Result<Option<ValueStream>> {
    let mut numbered = true;
    let mut targets = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-n" => numbered = true,
            _ if arg.starts_with('-') => return Ok(None),
            _ => targets.push(arg.clone()),
        }
    }

    let mut buffer = String::new();

    if targets.is_empty() {
        let text = input.to_text()?;
        append_numbered(&mut buffer, None, &text, numbered);
        return Ok(Some(ValueStream::Text(buffer)));
    }

    for target in targets {
        let path = normalize_path(cwd, &target);
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        append_numbered(&mut buffer, Some(&path), &text, numbered);
    }

    Ok(Some(ValueStream::Text(buffer)))
}

pub fn render_ps(args: &[String]) -> Result<Option<ValueStream>> {
    if !args.is_empty() {
        return Ok(None);
    }

    let output = Command::new("ps")
        .args(["-eo", "pid,ppid,stat,%cpu,%mem,comm"])
        .output()
        .context("failed to run ps")?;
    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut lines = stdout.lines();
    let _ = lines.next();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["pid", "ppid", "stat", "cpu", "mem", "command"]);

    for line in lines {
        let columns = line.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 6 {
            continue;
        }
        table.add_row(vec![
            Cell::new(columns[0]).fg(Color::Cyan),
            Cell::new(columns[1]),
            Cell::new(columns[2]).fg(Color::Yellow),
            Cell::new(columns[3]),
            Cell::new(columns[4]),
            Cell::new(columns[5..].join(" ")),
        ]);
    }

    Ok(Some(ValueStream::Text(format!("{table}\n"))))
}

pub fn render_kill(args: &[String]) -> Result<Option<ValueStream>> {
    if args.is_empty() {
        return Ok(None);
    }

    let mut signal = "TERM".to_string();
    let mut pid = None::<String>;
    for arg in args {
        if let Some(value) = arg.strip_prefix('-') {
            signal = value.to_string();
        } else {
            pid = Some(arg.clone());
        }
    }

    let Some(pid) = pid else {
        return Ok(None);
    };

    let output = Command::new("kill")
        .args([format!("-{signal}"), pid.clone()])
        .output()
        .context("failed to run kill")?;
    if !output.status.success() {
        return Ok(None);
    }

    Ok(Some(ValueStream::Text(format!(
        "sent SIG{} to {}\n",
        signal.to_uppercase(),
        pid
    ))))
}

fn append_numbered(buffer: &mut String, path: Option<&Path>, text: &str, numbered: bool) {
    if let Some(path) = path {
        buffer.push_str(&format!(
            "\u{1b}[1;36m==> {} <==\u{1b}[0m\n",
            path.display()
        ));
    }

    for (index, line) in text.lines().enumerate() {
        if numbered {
            buffer.push_str(&format!("\u{1b}[2m{:>4}\u{1b}[0m {}\n", index + 1, line));
        } else {
            buffer.push_str(line);
            buffer.push('\n');
        }
    }
}

fn normalize_path(cwd: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}
