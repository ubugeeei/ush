use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};

use super::{Shell, commands::BUILTIN_COMMANDS};
use crate::prompt::render_prompt;

impl Shell {
    pub(crate) fn command_names(&self) -> Vec<String> {
        let mut commands = BTreeSet::new();
        commands.extend(
            BUILTIN_COMMANDS
                .iter()
                .map(|builtin| (*builtin).to_string()),
        );
        commands.extend(
            [
                "len",
                "length",
                "lines",
                "json",
                "xml",
                "html",
                "car",
                "cdr",
                "map",
                "fmap",
                "head",
                "tail",
                "take",
                "drop",
                "nth",
                "enumerate",
                "swap",
                "fst",
                "snd",
                "frev",
                "fsort",
                "funiq",
                "fjoin",
                "flat",
                "ffmap",
                "fzip",
                "filter",
                "ffilter",
                "each",
                "any",
                "fany",
                "some",
                "fsome",
            ]
            .into_iter()
            .map(str::to_string),
        );
        commands.extend(self.aliases.keys().cloned());

        if let Some(path) = self.env.get("PATH") {
            for dir in env::split_paths(path) {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            commands.insert(name.to_string());
                        }
                    }
                }
            }
        }

        commands.into_iter().collect()
    }

    pub(crate) fn prompt(&self) -> String {
        if let Some(prompt) = &self.config.shell.prompt {
            return prompt.clone();
        }
        render_prompt(
            &self.cwd,
            self.env.get("HOME").map(String::as_str),
            self.last_status,
            self.config.shell.starship.as_ref(),
        )
    }

    pub(crate) fn expand_args(&self, args: &[String]) -> Result<Vec<String>> {
        let mut expanded = Vec::new();
        for arg in args {
            expanded.extend(self.expand_arg(arg)?);
        }
        Ok(expanded)
    }

    pub(crate) fn expand_value(&self, value: &str) -> Result<String> {
        let value = expand_home(value, &self.env);
        expand_vars(&value, &self.env, self.last_status)
    }

    pub(crate) fn normalize_path(&self, value: &str) -> PathBuf {
        let path = Path::new(value);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.cwd.join(path)
        }
    }

    fn expand_arg(&self, arg: &str) -> Result<Vec<String>> {
        let expanded = self.expand_value(arg)?;
        if contains_glob(&expanded) {
            let matches = glob::glob(&expanded)?
                .filter_map(Result::ok)
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>();
            if !matches.is_empty() {
                return Ok(matches);
            }
        }
        Ok(vec![expanded])
    }
}

pub(crate) fn strip_outer_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|inner| inner.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn expand_home(value: &str, env: &std::collections::HashMap<String, String>) -> String {
    if value == "~" {
        return env.get("HOME").cloned().unwrap_or_else(|| "~".to_string());
    }
    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = env.get("HOME") {
            return format!("{home}/{rest}");
        }
    }
    value.to_string()
}

fn expand_vars(
    value: &str,
    env: &std::collections::HashMap<String, String>,
    last_status: i32,
) -> Result<String> {
    let mut result = String::new();
    let chars = value.chars().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '$' {
            result.push(chars[index]);
            index += 1;
            continue;
        }
        if index + 1 >= chars.len() {
            result.push('$');
            break;
        }

        match chars[index + 1] {
            '?' => {
                result.push_str(&last_status.to_string());
                index += 2;
            }
            '{' => {
                let mut end = index + 2;
                while end < chars.len() && chars[end] != '}' {
                    end += 1;
                }
                if end == chars.len() {
                    bail!("unterminated variable expansion");
                }
                let name = chars[index + 2..end].iter().collect::<String>();
                result.push_str(env.get(&name).map(String::as_str).unwrap_or_default());
                index = end + 1;
            }
            next if next == '_' || next.is_ascii_alphabetic() => {
                let mut end = index + 1;
                while end < chars.len() && (chars[end] == '_' || chars[end].is_ascii_alphanumeric())
                {
                    end += 1;
                }
                let name = chars[index + 1..end].iter().collect::<String>();
                result.push_str(env.get(&name).map(String::as_str).unwrap_or_default());
                index = end;
            }
            _ => {
                result.push('$');
                index += 1;
            }
        }
    }

    Ok(result)
}

fn contains_glob(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}
