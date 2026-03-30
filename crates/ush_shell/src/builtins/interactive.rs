use anyhow::{Result, bail};

use super::interactive_support::{
    prompt_confirm, prompt_input, prompt_select, resolve_default_selection,
};
use crate::{Shell, ValueStream};

pub(super) enum ConfirmDefault {
    Yes,
    No,
}

pub(super) struct SelectConfig {
    pub prompt: Option<String>,
    pub default: Option<String>,
    pub options: Vec<String>,
}

impl Shell {
    pub(super) fn handle_confirm(
        &self,
        args: &[String],
        input: ValueStream,
    ) -> Result<(ValueStream, i32)> {
        let (prompt, default) = parse_confirm_args(args, input)?;
        if !self.options.interaction {
            return Ok((
                ValueStream::Empty,
                status_from_bool(matches!(default, ConfirmDefault::Yes)),
            ));
        }
        let accepted = prompt_confirm(prompt.as_deref(), default)?;
        Ok((ValueStream::Empty, status_from_bool(accepted)))
    }

    pub(super) fn handle_input(
        &self,
        args: &[String],
        input: ValueStream,
    ) -> Result<(ValueStream, i32)> {
        let (prompt, default) = parse_input_args(args)?;
        if let Some(value) = first_stream_line(input)? {
            return Ok((ValueStream::Text(format!("{value}\n")), 0));
        }
        if !self.options.interaction {
            return Ok(match default {
                Some(value) => (ValueStream::Text(format!("{value}\n")), 0),
                None => (ValueStream::Empty, 1),
            });
        }
        let answer = prompt_input(prompt.as_deref(), default.as_deref())?;
        Ok(match answer {
            Some(value) => (ValueStream::Text(format!("{value}\n")), 0),
            None => (ValueStream::Empty, 1),
        })
    }

    pub(super) fn handle_select(
        &self,
        args: &[String],
        input: ValueStream,
    ) -> Result<(ValueStream, i32)> {
        let config = parse_select_args(args, input)?;
        if !self.options.interaction {
            let selected = resolve_default_selection(&config)?;
            return Ok(match selected {
                Some(value) => (ValueStream::Text(format!("{value}\n")), 0),
                None => (ValueStream::Empty, 1),
            });
        }
        let answer = prompt_select(&config)?;
        Ok(match answer {
            Some(value) => (ValueStream::Text(format!("{value}\n")), 0),
            None => (ValueStream::Empty, 1),
        })
    }
}

fn parse_confirm_args(
    args: &[String],
    input: ValueStream,
) -> Result<(Option<String>, ConfirmDefault)> {
    let mut prompt = Vec::new();
    let mut default = ConfirmDefault::No;
    let mut index = 0usize;
    while let Some(arg) = args.get(index) {
        match arg.as_str() {
            "--default" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("confirm --default requires `yes` or `no`");
                };
                default = parse_confirm_default(value)?;
                index += 2;
            }
            "--prompt" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("confirm --prompt requires a value");
                };
                prompt.push(value.clone());
                index += 2;
            }
            _ => {
                prompt.extend(args[index..].iter().cloned());
                break;
            }
        }
    }
    if prompt.is_empty() {
        if let Some(value) = first_stream_line(input)? {
            prompt.push(value);
        }
    }
    Ok((join_prompt(prompt), default))
}

fn parse_input_args(args: &[String]) -> Result<(Option<String>, Option<String>)> {
    let mut prompt = Vec::new();
    let mut default = None;
    let mut index = 0usize;
    while let Some(arg) = args.get(index) {
        match arg.as_str() {
            "--default" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("input --default requires a value");
                };
                default = Some(value.clone());
                index += 2;
            }
            "--prompt" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("input --prompt requires a value");
                };
                prompt.push(value.clone());
                index += 2;
            }
            _ => {
                prompt.extend(args[index..].iter().cloned());
                break;
            }
        }
    }
    Ok((join_prompt(prompt), default))
}

fn parse_select_args(args: &[String], input: ValueStream) -> Result<SelectConfig> {
    let mut prompt = None;
    let mut default = None;
    let mut options = Vec::new();
    let mut index = 0usize;
    while let Some(arg) = args.get(index) {
        match arg.as_str() {
            "--default" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("select --default requires a value");
                };
                default = Some(value.clone());
                index += 2;
            }
            "--prompt" => {
                let Some(value) = args.get(index + 1) else {
                    bail!("select --prompt requires a value");
                };
                prompt = Some(value.clone());
                index += 2;
            }
            "--" => {
                options.extend(args[index + 1..].iter().cloned());
                break;
            }
            _ => {
                options.extend(args[index..].iter().cloned());
                break;
            }
        }
    }
    if options.is_empty() {
        options = input
            .into_lines()?
            .into_iter()
            .filter(|line| !line.is_empty())
            .collect();
    }
    if options.is_empty() {
        bail!("select requires at least one option");
    }
    Ok(SelectConfig {
        prompt,
        default,
        options,
    })
}

fn parse_confirm_default(value: &str) -> Result<ConfirmDefault> {
    match value {
        "y" | "Y" | "yes" | "YES" | "true" | "TRUE" => Ok(ConfirmDefault::Yes),
        "n" | "N" | "no" | "NO" | "false" | "FALSE" => Ok(ConfirmDefault::No),
        _ => bail!("confirm --default must be `yes` or `no`"),
    }
}

fn first_stream_line(input: ValueStream) -> Result<Option<String>> {
    Ok(input.into_lines()?.into_iter().next())
}

fn join_prompt(parts: Vec<String>) -> Option<String> {
    let prompt = parts.join(" ");
    if prompt.is_empty() {
        None
    } else {
        Some(prompt)
    }
}

fn status_from_bool(value: bool) -> i32 {
    if value { 0 } else { 1 }
}
