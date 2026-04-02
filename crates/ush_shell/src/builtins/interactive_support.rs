use std::io::{self, Write};

use anyhow::{Result, bail};

use super::interactive::{ConfirmDefault, SelectConfig};

pub(super) fn prompt_confirm(prompt: Option<&str>, default: ConfirmDefault) -> Result<bool> {
    let label = prompt.unwrap_or("confirm");
    let hint = match default {
        ConfirmDefault::Yes => "[Y/n]",
        ConfirmDefault::No => "[y/N]",
    };
    loop {
        eprint!("ush: {label} {hint} ");
        io::stderr().flush()?;
        let Some(answer) = read_line()? else {
            return Ok(matches!(default, ConfirmDefault::Yes));
        };
        match answer.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(matches!(default, ConfirmDefault::Yes)),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => eprintln!("ush: answer `yes` or `no`"),
        }
    }
}

pub(super) fn prompt_input(prompt: Option<&str>, default: Option<&str>) -> Result<Option<String>> {
    if let Some(label) = prompt {
        eprint!("ush: {label} ");
        io::stderr().flush()?;
    }
    match read_line()? {
        Some(answer) if !answer.is_empty() => Ok(Some(answer)),
        Some(_) | None => Ok(default.map(str::to_string)),
    }
}

pub(super) fn prompt_select(config: &SelectConfig) -> Result<Option<String>> {
    let prompt = config.prompt.as_deref().unwrap_or("select an option");
    eprintln!("ush: {prompt}");
    for (index, option) in config.options.iter().enumerate() {
        eprintln!("  {}: {}", index + 1, option);
    }
    loop {
        let suffix = config
            .default
            .as_deref()
            .map_or(String::new(), |value| format!(" [default: {value}]"));
        eprint!("ush: choice{suffix} ");
        io::stderr().flush()?;
        let Some(answer) = read_line()? else {
            return resolve_default_selection(config);
        };
        let answer = answer.trim();
        if answer.is_empty() {
            return resolve_default_selection(config);
        }
        if let Some(value) = select_option(answer, &config.options) {
            return Ok(Some(value));
        }
        eprintln!("ush: choose a number from the list or an exact option");
    }
}

pub(super) fn resolve_default_selection(config: &SelectConfig) -> Result<Option<String>> {
    if let Some(default) = &config.default {
        return match select_option(default, &config.options) {
            Some(value) => Ok(Some(value)),
            None => bail!("select default `{default}` did not match any option"),
        };
    }
    if config.options.len() == 1 {
        return Ok(config.options.first().cloned());
    }
    Ok(None)
}

fn select_option(input: &str, options: &[String]) -> Option<String> {
    if let Ok(index) = input.parse::<usize>()
        && (1..=options.len()).contains(&index)
    {
        return options.get(index - 1).cloned();
    }
    options
        .iter()
        .find(|option| option.as_str() == input)
        .cloned()
}

fn read_line() -> Result<Option<String>> {
    let mut answer = String::new();
    let bytes = io::stdin().read_line(&mut answer)?;
    if bytes == 0 {
        return Ok(None);
    }
    Ok(Some(answer.trim_end_matches(['\n', '\r']).to_string()))
}
