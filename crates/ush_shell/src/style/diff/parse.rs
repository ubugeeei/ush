use super::model::{DiffHunk, DiffLine, DiffLineKind, DiffOptions, DiffReport, DiffSection};

pub(super) fn parse_diff_args(args: &[String]) -> Option<DiffOptions> {
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
                options.context = arg.split_once('=')?.1.parse().ok()?
            }
            _ if arg.starts_with('-') && arg.len() > 1 => {
                parse_diff_short_flags(arg, &mut options, &mut pending_context)?
            }
            _ => options.targets.push(arg.clone()),
        }
    }

    (!pending_context && options.targets.len() == 2).then_some(options)
}

pub(super) fn build_diff_command_args(options: &DiffOptions) -> Vec<String> {
    let mut args = vec![format!("--unified={}", options.context)];
    for (enabled, flag) in [
        (options.recursive, "--recursive"),
        (options.new_file, "--new-file"),
        (options.text, "--text"),
        (options.ignore_all_space, "--ignore-all-space"),
        (options.ignore_space_change, "--ignore-space-change"),
        (options.ignore_blank_lines, "--ignore-blank-lines"),
        (options.ignore_case, "--ignore-case"),
    ] {
        if enabled {
            args.push(flag.to_string());
        }
    }
    args.push("--".to_string());
    args.extend(options.targets.iter().cloned());
    args
}

pub(super) fn parse_unified_diff(stdout: &str) -> DiffReport {
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
            current_section = Some(DiffSection::new(
                pending_old.take().unwrap_or_else(|| "?".to_string()),
                parse_diff_label(value),
            ));
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

        if let Some(kind) = diff_line_kind(line)
            && let Some(hunk) = current_hunk.as_mut()
        {
            hunk.lines.push(DiffLine {
                kind,
                text: line.to_string(),
            });
            continue;
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

fn parse_diff_label(value: &str) -> String {
    value
        .split_once('\t')
        .map(|(label, _)| label)
        .unwrap_or(value)
        .trim()
        .to_string()
}

fn diff_line_kind(line: &str) -> Option<DiffLineKind> {
    if line.starts_with('+') {
        Some(DiffLineKind::Added)
    } else if line.starts_with('-') {
        Some(DiffLineKind::Removed)
    } else if line.starts_with(' ') {
        Some(DiffLineKind::Context)
    } else if line == r"\ No newline at end of file" {
        Some(DiffLineKind::Note)
    } else {
        None
    }
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
