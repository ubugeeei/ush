use std::{
    fmt::{Display, Write as _},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};

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

    let mut sections = Vec::new();
    for target in targets {
        let path = normalize_path(cwd, &target);
        let mut entries = ls_entries(&path, show_hidden)
            .with_context(|| format!("failed to read {}", path.display()))?;
        entries.sort_by(|left, right| left.0.cmp(&right.0));

        let mut summary = LsSummary::default();
        let mut body = String::new();
        for (file_name, entry_path) in entries {
            let row = describe_ls_entry(&file_name, &entry_path, show_hidden)?;
            summary.observe(row.kind);
            render_ls_row(&mut body, &row);
        }
        sections.push(render_ls_section(&target, &summary, &body));
    }

    Ok(Some(ValueStream::Text(sections.join("\n"))))
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

    for (index, target) in targets.into_iter().enumerate() {
        if index > 0 {
            buffer.push('\n');
        }
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

    let mut rows = Vec::new();
    for line in lines {
        let columns = line.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 6 {
            continue;
        }
        rows.push(PsRow {
            pid: columns[0].to_string(),
            ppid: columns[1].to_string(),
            stat: columns[2].to_string(),
            cpu: columns[3].to_string(),
            mem: columns[4].to_string(),
            command: columns[5..].join(" "),
        });
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "ps"),
        dim(pluralize(rows.len(), "process", "processes"))
    );
    for row in rows {
        render_ps_row(&mut out, &row);
    }

    Ok(Some(ValueStream::Text(out)))
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
        "{} {} {}\n",
        paint(BLUE_BOLD, "kill"),
        badge(format!("SIG{}", signal.to_uppercase()), YELLOW_BOLD),
        paint(CYAN_BOLD, pid)
    ))))
}

pub fn render_git(cwd: &Path, args: &[String]) -> Result<Option<ValueStream>> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Ok(None);
    };

    match subcommand.as_str() {
        "status" => render_git_status(cwd, rest),
        "branch" => render_git_branch(cwd, rest),
        "log" => render_git_log(cwd, rest),
        _ => Ok(None),
    }
}

fn append_numbered(buffer: &mut String, path: Option<&Path>, text: &str, numbered: bool) {
    if let Some(path) = path {
        let _ = writeln!(
            buffer,
            "{} {}",
            paint(BLUE_BOLD, "cat"),
            paint(CYAN_BOLD, path.display())
        );
        let _ = writeln!(
            buffer,
            "{}",
            dim(format!(
                "{}, {}",
                pluralize(count_display_lines(text), "line", "lines"),
                human_bytes(text.len() as u64)
            ))
        );
    }

    if text.is_empty() {
        if path.is_some() {
            let _ = writeln!(buffer, "{}", dim("(empty)"));
        }
        return;
    }

    let line_count = count_display_lines(text);
    let width = line_count.to_string().len();
    for (index, chunk) in text.split_inclusive('\n').enumerate() {
        let line = chunk.strip_suffix('\n').unwrap_or(chunk);
        if numbered {
            let _ = write!(
                buffer,
                "{} {} {}",
                dim(format!("{:>width$}", index + 1, width = width)),
                paint(CYAN_BOLD, "|"),
                line
            );
        } else {
            buffer.push_str(line);
        }
        buffer.push('\n');
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

const RESET: &str = "\u{1b}[0m";
const BOLD: &str = "\u{1b}[1m";
const DIM: &str = "\u{1b}[2m";
const BLUE_BOLD: &str = "\u{1b}[1;34m";
const CYAN_BOLD: &str = "\u{1b}[1;36m";
const GREEN_BOLD: &str = "\u{1b}[1;32m";
const YELLOW_BOLD: &str = "\u{1b}[1;33m";
const MAGENTA_BOLD: &str = "\u{1b}[1;35m";
const RED_BOLD: &str = "\u{1b}[1;31m";

#[derive(Clone, Copy)]
enum EntryKind {
    Dir,
    Exec,
    File,
    Link,
}

impl EntryKind {
    fn label(self) -> &'static str {
        match self {
            Self::Dir => "dir",
            Self::Exec => "exec",
            Self::File => "file",
            Self::Link => "link",
        }
    }

    fn style(self) -> &'static str {
        match self {
            Self::Dir => BLUE_BOLD,
            Self::Exec => GREEN_BOLD,
            Self::File => BOLD,
            Self::Link => MAGENTA_BOLD,
        }
    }
}

#[derive(Default)]
struct LsSummary {
    dirs: usize,
    execs: usize,
    files: usize,
    links: usize,
}

impl LsSummary {
    fn observe(&mut self, kind: EntryKind) {
        match kind {
            EntryKind::Dir => self.dirs += 1,
            EntryKind::Exec => self.execs += 1,
            EntryKind::File => self.files += 1,
            EntryKind::Link => self.links += 1,
        }
    }
}

struct LsRow {
    display_name: String,
    kind: EntryKind,
    details: Vec<String>,
}

struct PsRow {
    pid: String,
    ppid: String,
    stat: String,
    cpu: String,
    mem: String,
    command: String,
}

struct GitStatusHeader {
    branch: String,
    details: Vec<String>,
}

struct GitStatusRow {
    path: String,
    original_path: Option<String>,
    badges: Vec<String>,
    style: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GitRefScope {
    Local,
    Remote,
}

struct GitBranchRow {
    scope: GitRefScope,
    name: String,
    current: bool,
    upstream: Option<String>,
    commit: String,
    date: String,
    subject: String,
}

struct GitLogRow {
    commit: String,
    date: String,
    author: String,
    refs: Vec<String>,
    subject: String,
}

pub(crate) fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit_index = 0usize;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if value >= 10.0 {
        format!("{value:.0} {}", UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}

pub(crate) fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

pub(crate) fn paint(style: &str, value: impl Display) -> String {
    format!("{style}{value}{RESET}")
}

pub(crate) fn dim(value: impl Display) -> String {
    paint(DIM, value)
}

pub(crate) fn badge(value: impl Display, style: &str) -> String {
    format!("{style}[{value}]{RESET}")
}

fn describe_ls_entry(file_name: &str, entry_path: &Path, show_hidden: bool) -> Result<LsRow> {
    let metadata = fs::symlink_metadata(entry_path)?;
    let file_type = metadata.file_type();
    let modified = metadata
        .modified()
        .ok()
        .map(|time| {
            DateTime::<Local>::from(time)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "-".to_string());

    let (kind, display_name) = if file_type.is_symlink() {
        (EntryKind::Link, file_name.to_string())
    } else if metadata.is_dir() {
        (EntryKind::Dir, format!("{file_name}/"))
    } else if metadata.permissions().mode() & 0o111 != 0 {
        (EntryKind::Exec, file_name.to_string())
    } else {
        (EntryKind::File, file_name.to_string())
    };

    let mut details = Vec::new();
    match kind {
        EntryKind::Dir => details.push(pluralize(
            count_visible_children(entry_path, show_hidden)?,
            "item",
            "items",
        )),
        EntryKind::Link => {
            let target = fs::read_link(entry_path)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|_| "?".to_string());
            details.push(format!("-> {target}"));
        }
        EntryKind::Exec | EntryKind::File => details.push(human_bytes(metadata.len())),
    }
    details.push(format!("updated {modified}"));

    Ok(LsRow {
        display_name,
        kind,
        details,
    })
}

fn count_visible_children(path: &Path, show_hidden: bool) -> Result<usize> {
    Ok(fs::read_dir(path)?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|entry| show_hidden || !entry.file_name().to_string_lossy().starts_with('.'))
        .count())
}

fn render_ls_section(target: &str, summary: &LsSummary, body: &str) -> String {
    let mut meta = vec![pluralize(
        summary.dirs + summary.execs + summary.files + summary.links,
        "entry",
        "entries",
    )];
    if summary.dirs > 0 {
        meta.push(pluralize(summary.dirs, "dir", "dirs"));
    }
    if summary.execs > 0 {
        meta.push(pluralize(summary.execs, "exec", "execs"));
    }
    if summary.files > 0 {
        meta.push(pluralize(summary.files, "file", "files"));
    }
    if summary.links > 0 {
        meta.push(pluralize(summary.links, "link", "links"));
    }

    format!(
        "{} {}\n{}\n{}",
        paint(BLUE_BOLD, "ls"),
        paint(CYAN_BOLD, target),
        dim(meta.join(", ")),
        body
    )
}

fn render_ls_row(out: &mut String, row: &LsRow) {
    let _ = writeln!(
        out,
        "{} {} {}",
        paint(row.kind.style(), &row.display_name),
        badge(row.kind.label(), row.kind.style()),
        dim(row.details.join(", "))
    );
}

fn render_ps_row(out: &mut String, row: &PsRow) {
    let display_name = Path::new(&row.command)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(&row.command);
    let _ = writeln!(
        out,
        "{} {} {} {}",
        paint(CYAN_BOLD, display_name),
        badge(format!("pid {}", row.pid), BLUE_BOLD),
        badge(row.stat.as_str(), YELLOW_BOLD),
        dim(format!(
            "ppid {}, cpu {}%, mem {}%",
            row.ppid, row.cpu, row.mem
        ))
    );
    if display_name != row.command {
        let _ = writeln!(out, "  {}", dim(&row.command));
    }
}

fn count_display_lines(text: &str) -> usize {
    text.split_inclusive('\n').count()
}

fn render_git_status(cwd: &Path, args: &[String]) -> Result<Option<ValueStream>> {
    let Some(pathspecs) = parse_git_status_args(args) else {
        return Ok(None);
    };

    let mut command_args = vec![
        "status".to_string(),
        "--porcelain=v1".to_string(),
        "--branch".to_string(),
        "-z".to_string(),
        "--untracked-files=all".to_string(),
    ];
    if !pathspecs.is_empty() {
        command_args.push("--".to_string());
        command_args.extend(pathspecs);
    }

    let Some(stdout) = git_capture(cwd, &command_args)? else {
        return Ok(None);
    };

    let mut records = stdout.split('\0').filter(|record| !record.is_empty());
    let mut header = None;
    let mut rows = Vec::new();

    if let Some(record) = records.next() {
        if let Some(branch) = record.strip_prefix("## ") {
            header = Some(parse_git_status_header(branch));
        } else {
            parse_git_status_record(record, &mut records, &mut rows);
        }
    }
    while let Some(record) = records.next() {
        parse_git_status_record(record, &mut records, &mut rows);
    }

    let branch = header.unwrap_or_else(|| GitStatusHeader {
        branch: "repository".to_string(),
        details: Vec::new(),
    });

    let mut meta = branch.details;
    if rows.is_empty() {
        meta.push("working tree clean".to_string());
    } else {
        meta.push(pluralize(rows.len(), "changed path", "changed paths"));
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {} {}",
        paint(BLUE_BOLD, "git"),
        paint(CYAN_BOLD, "status"),
        paint(MAGENTA_BOLD, &branch.branch)
    );
    let _ = writeln!(out, "{}", dim(meta.join(", ")));

    if rows.is_empty() {
        let _ = writeln!(
            out,
            "{} {}",
            badge("clean", GREEN_BOLD),
            dim("nothing to commit")
        );
    } else {
        for row in rows {
            render_git_status_row(&mut out, &row);
        }
    }

    Ok(Some(ValueStream::Text(out)))
}

fn render_git_branch(cwd: &Path, args: &[String]) -> Result<Option<ValueStream>> {
    let Some((include_local, include_remote)) = parse_git_branch_args(args) else {
        return Ok(None);
    };

    let mut command_args = vec![
        "for-each-ref".to_string(),
        "--format=%(HEAD)\t%(refname)\t%(refname:short)\t%(upstream:short)\t%(objectname:short)\t%(committerdate:short)\t%(subject)".to_string(),
    ];
    if include_local {
        command_args.push("refs/heads".to_string());
    }
    if include_remote {
        command_args.push("refs/remotes".to_string());
    }

    let Some(stdout) = git_capture(cwd, &command_args)? else {
        return Ok(None);
    };

    let mut local = Vec::new();
    let mut remote = Vec::new();
    for line in stdout.lines() {
        let columns = line.splitn(7, '\t').collect::<Vec<_>>();
        if columns.len() < 7 {
            continue;
        }

        let full_ref = columns[1];
        let name = columns[2].trim();
        if name.is_empty() || name.ends_with("/HEAD") {
            continue;
        }

        let row = GitBranchRow {
            scope: if full_ref.starts_with("refs/remotes/") {
                GitRefScope::Remote
            } else {
                GitRefScope::Local
            },
            name: name.to_string(),
            current: columns[0].trim() == "*",
            upstream: (!columns[3].is_empty()).then_some(columns[3].to_string()),
            commit: columns[4].to_string(),
            date: columns[5].to_string(),
            subject: columns[6].to_string(),
        };

        match row.scope {
            GitRefScope::Local => local.push(row),
            GitRefScope::Remote => remote.push(row),
        }
    }

    let total = local.len() + remote.len();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "git"),
        paint(CYAN_BOLD, "branch")
    );
    let _ = writeln!(out, "{}", dim(pluralize(total, "branch", "branches")));

    let grouped = !local.is_empty() && !remote.is_empty();
    if !local.is_empty() {
        if grouped {
            let _ = writeln!(out, "{}", dim("local"));
        }
        for row in &local {
            render_git_branch_row(&mut out, row);
        }
    }
    if !remote.is_empty() {
        if !local.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(out, "{}", dim("remote"));
        for row in &remote {
            render_git_branch_row(&mut out, row);
        }
    }

    Ok(Some(ValueStream::Text(out)))
}

fn render_git_log(cwd: &Path, args: &[String]) -> Result<Option<ValueStream>> {
    let Some((max_count, include_all)) = parse_git_log_args(args) else {
        return Ok(None);
    };

    let mut command_args = vec![
        "log".to_string(),
        "--decorate=short".to_string(),
        "--date=short".to_string(),
        format!("-n{max_count}"),
        "--pretty=format:%h%x1f%ad%x1f%an%x1f%d%x1f%s".to_string(),
    ];
    if include_all {
        command_args.push("--all".to_string());
    }

    let Some(stdout) = git_capture(cwd, &command_args)? else {
        return Ok(None);
    };

    let rows = stdout
        .lines()
        .filter_map(|line| {
            let columns = line.split('\u{1f}').collect::<Vec<_>>();
            (columns.len() >= 5).then(|| GitLogRow {
                commit: columns[0].to_string(),
                date: columns[1].to_string(),
                author: columns[2].to_string(),
                refs: format_git_refs(columns[3]),
                subject: columns[4].to_string(),
            })
        })
        .collect::<Vec<_>>();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "git"),
        paint(CYAN_BOLD, "log")
    );
    let _ = writeln!(out, "{}", dim(pluralize(rows.len(), "commit", "commits")));
    for row in &rows {
        render_git_log_row(&mut out, row);
    }

    Ok(Some(ValueStream::Text(out)))
}

fn parse_git_status_args(args: &[String]) -> Option<Vec<String>> {
    let mut pathspecs = Vec::new();
    let mut force_paths = false;

    for arg in args {
        match arg.as_str() {
            "-s" | "--short" | "-b" | "--branch" | "--porcelain" | "--porcelain=v1" => {}
            "--" => force_paths = true,
            _ if force_paths || !arg.starts_with('-') => pathspecs.push(arg.clone()),
            _ => return None,
        }
    }

    Some(pathspecs)
}

fn parse_git_branch_args(args: &[String]) -> Option<(bool, bool)> {
    let mut include_local = true;
    let mut include_remote = false;

    for arg in args {
        match arg.as_str() {
            "-a" | "--all" => include_remote = true,
            "-r" | "--remotes" => {
                include_local = false;
                include_remote = true;
            }
            "--list" => {}
            _ => return None,
        }
    }

    Some((include_local, include_remote))
}

fn parse_git_log_args(args: &[String]) -> Option<(usize, bool)> {
    let mut max_count = 12usize;
    let mut include_all = false;
    let mut pending_count = false;

    for arg in args {
        if pending_count {
            max_count = arg.parse().ok()?;
            pending_count = false;
            continue;
        }

        match arg.as_str() {
            "--oneline" => {}
            "--all" => include_all = true,
            "-n" | "--max-count" => pending_count = true,
            _ if arg.starts_with("--max-count=") => {
                max_count = arg.split_once('=')?.1.parse().ok()?;
            }
            _ if arg.starts_with("-n") && arg.len() > 2 => {
                max_count = arg[2..].parse().ok()?;
            }
            _ if arg.starts_with('-') && arg[1..].chars().all(|ch| ch.is_ascii_digit()) => {
                max_count = arg[1..].parse().ok()?;
            }
            _ => return None,
        }
    }

    (!pending_count).then_some((max_count.max(1), include_all))
}

fn git_capture(cwd: &Path, args: &[String]) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .context("failed to run git")?;
    if !output.status.success() {
        return Ok(None);
    }

    Ok(Some(String::from_utf8(output.stdout).unwrap_or_else(
        |error| String::from_utf8_lossy(&error.into_bytes()).to_string(),
    )))
}

fn parse_git_status_header(value: &str) -> GitStatusHeader {
    if let Some(branch) = value.strip_prefix("No commits yet on ") {
        return GitStatusHeader {
            branch: branch.to_string(),
            details: vec!["no commits yet".to_string()],
        };
    }
    if value.starts_with("HEAD ") {
        return GitStatusHeader {
            branch: "detached HEAD".to_string(),
            details: vec![value.to_string()],
        };
    }

    let (head, tracking) = value
        .split_once(" [")
        .map(|(head, rest)| (head, Some(rest.trim_end_matches(']'))))
        .unwrap_or((value, None));
    let (branch, upstream) = head
        .split_once("...")
        .map(|(branch, upstream)| (branch, Some(upstream)))
        .unwrap_or((head, None));

    let mut details = Vec::new();
    if let Some(upstream) = upstream {
        details.push(format!("tracks {upstream}"));
    }
    if let Some(tracking) = tracking {
        details.extend(tracking.split(", ").map(str::to_string));
    }

    GitStatusHeader {
        branch: branch.to_string(),
        details,
    }
}

fn parse_git_status_record<'a, I>(record: &str, records: &mut I, rows: &mut Vec<GitStatusRow>)
where
    I: Iterator<Item = &'a str>,
{
    if record.len() < 3 {
        return;
    }

    let mut chars = record.chars();
    let index = chars.next().unwrap_or(' ');
    let worktree = chars.next().unwrap_or(' ');
    let path = record[3..].to_string();
    let original_path = if matches!(index, 'R' | 'C') {
        records.next().map(str::to_string)
    } else {
        None
    };

    rows.push(describe_git_status_row(
        index,
        worktree,
        path,
        original_path,
    ));
}

fn describe_git_status_row(
    index: char,
    worktree: char,
    path: String,
    original_path: Option<String>,
) -> GitStatusRow {
    let mut badges = Vec::new();
    let conflict = git_status_conflict(index, worktree);
    if conflict {
        badges.push(badge("conflict", RED_BOLD));
    } else if index == '?' && worktree == '?' {
        badges.push(badge("untracked", MAGENTA_BOLD));
    } else {
        if let Some(label) = git_status_label(index) {
            badges.push(badge(format!("staged {label}"), GREEN_BOLD));
        }
        if let Some(label) = git_status_label(worktree) {
            badges.push(badge(format!("unstaged {label}"), YELLOW_BOLD));
        }
    }

    if original_path.is_some() {
        badges.push(badge("renamed", MAGENTA_BOLD));
    }

    let style = if conflict {
        RED_BOLD
    } else if original_path.is_some() {
        MAGENTA_BOLD
    } else if index == '?' && worktree == '?' {
        GREEN_BOLD
    } else if matches!(index, 'D') || matches!(worktree, 'D') {
        RED_BOLD
    } else if matches!(index, 'A') || matches!(worktree, 'A') {
        GREEN_BOLD
    } else {
        CYAN_BOLD
    };

    GitStatusRow {
        path,
        original_path,
        badges,
        style,
    }
}

fn git_status_conflict(index: char, worktree: char) -> bool {
    matches!(
        (index, worktree),
        ('D', 'D') | ('A', 'U') | ('U', 'D') | ('U', 'A') | ('D', 'U') | ('A', 'A') | ('U', 'U')
    )
}

fn git_status_label(code: char) -> Option<&'static str> {
    match code {
        'M' => Some("modified"),
        'A' => Some("added"),
        'D' => Some("deleted"),
        'R' => Some("renamed"),
        'C' => Some("copied"),
        'U' => Some("updated"),
        _ => None,
    }
}

fn format_git_refs(raw: &str) -> Vec<String> {
    let trimmed = raw
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    trimmed
        .split(", ")
        .map(|value| {
            if let Some(tag) = value.strip_prefix("tag: ") {
                badge(format!("tag {tag}"), YELLOW_BOLD)
            } else if value.starts_with("HEAD") {
                badge(value, BLUE_BOLD)
            } else if value.contains('/') {
                badge(value, CYAN_BOLD)
            } else {
                badge(value, MAGENTA_BOLD)
            }
        })
        .collect()
}

fn render_git_status_row(out: &mut String, row: &GitStatusRow) {
    let mut parts = vec![paint(row.style, &row.path)];
    parts.extend(row.badges.iter().cloned());
    let _ = writeln!(out, "{}", parts.join(" "));
    if let Some(original_path) = &row.original_path {
        let _ = writeln!(out, "  {}", dim(format!("from {original_path}")));
    }
}

fn render_git_branch_row(out: &mut String, row: &GitBranchRow) {
    let style = if row.current {
        BLUE_BOLD
    } else if row.scope == GitRefScope::Remote {
        CYAN_BOLD
    } else {
        BOLD
    };

    let mut parts = vec![paint(style, &row.name)];
    if row.current {
        parts.push(badge("current", BLUE_BOLD));
    }
    if row.scope == GitRefScope::Remote {
        parts.push(badge("remote", CYAN_BOLD));
    }
    if let Some(upstream) = &row.upstream {
        parts.push(badge(upstream, MAGENTA_BOLD));
    }
    parts.push(dim(&row.commit));
    let _ = writeln!(out, "{}", parts.join(" "));

    let mut details = Vec::new();
    if !row.date.is_empty() {
        details.push(row.date.clone());
    }
    if !row.subject.is_empty() {
        details.push(row.subject.clone());
    }
    if !details.is_empty() {
        let _ = writeln!(out, "  {}", dim(details.join(" · ")));
    }
}

fn render_git_log_row(out: &mut String, row: &GitLogRow) {
    let mut parts = vec![paint(CYAN_BOLD, &row.commit)];
    parts.extend(row.refs.iter().cloned());
    parts.push(paint(BOLD, &row.subject));
    let _ = writeln!(out, "{}", parts.join(" "));
    let _ = writeln!(out, "  {}", dim(format!("{} · {}", row.date, row.author)));
}
