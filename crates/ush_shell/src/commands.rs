use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{Result, anyhow};
use which::{Error as WhichError, which, which_all};

pub(crate) const BUILTIN_COMMANDS: &[&str] = &[
    ":", ".", "[", "alias", "bg", "cd", "command", "confirm", "disown", "echo", "env", "exit",
    "export", "false", "fg", "fsam", "glob", "help", "history", "input", "jobs", "port", "pwd",
    "rm", "sammary", "select", "source", "stop", "test", "true", "type", "unalias", "unset",
    "wait", "which",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CommandLookup {
    Alias(String),
    Builtin,
    External(PathBuf),
}

pub(crate) fn is_builtin(command: &str) -> bool {
    BUILTIN_COMMANDS.contains(&command)
}

pub(crate) fn lookup_command(
    command: &str,
    aliases: &BTreeMap<String, String>,
) -> Option<CommandLookup> {
    lookup_all_commands(command, aliases).into_iter().next()
}

pub(crate) fn lookup_all_commands(
    command: &str,
    aliases: &BTreeMap<String, String>,
) -> Vec<CommandLookup> {
    let mut lookups = Vec::new();

    if let Some(alias) = aliases.get(command) {
        lookups.push(CommandLookup::Alias(alias.clone()));
    }
    if is_builtin(command) {
        lookups.push(CommandLookup::Builtin);
    }
    lookups.extend(find_all_external_commands(command).into_iter().map(CommandLookup::External));

    lookups
}

pub(crate) fn find_external_command(command: &str) -> Option<PathBuf> {
    which(command).ok()
}

pub(crate) fn find_all_external_commands(command: &str) -> Vec<PathBuf> {
    match which_all(command) {
        Ok(paths) => {
            let mut unique = Vec::new();
            for path in paths {
                if !unique.contains(&path) {
                    unique.push(path);
                }
            }
            unique
        }
        Err(WhichError::CannotFindBinaryPath) => Vec::new(),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn ensure_external_command(command: &str) -> Result<()> {
    if find_external_command(command).is_some() {
        return Ok(());
    }
    Err(anyhow!("command not found: {command}"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{CommandLookup, is_builtin, lookup_command};

    #[test]
    fn recognizes_builtin_names() {
        assert!(is_builtin("echo"));
        assert!(is_builtin("test"));
        assert!(!is_builtin("definitely-not-a-real-command"));
    }

    #[test]
    fn aliases_take_priority_in_lookup() {
        let mut aliases = BTreeMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());

        assert_eq!(
            lookup_command("ll", &aliases),
            Some(CommandLookup::Alias("ls -la".to_string()))
        );
    }
}
