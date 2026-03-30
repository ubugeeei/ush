use std::path::Path;

pub(crate) fn default_prompt(cwd: &Path, home: Option<&str>, last_status: i32) -> String {
    let mark = if last_status == 0 { "$" } else { "!" };
    format!("ush {} {} ", compact_path(cwd, home), mark)
}

fn compact_path(cwd: &Path, home: Option<&str>) -> String {
    if cwd == Path::new("/") {
        return "/".to_string();
    }

    if let Some(home) = home {
        let home_path = Path::new(home);
        if cwd == home_path {
            return "~".to_string();
        }
        if let Ok(relative) = cwd.strip_prefix(home_path) {
            return compact_components("~", path_parts(relative));
        }
    }

    compact_components("/", path_parts(cwd))
}

fn compact_components(prefix: &str, parts: Vec<String>) -> String {
    match parts.as_slice() {
        [] => prefix.to_string(),
        [only] if prefix == "~" => format!("~/{only}"),
        [only] => format!("/{only}"),
        [first, second] if prefix == "~" => format!("~/{first}/{second}"),
        [first, second] => format!("/{first}/{second}"),
        [.., penultimate, last] if prefix == "~" => format!("~/.../{penultimate}/{last}"),
        [.., penultimate, last] => format!("/.../{penultimate}/{last}"),
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{compact_path, default_prompt};

    #[test]
    fn abbreviates_home_and_deep_paths() {
        let home = Some("/Users/nishimura");

        assert_eq!(compact_path(Path::new("/Users/nishimura"), home), "~");
        assert_eq!(
            compact_path(Path::new("/Users/nishimura/src"), home),
            "~/src"
        );
        assert_eq!(
            compact_path(
                Path::new("/Users/nishimura/Code/github.com/ubugeeei/ubshell"),
                home
            ),
            "~/.../ubugeeei/ubshell"
        );
        assert_eq!(
            compact_path(Path::new("/usr/local/bin"), home),
            "/.../local/bin"
        );
    }

    #[test]
    fn formats_default_prompt_with_short_path() {
        let prompt = default_prompt(
            Path::new("/Users/nishimura/Code/github.com/ubugeeei/ubshell"),
            Some("/Users/nishimura"),
            0,
        );

        assert_eq!(prompt, "ush ~/.../ubugeeei/ubshell $ ");
    }
}
