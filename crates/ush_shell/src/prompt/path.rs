use std::path::Path;

pub(super) fn compact_path(cwd: &Path, home: Option<&str>) -> String {
    compact_path_with(cwd, home, 2, ".../", "~")
}

pub(super) fn compact_path_with(
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
