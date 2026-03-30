use std::collections::BTreeMap;

use crate::commands::{CommandLookup, lookup_command};

pub(super) enum LookupStyle {
    Path,
    Verbose,
}

pub(super) fn describe_commands(
    aliases: &BTreeMap<String, String>,
    names: &[String],
    style: LookupStyle,
) -> (String, i32) {
    let mut lines = Vec::new();
    let mut status = 0;

    for name in names {
        match lookup_command(name, aliases) {
            Some(CommandLookup::Alias(alias)) => lines.push(match style {
                LookupStyle::Path => format_alias(name, &alias),
                LookupStyle::Verbose => format!("{name} is aliased to `{alias}`"),
            }),
            Some(CommandLookup::Builtin) => lines.push(match style {
                LookupStyle::Path => name.clone(),
                LookupStyle::Verbose => format!("{name} is a shell builtin"),
            }),
            Some(CommandLookup::External(path)) => lines.push(match style {
                LookupStyle::Path => path.display().to_string(),
                LookupStyle::Verbose => format!("{name} is {}", path.display()),
            }),
            None => {
                eprintln!("ush: {name}: not found");
                status = 1;
            }
        }
    }

    let text = if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    };
    (text, status)
}

fn format_alias(name: &str, value: &str) -> String {
    format!("alias {name}='{}'", value.replace('\'', r#"'\''"#))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{LookupStyle, describe_commands};

    #[test]
    fn renders_aliases_in_short_form() {
        let mut aliases = BTreeMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());

        let (text, status) = describe_commands(&aliases, &[String::from("ll")], LookupStyle::Path);

        assert_eq!(status, 0);
        assert_eq!(text, "alias ll='ls -la'\n");
    }
}
