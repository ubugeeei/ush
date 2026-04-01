use std::{
    collections::BTreeMap,
    fmt::{Display, Write as _},
    fs,
    io::Write as _,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Local};

use crate::{commands::CommandLookup, helpers::ValueStream};

pub fn render_ls(cwd: &Path, args: &[String]) -> Result<Option<ValueStream>> {
    let Some((hidden_mode, mut targets)) = parse_ls_args(args) else {
        return Ok(None);
    };

    if targets.is_empty() {
        targets.push(".".to_string());
    }

    let mut sections = Vec::new();
    for target in targets {
        let path = normalize_path(cwd, &target);
        let mut entries = ls_entries(&path, hidden_mode)
            .with_context(|| format!("failed to read {}", path.display()))?;
        entries.sort_by(|left, right| left.0.cmp(&right.0));

        let mut summary = LsSummary::default();
        let mut body = String::new();
        for (file_name, entry_path) in entries {
            let row = describe_ls_entry(&file_name, &entry_path, hidden_mode)?;
            summary.observe(row.kind);
            render_ls_row(&mut body, &row);
        }
        sections.push(render_ls_section(&target, &summary, &body));
    }

    Ok(Some(ValueStream::Text(sections.join("\n"))))
}

fn parse_ls_args(args: &[String]) -> Option<(HiddenMode, Vec<String>)> {
    let mut hidden_mode = HiddenMode::Default;
    let mut targets = Vec::new();
    let mut force_paths = false;

    for arg in args {
        if force_paths {
            targets.push(arg.clone());
            continue;
        }

        match arg.as_str() {
            "--" => force_paths = true,
            "--all" => hidden_mode = hidden_mode.include(HiddenMode::All),
            "--almost-all" => hidden_mode = hidden_mode.include(HiddenMode::AlmostAll),
            "--long" | "--human-readable" | "--classify" | "--file-type" | "--color" => {}
            _ if arg.starts_with("--color=") => {}
            _ if arg.starts_with("--indicator-style=") => {
                match arg.split_once('=').map(|(_, value)| value) {
                    Some("classify" | "file-type" | "slash") => {}
                    _ => return None,
                }
            }
            _ if arg.starts_with('-') && arg.len() > 1 => {
                parse_ls_short_flags(arg, &mut hidden_mode)?
            }
            _ => targets.push(arg.clone()),
        }
    }

    Some((hidden_mode, targets))
}

fn parse_ls_short_flags(arg: &str, hidden_mode: &mut HiddenMode) -> Option<()> {
    for flag in arg[1..].chars() {
        match flag {
            'a' => *hidden_mode = hidden_mode.include(HiddenMode::All),
            'A' => *hidden_mode = hidden_mode.include(HiddenMode::AlmostAll),
            '1' | 'C' | 'F' | 'G' | 'h' | 'l' | 'm' | 'p' | 'x' => {}
            _ => return None,
        }
    }

    Some(())
}

fn ls_entries(path: &Path, hidden_mode: HiddenMode) -> Result<Vec<(String, PathBuf)>> {
    if path.is_dir() {
        let mut entries = fs::read_dir(path)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter_map(|entry| {
                let file_name = entry.file_name().to_string_lossy().to_string();
                (!file_name.starts_with('.') || hidden_mode.shows_hidden())
                    .then_some((file_name, entry.path()))
            })
            .collect::<Vec<_>>();
        if hidden_mode.shows_dot_entries() {
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

pub fn render_aliases(aliases: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "alias"),
        dim(pluralize(aliases.len(), "alias", "aliases"))
    );

    if aliases.is_empty() {
        let _ = writeln!(out, "{}", dim("(empty)"));
        return out;
    }

    let _ = writeln!(
        out,
        "{}",
        dim("shell shortcuts expanded before command lookup")
    );
    for (name, value) in aliases {
        render_alias_row(&mut out, name, value);
    }

    out
}

pub fn render_lookup(command: &str, rows: &[(String, Option<CommandLookup>)]) -> String {
    let mut alias_count = 0usize;
    let mut builtin_count = 0usize;
    let mut external_count = 0usize;
    let mut missing_count = 0usize;

    for (_, result) in rows {
        match result {
            Some(CommandLookup::Alias(_)) => alias_count += 1,
            Some(CommandLookup::Builtin) => builtin_count += 1,
            Some(CommandLookup::External(_)) => external_count += 1,
            None => missing_count += 1,
        }
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, command),
        dim(pluralize(rows.len(), "target", "targets"))
    );

    let mut meta = Vec::new();
    if alias_count > 0 {
        meta.push(pluralize(alias_count, "alias", "aliases"));
    }
    if builtin_count > 0 {
        meta.push(pluralize(builtin_count, "builtin", "builtins"));
    }
    if external_count > 0 {
        meta.push(pluralize(
            external_count,
            "external command",
            "external commands",
        ));
    }
    if missing_count > 0 {
        meta.push(pluralize(
            missing_count,
            "missing target",
            "missing targets",
        ));
    }
    if !meta.is_empty() {
        let _ = writeln!(out, "{}", dim(meta.join(", ")));
    }

    for (name, result) in rows {
        render_which_row(&mut out, name, result.as_ref());
    }

    out
}

pub fn render_history(entries: &[String], limit: Option<usize>) -> String {
    let shown = limit.unwrap_or(entries.len()).min(entries.len());
    let start = entries.len().saturating_sub(shown);

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "history"),
        dim(pluralize(entries.len(), "entry", "entries"))
    );

    let mut meta = Vec::new();
    if shown < entries.len() {
        meta.push(format!("showing latest {shown}"));
    } else {
        meta.push(format!("showing all {shown}"));
    }
    if let Some(last) = entries.last() {
        meta.push(format!("latest {}", truncate_history_entry(last, 48)));
    }
    let _ = writeln!(out, "{}", dim(meta.join(", ")));

    if shown == 0 {
        let _ = writeln!(out, "{}", dim("(empty)"));
        return out;
    }

    for (offset, entry) in entries[start..].iter().enumerate() {
        let index = start + offset + 1;
        let _ = writeln!(out, "{} {}", badge(index, CYAN_BOLD), paint(BOLD, entry));
    }

    out
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

pub fn render_diff(cwd: &Path, args: &[String]) -> Result<Option<(ValueStream, i32)>> {
    let Some(options) = parse_diff_args(args) else {
        return Ok(None);
    };

    let output = match Command::new("diff")
        .args(build_diff_command_args(&options))
        .current_dir(cwd)
        .output()
    {
        Ok(output) => output,
        Err(_) => return Ok(None),
    };

    let status = output.status.code().unwrap_or(1);
    if status > 1 {
        return Ok(None);
    }

    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_else(|error| String::from_utf8_lossy(&error.into_bytes()).to_string());

    let rendered = if status == 0 {
        render_diff_clean(&options)
    } else {
        render_diff_report(&options, &stdout)
    };

    Ok(Some((ValueStream::Text(rendered), status)))
}

pub fn render_grep(
    cwd: &Path,
    args: &[String],
    input: &ValueStream,
) -> Result<Option<(ValueStream, i32)>> {
    let Some(options) = parse_grep_args(args) else {
        return Ok(None);
    };
    if options.targets.is_empty() && input.is_empty() {
        return Ok(None);
    }

    let mut command = Command::new("grep");
    command
        .args(build_grep_command_args(&options))
        .current_dir(cwd);

    let output = match capture_command_output(&mut command, input) {
        Ok(output) => output,
        Err(_) => return Ok(None),
    };

    let status = output.status.code().unwrap_or(1);
    if status > 1 {
        return Ok(None);
    }

    let stdout = String::from_utf8(output.stdout)
        .unwrap_or_else(|error| String::from_utf8_lossy(&error.into_bytes()).to_string());
    let rendered = if status == 1 {
        render_grep_no_matches(&options)
    } else {
        render_grep_report(&options, &parse_grep_output(&stdout))
    };

    Ok(Some((ValueStream::Text(rendered), status)))
}

fn parse_diff_args(args: &[String]) -> Option<DiffOptions> {
    let mut options = DiffOptions {
        context: 3,
        ..DiffOptions::default()
    };
    let mut pending_context = false;
    let mut force_paths = false;

    for arg in args {
        if pending_context {
            options.context = arg.parse().ok()?;
            pending_context = false;
            continue;
        }

        if force_paths {
            options.targets.push(arg.clone());
            continue;
        }

        match arg.as_str() {
            "--" => force_paths = true,
            "-u" | "--unified" => {}
            "-r" | "--recursive" => options.recursive = true,
            "-N" | "--new-file" => options.new_file = true,
            "-a" | "--text" => options.text = true,
            "-w" | "--ignore-all-space" => options.ignore_all_space = true,
            "-b" | "--ignore-space-change" => options.ignore_space_change = true,
            "-B" | "--ignore-blank-lines" => options.ignore_blank_lines = true,
            "-i" | "--ignore-case" => options.ignore_case = true,
            "-U" => pending_context = true,
            _ if arg.starts_with("--unified=") => {
                options.context = arg.split_once('=')?.1.parse().ok()?;
            }
            _ if arg.starts_with('-') && arg.len() > 1 => {
                parse_diff_short_flags(arg, &mut options, &mut pending_context)?
            }
            _ => options.targets.push(arg.clone()),
        }
    }

    (!pending_context && options.targets.len() == 2).then_some(options)
}

fn parse_grep_args(args: &[String]) -> Option<GrepOptions> {
    let mut options = GrepOptions::default();
    let mut pending = None::<GrepPending>;
    let mut force_positional = false;

    for arg in args {
        if let Some(kind) = pending.take() {
            match kind {
                GrepPending::Pattern => options.patterns.push(arg.clone()),
                GrepPending::PatternFile => options.pattern_files.push(arg.clone()),
                GrepPending::MaxCount => options.max_count = Some(arg.parse().ok()?),
            }
            continue;
        }

        if force_positional {
            push_grep_positional(&mut options, arg.clone());
            continue;
        }

        match arg.as_str() {
            "--" => force_positional = true,
            "-n" | "--line-number" | "-H" | "--with-filename" | "-h" | "--no-filename" => {}
            "-i" | "--ignore-case" => options.ignore_case = true,
            "-v" | "--invert-match" => options.invert_match = true,
            "-w" | "--word-regexp" => options.word_regexp = true,
            "-x" | "--line-regexp" => options.line_regexp = true,
            "-F" | "--fixed-strings" => options.fixed_strings = true,
            "-E" | "--extended-regexp" => options.extended_regexp = true,
            "-r" | "--recursive" | "-R" | "--dereference-recursive" => options.recursive = true,
            "-s" | "--no-messages" => options.no_messages = true,
            "-a" | "--text" => options.text = true,
            "-e" | "--regexp" => pending = Some(GrepPending::Pattern),
            "-f" | "--file" => pending = Some(GrepPending::PatternFile),
            "-m" | "--max-count" => pending = Some(GrepPending::MaxCount),
            "--color" => {}
            _ if arg.starts_with("--color=") => {}
            _ if arg.starts_with("--regexp=") => {
                options.patterns.push(arg.split_once('=')?.1.to_string());
            }
            _ if arg.starts_with("--file=") => {
                options
                    .pattern_files
                    .push(arg.split_once('=')?.1.to_string());
            }
            _ if arg.starts_with("--max-count=") => {
                options.max_count = Some(arg.split_once('=')?.1.parse().ok()?);
            }
            _ if arg.starts_with("--binary-files=") => match arg.split_once('=')?.1 {
                "text" => options.text = true,
                _ => return None,
            },
            _ if arg.starts_with('-') && arg.len() > 1 => {
                parse_grep_short_flags(arg, &mut options, &mut pending)?
            }
            _ => push_grep_positional(&mut options, arg.clone()),
        }
    }

    (pending.is_none() && options.has_pattern_source()).then_some(options)
}

fn parse_grep_short_flags(
    arg: &str,
    options: &mut GrepOptions,
    pending: &mut Option<GrepPending>,
) -> Option<()> {
    let mut chars = arg[1..].chars().peekable();
    while let Some(flag) = chars.next() {
        match flag {
            'n' | 'H' | 'h' => {}
            'i' => options.ignore_case = true,
            'v' => options.invert_match = true,
            'w' => options.word_regexp = true,
            'x' => options.line_regexp = true,
            'F' => options.fixed_strings = true,
            'E' => options.extended_regexp = true,
            'r' | 'R' => options.recursive = true,
            's' => options.no_messages = true,
            'a' => options.text = true,
            'e' => {
                let rest = chars.collect::<String>();
                if rest.is_empty() {
                    *pending = Some(GrepPending::Pattern);
                } else {
                    options.patterns.push(rest);
                }
                break;
            }
            'f' => {
                let rest = chars.collect::<String>();
                if rest.is_empty() {
                    *pending = Some(GrepPending::PatternFile);
                } else {
                    options.pattern_files.push(rest);
                }
                break;
            }
            'm' => {
                let rest = chars.collect::<String>();
                if rest.is_empty() {
                    *pending = Some(GrepPending::MaxCount);
                } else {
                    options.max_count = Some(rest.parse().ok()?);
                }
                break;
            }
            _ => return None,
        }
    }

    Some(())
}

fn push_grep_positional(options: &mut GrepOptions, value: String) {
    if options.has_pattern_source() {
        options.targets.push(value);
    } else {
        options.patterns.push(value);
    }
}

fn build_grep_command_args(options: &GrepOptions) -> Vec<String> {
    let mut args = vec!["-nH".to_string()];
    if options.ignore_case {
        args.push("-i".to_string());
    }
    if options.invert_match {
        args.push("-v".to_string());
    }
    if options.word_regexp {
        args.push("-w".to_string());
    }
    if options.line_regexp {
        args.push("-x".to_string());
    }
    if options.fixed_strings {
        args.push("-F".to_string());
    }
    if options.extended_regexp {
        args.push("-E".to_string());
    }
    if options.recursive {
        args.push("-R".to_string());
    }
    if options.no_messages {
        args.push("-s".to_string());
    }
    if options.text {
        args.push("-a".to_string());
    }
    if let Some(max_count) = options.max_count {
        args.push("-m".to_string());
        args.push(max_count.to_string());
    }
    for pattern in &options.patterns {
        args.push("-e".to_string());
        args.push(pattern.clone());
    }
    for pattern_file in &options.pattern_files {
        args.push("-f".to_string());
        args.push(pattern_file.clone());
    }
    args.push("--".to_string());
    args.extend(options.targets.iter().cloned());
    args
}

fn capture_command_output(command: &mut Command, input: &ValueStream) -> Result<Output> {
    if input.is_empty() {
        return command.output().context("failed to run command");
    }

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().context("failed to run command")?;
    child
        .stdin
        .as_mut()
        .context("command stdin unavailable")?
        .write_all(&input.to_bytes()?)?;
    Ok(child.wait_with_output()?)
}

fn parse_grep_output(stdout: &str) -> GrepReport {
    let mut report = GrepReport::default();

    for line in stdout.lines() {
        let Some((source, rest)) = line.split_once(':') else {
            if !line.is_empty() {
                report.notes.push(line.to_string());
            }
            continue;
        };
        let Some((line_number, text)) = rest.split_once(':') else {
            report.notes.push(line.to_string());
            continue;
        };
        let Ok(line_number) = line_number.parse() else {
            report.notes.push(line.to_string());
            continue;
        };

        push_grep_match(
            &mut report.groups,
            GrepMatch {
                source: normalize_grep_source(source),
                line_number,
                text: text.to_string(),
            },
        );
    }

    report
}

fn push_grep_match(groups: &mut Vec<GrepGroup>, row: GrepMatch) {
    if let Some(group) = groups.last_mut() {
        if group.source == row.source {
            group.rows.push(GrepMatchRow {
                line_number: row.line_number,
                text: row.text,
            });
            return;
        }
    }

    groups.push(GrepGroup {
        source: row.source,
        rows: vec![GrepMatchRow {
            line_number: row.line_number,
            text: row.text,
        }],
    });
}

fn normalize_grep_source(source: &str) -> String {
    match source {
        "(standard input)" => "stdin".to_string(),
        _ => source.to_string(),
    }
}

fn render_grep_no_matches(options: &GrepOptions) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "grep"),
        paint(CYAN_BOLD, grep_query_label(options))
    );
    let _ = writeln!(
        out,
        "{} {}",
        badge("no matches", YELLOW_BOLD),
        dim("pattern not found")
    );
    out
}

fn render_grep_report(options: &GrepOptions, report: &GrepReport) -> String {
    let match_count = report
        .groups
        .iter()
        .map(|group| group.rows.len())
        .sum::<usize>();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint(BLUE_BOLD, "grep"),
        paint(CYAN_BOLD, grep_query_label(options))
    );

    let mut meta = vec![pluralize(match_count, "match", "matches")];
    if !report.groups.is_empty() {
        meta.push(pluralize(report.groups.len(), "source", "sources"));
    }
    let _ = writeln!(out, "{}", dim(meta.join(", ")));

    for note in &report.notes {
        let _ = writeln!(out, "{} {}", badge("note", YELLOW_BOLD), dim(note));
    }

    for (index, group) in report.groups.iter().enumerate() {
        if index > 0 || !report.notes.is_empty() {
            out.push('\n');
        }
        render_grep_group(&mut out, group);
    }

    out
}

fn grep_query_label(options: &GrepOptions) -> String {
    if options.patterns.len() == 1 && options.pattern_files.is_empty() {
        return options.patterns[0].clone();
    }
    if options.patterns.is_empty() && options.pattern_files.len() == 1 {
        return format!("patterns from {}", options.pattern_files[0]);
    }

    pluralize(
        options.patterns.len() + options.pattern_files.len(),
        "pattern",
        "patterns",
    )
}

fn parse_diff_short_flags(
    arg: &str,
    options: &mut DiffOptions,
    pending_context: &mut bool,
) -> Option<()> {
    let mut chars = arg[1..].chars().peekable();
    while let Some(flag) = chars.next() {
        match flag {
            'u' => {}
            'r' => options.recursive = true,
            'N' => options.new_file = true,
            'a' => options.text = true,
            'w' => options.ignore_all_space = true,
            'b' => options.ignore_space_change = true,
            'B' => options.ignore_blank_lines = true,
            'i' => options.ignore_case = true,
            'U' => {
                let rest = chars.collect::<String>();
                if rest.is_empty() {
                    *pending_context = true;
                } else {
                    options.context = rest.parse().ok()?;
                }
                break;
            }
            _ => return None,
        }
    }

    Some(())
}

fn build_diff_command_args(options: &DiffOptions) -> Vec<String> {
    let mut args = vec![format!("--unified={}", options.context)];
    if options.recursive {
        args.push("--recursive".to_string());
    }
    if options.new_file {
        args.push("--new-file".to_string());
    }
    if options.text {
        args.push("--text".to_string());
    }
    if options.ignore_all_space {
        args.push("--ignore-all-space".to_string());
    }
    if options.ignore_space_change {
        args.push("--ignore-space-change".to_string());
    }
    if options.ignore_blank_lines {
        args.push("--ignore-blank-lines".to_string());
    }
    if options.ignore_case {
        args.push("--ignore-case".to_string());
    }
    args.push("--".to_string());
    args.extend(options.targets.iter().cloned());
    args
}

fn render_diff_clean(options: &DiffOptions) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {} {}",
        paint(BLUE_BOLD, "diff"),
        paint(CYAN_BOLD, &options.targets[0]),
        paint(MAGENTA_BOLD, &options.targets[1])
    );
    let _ = writeln!(
        out,
        "{} {}",
        badge("same", GREEN_BOLD),
        dim("no differences")
    );
    out
}

fn render_diff_report(options: &DiffOptions, stdout: &str) -> String {
    let report = parse_unified_diff(stdout);
    let total_hunks = report
        .sections
        .iter()
        .map(|section| section.hunks.len())
        .sum::<usize>();
    let total_additions = report
        .sections
        .iter()
        .map(|section| section.additions)
        .sum::<usize>();
    let total_deletions = report
        .sections
        .iter()
        .map(|section| section.deletions)
        .sum::<usize>();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {} {}",
        paint(BLUE_BOLD, "diff"),
        paint(CYAN_BOLD, &options.targets[0]),
        paint(MAGENTA_BOLD, &options.targets[1])
    );

    let mut meta = Vec::new();
    if !report.sections.is_empty() {
        meta.push(pluralize(
            report.sections.len(),
            "changed file",
            "changed files",
        ));
    }
    if total_hunks > 0 {
        meta.push(pluralize(total_hunks, "hunk", "hunks"));
    }
    if total_additions > 0 {
        meta.push(format!("+{total_additions}"));
    }
    if total_deletions > 0 {
        meta.push(format!("-{total_deletions}"));
    }
    if meta.is_empty() {
        meta.push("differences detected".to_string());
    }
    let _ = writeln!(out, "{}", dim(meta.join(", ")));

    for note in &report.notes {
        let _ = writeln!(out, "{} {}", badge("note", YELLOW_BOLD), dim(note));
    }

    for (index, section) in report.sections.iter().enumerate() {
        if index > 0 || !report.notes.is_empty() {
            out.push('\n');
        }
        render_diff_section(&mut out, section);
    }

    out
}

fn parse_unified_diff(stdout: &str) -> DiffReport {
    let mut report = DiffReport::default();
    let mut pending_old = None::<String>;
    let mut current_section = None::<DiffSection>;
    let mut current_hunk = None::<DiffHunk>;

    for line in stdout.lines() {
        if let Some(value) = line.strip_prefix("--- ") {
            finalize_diff_hunk(&mut current_section, &mut current_hunk);
            finalize_diff_section(&mut report, &mut current_section);
            pending_old = Some(parse_diff_label(value));
            continue;
        }

        if let Some(value) = line.strip_prefix("+++ ") {
            finalize_diff_hunk(&mut current_section, &mut current_hunk);
            finalize_diff_section(&mut report, &mut current_section);
            let old_label = pending_old.take().unwrap_or_else(|| "?".to_string());
            current_section = Some(DiffSection::new(old_label, parse_diff_label(value)));
            continue;
        }

        if line.starts_with("@@") {
            finalize_diff_hunk(&mut current_section, &mut current_hunk);
            current_hunk = Some(DiffHunk {
                header: line.to_string(),
                lines: Vec::new(),
            });
            continue;
        }

        let line_kind = if line.starts_with('+') {
            Some(DiffLineKind::Added)
        } else if line.starts_with('-') {
            Some(DiffLineKind::Removed)
        } else if line.starts_with(' ') {
            Some(DiffLineKind::Context)
        } else if line == r"\ No newline at end of file" {
            Some(DiffLineKind::Note)
        } else {
            None
        };

        if let Some(kind) = line_kind {
            if let Some(hunk) = current_hunk.as_mut() {
                hunk.lines.push(DiffLine {
                    kind,
                    text: line.to_string(),
                });
                continue;
            }
        }

        if line.starts_with("diff ") {
            continue;
        }

        if let Some(section) = current_section.as_mut() {
            section.notes.push(line.to_string());
        } else if !line.is_empty() {
            report.notes.push(line.to_string());
        }
    }

    finalize_diff_hunk(&mut current_section, &mut current_hunk);
    finalize_diff_section(&mut report, &mut current_section);
    report
}

fn parse_diff_label(value: &str) -> String {
    value
        .split_once('\t')
        .map(|(label, _)| label)
        .unwrap_or(value)
        .trim()
        .to_string()
}

fn finalize_diff_hunk(section: &mut Option<DiffSection>, hunk: &mut Option<DiffHunk>) {
    let Some(hunk) = hunk.take() else {
        return;
    };
    let Some(section) = section.as_mut() else {
        return;
    };

    section.additions += hunk
        .lines
        .iter()
        .filter(|line| line.kind == DiffLineKind::Added)
        .count();
    section.deletions += hunk
        .lines
        .iter()
        .filter(|line| line.kind == DiffLineKind::Removed)
        .count();
    section.hunks.push(hunk);
}

fn finalize_diff_section(report: &mut DiffReport, section: &mut Option<DiffSection>) {
    if let Some(section) = section.take() {
        report.sections.push(section);
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

fn truncate_history_entry(entry: &str, max_chars: usize) -> String {
    if entry.chars().count() <= max_chars {
        return format!("`{entry}`");
    }

    let truncated = entry
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    format!("`{}...`", truncated.trim_end())
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

#[derive(Clone, Copy, Default)]
enum HiddenMode {
    #[default]
    Default,
    AlmostAll,
    All,
}

impl HiddenMode {
    fn include(self, other: Self) -> Self {
        match (self, other) {
            (Self::All, _) | (_, Self::All) => Self::All,
            (Self::AlmostAll, _) | (_, Self::AlmostAll) => Self::AlmostAll,
            _ => Self::Default,
        }
    }

    fn shows_hidden(self) -> bool {
        !matches!(self, Self::Default)
    }

    fn shows_dot_entries(self) -> bool {
        matches!(self, Self::All)
    }
}

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

#[derive(Default)]
struct GrepOptions {
    ignore_case: bool,
    invert_match: bool,
    word_regexp: bool,
    line_regexp: bool,
    fixed_strings: bool,
    extended_regexp: bool,
    recursive: bool,
    no_messages: bool,
    text: bool,
    max_count: Option<usize>,
    patterns: Vec<String>,
    pattern_files: Vec<String>,
    targets: Vec<String>,
}

impl GrepOptions {
    fn has_pattern_source(&self) -> bool {
        !self.patterns.is_empty() || !self.pattern_files.is_empty()
    }
}

enum GrepPending {
    Pattern,
    PatternFile,
    MaxCount,
}

#[derive(Default)]
struct GrepReport {
    groups: Vec<GrepGroup>,
    notes: Vec<String>,
}

struct GrepGroup {
    source: String,
    rows: Vec<GrepMatchRow>,
}

struct GrepMatchRow {
    line_number: usize,
    text: String,
}

struct GrepMatch {
    source: String,
    line_number: usize,
    text: String,
}

#[derive(Default)]
struct DiffOptions {
    recursive: bool,
    new_file: bool,
    text: bool,
    ignore_all_space: bool,
    ignore_space_change: bool,
    ignore_blank_lines: bool,
    ignore_case: bool,
    context: usize,
    targets: Vec<String>,
}

#[derive(Default)]
struct DiffReport {
    sections: Vec<DiffSection>,
    notes: Vec<String>,
}

struct DiffSection {
    old_label: String,
    new_label: String,
    hunks: Vec<DiffHunk>,
    additions: usize,
    deletions: usize,
    notes: Vec<String>,
}

impl DiffSection {
    fn new(old_label: String, new_label: String) -> Self {
        Self {
            old_label,
            new_label,
            hunks: Vec::new(),
            additions: 0,
            deletions: 0,
            notes: Vec::new(),
        }
    }
}

struct DiffHunk {
    header: String,
    lines: Vec<DiffLine>,
}

struct DiffLine {
    kind: DiffLineKind,
    text: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DiffLineKind {
    Context,
    Added,
    Removed,
    Note,
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

fn describe_ls_entry(file_name: &str, entry_path: &Path, hidden_mode: HiddenMode) -> Result<LsRow> {
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
            count_visible_children(entry_path, hidden_mode)?,
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

fn count_visible_children(path: &Path, hidden_mode: HiddenMode) -> Result<usize> {
    let mut count = fs::read_dir(path)?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|entry| {
            hidden_mode.shows_hidden() || !entry.file_name().to_string_lossy().starts_with('.')
        })
        .count();
    if hidden_mode.shows_dot_entries() {
        count += 2;
    }
    Ok(count)
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

fn render_alias_row(out: &mut String, name: &str, value: &str) {
    let _ = writeln!(
        out,
        "{} {} {}",
        paint(CYAN_BOLD, name),
        badge("alias", BLUE_BOLD),
        paint(BOLD, value)
    );
}

fn render_which_row(out: &mut String, name: &str, result: Option<&CommandLookup>) {
    match result {
        Some(CommandLookup::Alias(value)) => {
            let _ = writeln!(
                out,
                "{} {} {}",
                paint(CYAN_BOLD, name),
                badge("alias", BLUE_BOLD),
                paint(BOLD, value)
            );
        }
        Some(CommandLookup::Builtin) => {
            let _ = writeln!(
                out,
                "{} {} {}",
                paint(CYAN_BOLD, name),
                badge("builtin", YELLOW_BOLD),
                dim("shell builtin")
            );
        }
        Some(CommandLookup::External(path)) => {
            let _ = writeln!(
                out,
                "{} {} {}",
                paint(CYAN_BOLD, name),
                badge("external", GREEN_BOLD),
                dim(path.display())
            );
        }
        None => {
            let _ = writeln!(
                out,
                "{} {} {}",
                paint(CYAN_BOLD, name),
                badge("not found", RED_BOLD),
                dim("unavailable on PATH")
            );
        }
    }
}

fn render_diff_section(out: &mut String, section: &DiffSection) {
    let mut parts = vec![
        paint(BOLD, &section.old_label),
        paint(CYAN_BOLD, "->"),
        paint(BOLD, &section.new_label),
    ];
    if !section.hunks.is_empty() {
        parts.push(badge(
            pluralize(section.hunks.len(), "hunk", "hunks"),
            BLUE_BOLD,
        ));
    }
    if section.additions > 0 {
        parts.push(badge(format!("+{}", section.additions), GREEN_BOLD));
    }
    if section.deletions > 0 {
        parts.push(badge(format!("-{}", section.deletions), RED_BOLD));
    }
    let _ = writeln!(out, "{}", parts.join(" "));

    for note in &section.notes {
        let _ = writeln!(out, "{} {}", badge("note", YELLOW_BOLD), dim(note));
    }

    for (index, hunk) in section.hunks.iter().enumerate() {
        if index > 0 || !section.notes.is_empty() {
            out.push('\n');
        }
        let _ = writeln!(out, "{}", paint(CYAN_BOLD, &hunk.header));
        for line in &hunk.lines {
            render_diff_line(out, line);
        }
    }
}

fn render_diff_line(out: &mut String, line: &DiffLine) {
    let rendered = match line.kind {
        DiffLineKind::Context => dim(&line.text),
        DiffLineKind::Added => paint(GREEN_BOLD, &line.text),
        DiffLineKind::Removed => paint(RED_BOLD, &line.text),
        DiffLineKind::Note => dim(&line.text),
    };
    let _ = writeln!(out, "{rendered}");
}

fn render_grep_group(out: &mut String, group: &GrepGroup) {
    let _ = writeln!(
        out,
        "{} {}",
        paint(BOLD, &group.source),
        badge(pluralize(group.rows.len(), "match", "matches"), BLUE_BOLD,)
    );

    for row in &group.rows {
        let _ = writeln!(
            out,
            "  {} {}",
            badge(format!("line {}", row.line_number), CYAN_BOLD),
            paint(BOLD, &row.text)
        );
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
