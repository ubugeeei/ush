use std::{path::Path, process::Command};

use ush_config::StarshipPromptConfig;

pub(crate) fn render_prompt(
    cwd: &Path,
    home: Option<&str>,
    last_status: i32,
    starship: Option<&StarshipPromptConfig>,
) -> String {
    starship
        .map(|config| starship_prompt(cwd, home, last_status, config))
        .unwrap_or_else(|| default_prompt(cwd, home, last_status))
}

pub(crate) fn default_prompt(cwd: &Path, home: Option<&str>, last_status: i32) -> String {
    let mark = if last_status == 0 { "$" } else { "!" };
    format!("{} {} ", compact_path(cwd, home), mark)
}

fn compact_path(cwd: &Path, home: Option<&str>) -> String {
    compact_path_with(cwd, home, 2, ".../", "~")
}

fn starship_prompt(
    cwd: &Path,
    home: Option<&str>,
    last_status: i32,
    config: &StarshipPromptConfig,
) -> String {
    let directory = compact_path_with(
        cwd,
        home,
        config.directory.truncation_length,
        &config.directory.truncation_symbol,
        &config.directory.home_symbol,
    );
    let character = normalize_character(if last_status == 0 {
        &config.character.success_symbol
    } else {
        &config.character.error_symbol
    });
    let git_branch = current_git_branch(cwd).map(|branch| render_git_branch(&branch, config));

    if let Some(format) = &config.format {
        let rendered =
            render_starship_format(format, &directory, &character, git_branch.as_deref());
        return ensure_prompt_suffix(rendered);
    }

    if config.add_newline {
        return format!("{directory}\n{character}");
    }
    format!("{directory} {character}")
}

fn compact_path_with(
    cwd: &Path,
    home: Option<&str>,
    truncation_length: usize,
    truncation_symbol: &str,
    home_symbol: &str,
) -> String {
    if cwd == Path::new("/") {
        return "/".to_string();
    }

    if let Some(home) = home {
        let home_path = Path::new(home);
        if cwd == home_path {
            return home_symbol.to_string();
        }
        if let Ok(relative) = cwd.strip_prefix(home_path) {
            return compact_components(
                home_symbol,
                path_parts(relative),
                truncation_length,
                truncation_symbol,
            );
        }
    }

    compact_components("/", path_parts(cwd), truncation_length, truncation_symbol)
}

fn compact_components(
    prefix: &str,
    parts: Vec<String>,
    truncation_length: usize,
    truncation_symbol: &str,
) -> String {
    if parts.is_empty() {
        return prefix.to_string();
    }
    let body = if parts.len() <= truncation_length {
        parts.join("/")
    } else {
        format!(
            "{}{}",
            truncation_symbol,
            parts[parts.len() - truncation_length..].join("/")
        )
    };
    if prefix == "/" {
        format!("/{body}")
    } else {
        format!("{prefix}/{body}")
    }
}

fn path_parts(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| {
            let part = component.as_os_str().to_string_lossy();
            (!part.is_empty() && part != "/").then_some(part.into_owned())
        })
        .collect()
}

fn normalize_character(symbol: &str) -> String {
    if symbol.ends_with(char::is_whitespace) {
        symbol.to_string()
    } else {
        format!("{symbol} ")
    }
}

fn current_git_branch(cwd: &Path) -> Option<String> {
    let branch = git_output(cwd, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if branch != "HEAD" {
        return Some(branch);
    }
    let commit = git_output(cwd, &["rev-parse", "--short", "HEAD"])?;
    Some(commit)
}

fn git_output(cwd: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn render_git_branch(branch: &str, config: &StarshipPromptConfig) -> String {
    let git_branch = &config.git_branch;
    let format = git_branch
        .format
        .as_deref()
        .unwrap_or("[$symbol$branch]($style) ");
    let rendered = format
        .replace("$symbol", &git_branch.symbol)
        .replace("$branch", branch)
        .replace("$style", &git_branch.style);
    strip_starship_markup(&rendered)
}

fn render_starship_format(
    format: &str,
    directory: &str,
    character: &str,
    git_branch: Option<&str>,
) -> String {
    let mut out = String::new();
    let chars = format.chars().collect::<Vec<_>>();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] != '$' {
            out.push(chars[index]);
            index += 1;
            continue;
        }
        let rest = chars[index..].iter().collect::<String>();
        if rest.starts_with("$directory") {
            out.push_str(directory);
            index += "$directory".len();
            continue;
        }
        if rest.starts_with("$character") {
            out.push_str(character);
            index += "$character".len();
            continue;
        }
        if rest.starts_with("$line_break") {
            out.push('\n');
            index += "$line_break".len();
            continue;
        }
        if rest.starts_with("$git_branch") {
            if let Some(git_branch) = git_branch {
                out.push_str(git_branch);
            }
            index += "$git_branch".len();
            continue;
        }
        if chars
            .get(index + 1)
            .copied()
            .is_some_and(is_starship_module_char)
        {
            index += 1;
            while chars
                .get(index)
                .copied()
                .is_some_and(is_starship_module_char)
            {
                index += 1;
            }
            continue;
        }
        out.push('$');
        index += 1;
    }

    out
}

fn is_starship_module_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn strip_starship_markup(input: &str) -> String {
    let chars = input.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '[' {
            if let Some((inner, end)) = starship_style_group(&chars, index) {
                out.push_str(&unescape_starship_text(&inner));
                index = end;
                continue;
            }
        }
        if chars[index] == '\\' {
            if let Some(next) = chars.get(index + 1) {
                out.push(*next);
                index += 2;
                continue;
            }
        }
        out.push(chars[index]);
        index += 1;
    }
    out
}

fn starship_style_group(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut inner = String::new();
    let mut cursor = start + 1;
    while cursor < chars.len() {
        if chars[cursor] == '\\' {
            if let Some(next) = chars.get(cursor + 1) {
                inner.push(chars[cursor]);
                inner.push(*next);
                cursor += 2;
                continue;
            }
        }
        if chars[cursor] == ']' && chars.get(cursor + 1).copied() == Some('(') {
            let mut style_cursor = cursor + 2;
            while style_cursor < chars.len() {
                if chars[style_cursor] == ')' {
                    return Some((inner, style_cursor + 1));
                }
                style_cursor += 1;
            }
            return None;
        }
        inner.push(chars[cursor]);
        cursor += 1;
    }
    None
}

fn unescape_starship_text(raw: &str) -> String {
    let mut out = String::new();
    let mut escaped = false;
    for ch in raw.chars() {
        if escaped {
            out.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    if escaped {
        out.push('\\');
    }
    out
}

fn ensure_prompt_suffix(mut prompt: String) -> String {
    if prompt.ends_with('\n') || prompt.ends_with(' ') {
        return prompt;
    }
    prompt.push(' ');
    prompt
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ush_config::StarshipPromptConfig;

    use super::{compact_path, default_prompt, render_prompt};

    #[test]
    fn abbreviates_home_and_deep_paths() {
        let home = Some("/Users/user");

        assert_eq!(compact_path(Path::new("/Users/user"), home), "~");
        assert_eq!(compact_path(Path::new("/Users/user/src"), home), "~/src");
        assert_eq!(
            compact_path(Path::new("/Users/user/Code/github.com/ubugeeei/ush"), home),
            "~/.../ubugeeei/ush"
        );
        assert_eq!(
            compact_path(Path::new("/usr/local/bin"), home),
            "/.../local/bin"
        );
    }

    #[test]
    fn formats_default_prompt_with_short_path() {
        let prompt = default_prompt(
            Path::new("/Users/user/Code/github.com/ubugeeei/ush"),
            Some("/Users/user"),
            0,
        );

        assert_eq!(prompt, "~/.../ubugeeei/ush $ ");
    }

    #[test]
    fn renders_starship_style_prompt_when_config_is_present() {
        let mut starship = StarshipPromptConfig {
            add_newline: true,
            ..StarshipPromptConfig::default()
        };
        starship.directory.truncation_length = 2;
        starship.character.success_symbol = "❯".into();

        let prompt = render_prompt(
            Path::new("/Users/user/Code/github.com/ubugeeei/ush"),
            Some("/Users/user"),
            0,
            Some(&starship),
        );

        assert_eq!(prompt, "~/.../ubugeeei/ush\n❯ ");
    }

    #[test]
    fn honors_starship_home_symbol_and_error_character() {
        let mut starship = StarshipPromptConfig::default();
        starship.directory.home_symbol = "~home".into();
        starship.character.error_symbol = "✗".into();

        let prompt = render_prompt(
            Path::new("/Users/user/project"),
            Some("/Users/user"),
            1,
            Some(&starship),
        );

        assert_eq!(prompt, "~home/project ✗ ");
    }

    #[test]
    fn renders_git_branch_from_starship_format() {
        let mut starship = StarshipPromptConfig::default();
        starship.git_branch.symbol = "|- ".into();
        starship.git_branch.format = Some(" [$symbol$branch]($style)".into());

        let branch = super::render_git_branch("main", &starship);
        let prompt = super::render_starship_format(
            "$directory$git_branch$line_break$character",
            "~/repo",
            "$ ",
            Some(&branch),
        );

        assert_eq!(prompt, "~/repo |- main\n$ ");
    }

    #[test]
    fn strips_starship_markup_and_escapes() {
        assert_eq!(
            super::strip_starship_markup(r" [\(main\)](cyan)"),
            " (main)"
        );
    }
}
