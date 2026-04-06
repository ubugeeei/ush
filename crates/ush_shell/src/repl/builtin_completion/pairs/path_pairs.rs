use std::path::Path;

use rustyline::completion::Pair;

fn described_pair(replacement: &str, summary: &str) -> Pair {
    Pair {
        display: format!("{replacement}  {summary}"),
        replacement: replacement.to_string(),
    }
}

pub(super) fn complete_path_pairs(cwd: &Path, word: &str) -> Vec<Pair> {
    let (dir_prefix, file_prefix) = split_path_prefix(word);
    let search_dir = resolve_completion_dir(cwd, &dir_prefix);
    let mut entries: Vec<_> = std::fs::read_dir(search_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if !file_name.starts_with(&file_prefix) {
                return None;
            }
            if !file_prefix.starts_with('.') && file_name.starts_with('.') {
                return None;
            }

            let metadata = entry.metadata().ok()?;
            let suffix = if metadata.is_dir() { "/" } else { "" };
            let replacement = format!("{}{}{}", dir_prefix, escape_shell_text(&file_name), suffix);
            let summary = if metadata.is_dir() {
                "directory"
            } else {
                "path"
            };
            Some(described_pair(&replacement, summary))
        })
        .collect();
    entries.sort_by(|left, right| left.replacement.cmp(&right.replacement));
    entries
}

fn split_path_prefix(word: &str) -> (String, String) {
    if word == "~" {
        return ("~/".to_string(), String::new());
    }
    if let Some(index) = word.rfind('/') {
        return (word[..=index].to_string(), word[index + 1..].to_string());
    }
    (String::new(), word.to_string())
}

fn resolve_completion_dir(cwd: &Path, dir_prefix: &str) -> std::path::PathBuf {
    if dir_prefix.is_empty() {
        return cwd.to_path_buf();
    }
    if let Some(rest) = dir_prefix.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return Path::new(&home).join(rest);
    }
    let path = Path::new(dir_prefix);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn escape_shell_text(value: &str) -> String {
    value.replace(' ', "\\ ")
}
