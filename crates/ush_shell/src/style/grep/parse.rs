use std::{
    io::Write as _,
    process::{Command, Output, Stdio},
};

use anyhow::{Context, Result};

use crate::helpers::ValueStream;

use super::model::{GrepOptions, GrepPending};

pub(super) fn parse_grep_args(args: &[String]) -> Option<GrepOptions> {
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
                options.patterns.push(arg.split_once('=')?.1.to_string())
            }
            _ if arg.starts_with("--file=") => options
                .pattern_files
                .push(arg.split_once('=')?.1.to_string()),
            _ if arg.starts_with("--max-count=") => {
                options.max_count = Some(arg.split_once('=')?.1.parse().ok()?)
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

pub(super) fn build_grep_command_args(options: &GrepOptions) -> Vec<String> {
    let mut args = vec!["-nH".to_string()];
    for (enabled, flag) in [
        (options.ignore_case, "-i"),
        (options.invert_match, "-v"),
        (options.word_regexp, "-w"),
        (options.line_regexp, "-x"),
        (options.fixed_strings, "-F"),
        (options.extended_regexp, "-E"),
        (options.recursive, "-R"),
        (options.no_messages, "-s"),
        (options.text, "-a"),
    ] {
        if enabled {
            args.push(flag.to_string());
        }
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

pub(super) fn capture_command_output(command: &mut Command, input: &ValueStream) -> Result<Output> {
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
            'e' | 'f' | 'm' => {
                return parse_grep_value_flag(flag, chars.collect(), options, pending);
            }
            _ => return None,
        }
    }
    Some(())
}

fn parse_grep_value_flag(
    flag: char,
    rest: String,
    options: &mut GrepOptions,
    pending: &mut Option<GrepPending>,
) -> Option<()> {
    if rest.is_empty() {
        *pending = Some(match flag {
            'e' => GrepPending::Pattern,
            'f' => GrepPending::PatternFile,
            'm' => GrepPending::MaxCount,
            _ => return None,
        });
    } else {
        match flag {
            'e' => options.patterns.push(rest),
            'f' => options.pattern_files.push(rest),
            'm' => options.max_count = Some(rest.parse().ok()?),
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
