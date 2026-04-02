use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::Shell;

#[derive(Debug, Clone, Default)]
pub struct SessionStartup {
    pub load_profile: bool,
    pub load_rc: bool,
    pub profile_file: Option<PathBuf>,
    pub rc_file: Option<PathBuf>,
}

#[derive(Clone, Copy)]
enum StartupKind {
    Profile,
    Rc,
}

struct StartupEntry {
    path: PathBuf,
    required: bool,
    label: &'static str,
}

impl Shell {
    pub fn load_session_startup(&mut self, startup: &SessionStartup) -> Result<()> {
        for entry in self.startup_entries(StartupKind::Profile, startup) {
            self.load_startup_entry(entry)?;
        }
        for entry in self.startup_entries(StartupKind::Rc, startup) {
            self.load_startup_entry(entry)?;
        }
        Ok(())
    }

    fn load_startup_entry(&mut self, entry: StartupEntry) -> Result<()> {
        if !entry.path.exists() {
            if entry.required {
                bail!("missing {} file: {}", entry.label, entry.path.display());
            }
            return Ok(());
        }

        self.last_status = self
            .source_path(&entry.path)
            .with_context(|| format!("failed to load {} {}", entry.label, entry.path.display()))?;
        Ok(())
    }

    fn startup_entries(&self, kind: StartupKind, startup: &SessionStartup) -> Vec<StartupEntry> {
        let mut entries = Vec::new();
        let mut seen = BTreeSet::new();

        let (enabled, explicit, config_paths, defaults, label) = match kind {
            StartupKind::Profile => (
                startup.load_profile || startup.profile_file.is_some(),
                startup.profile_file.as_ref(),
                &self.config.shell.profile_files,
                self.default_profile_candidates(),
                "profile",
            ),
            StartupKind::Rc => (
                startup.load_rc || startup.rc_file.is_some(),
                startup.rc_file.as_ref(),
                &self.config.shell.rc_files,
                self.default_rc_candidates(),
                "rc",
            ),
        };

        if !enabled {
            return entries;
        }

        if let Some(path) = explicit {
            push_unique(
                &mut entries,
                &mut seen,
                StartupEntry {
                    path: self.resolve_cli_startup_path(path),
                    required: true,
                    label,
                },
            );
        }

        for path in config_paths {
            push_unique(
                &mut entries,
                &mut seen,
                StartupEntry {
                    path: self.resolve_config_startup_path(path),
                    required: true,
                    label,
                },
            );
        }

        for path in defaults {
            push_unique(
                &mut entries,
                &mut seen,
                StartupEntry {
                    path,
                    required: false,
                    label,
                },
            );
        }

        entries
    }

    fn default_profile_candidates(&self) -> Vec<PathBuf> {
        let mut paths = vec![self.paths.config_dir.join("profile.sh")];
        if let Some(home) = self.home_dir() {
            paths.push(home.join(".ush_profile"));
        }
        paths
    }

    fn default_rc_candidates(&self) -> Vec<PathBuf> {
        let mut paths = vec![self.paths.config_dir.join("rc.sh")];
        if let Some(home) = self.home_dir() {
            paths.push(home.join(".ushrc"));
        }
        paths
    }

    fn resolve_config_startup_path(&self, path: &Path) -> PathBuf {
        let expanded = self.expand_startup_path(path);
        if expanded.is_absolute() {
            expanded
        } else {
            self.paths.config_dir.join(expanded)
        }
    }

    fn resolve_cli_startup_path(&self, path: &Path) -> PathBuf {
        let expanded = self.expand_startup_path(path);
        if expanded.is_absolute() {
            expanded
        } else {
            self.cwd.join(expanded)
        }
    }

    fn expand_startup_path(&self, path: &Path) -> PathBuf {
        let value = path.to_string_lossy();
        if value == "~"
            && let Some(home) = self.home_dir()
        {
            return home;
        }
        if let Some(rest) = value.strip_prefix("~/")
            && let Some(home) = self.home_dir()
        {
            return home.join(rest);
        }
        path.to_path_buf()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        self.env.get("HOME").map(PathBuf::from)
    }
}

fn push_unique(entries: &mut Vec<StartupEntry>, seen: &mut BTreeSet<PathBuf>, entry: StartupEntry) {
    if seen.insert(entry.path.clone()) {
        entries.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tempfile::tempdir;
    use ush_config::UshConfig;

    use crate::{SessionStartup, Shell, ShellOptions};

    fn new_shell() -> Shell {
        Shell::new(
            UshConfig::default(),
            ShellOptions {
                stylish: false,
                interaction: false,
                print_ast: false,
            },
        )
        .expect("shell")
    }

    #[test]
    fn explicit_startup_files_load_before_command_execution() {
        let dir = tempdir().expect("tempdir");
        let profile = dir.path().join("profile.sh");
        let rc = dir.path().join("rc.sh");
        fs::write(&profile, "export PROFILE_LOADED=profile\n").expect("write profile");
        fs::write(&rc, "alias ll='echo rc-loaded'\n").expect("write rc");

        let mut shell = new_shell();
        shell
            .load_session_startup(&SessionStartup {
                load_profile: true,
                load_rc: true,
                profile_file: Some(profile),
                rc_file: Some(rc),
            })
            .expect("load startup");

        assert_eq!(
            shell.env.get("PROFILE_LOADED"),
            Some(&"profile".to_string())
        );
        assert_eq!(shell.aliases.get("ll"), Some(&"echo rc-loaded".to_string()));
    }

    #[test]
    fn config_relative_startup_paths_resolve_from_config_dir() {
        let dir = tempdir().expect("tempdir");
        let config_dir = dir.path().join("config");
        fs::create_dir_all(&config_dir).expect("mkdir config");
        fs::write(config_dir.join("team.rc"), "export TEAM_RC=loaded\n").expect("write rc");

        let mut config = UshConfig::default();
        config.shell.rc_files.push(PathBuf::from("team.rc"));
        let mut shell = Shell::new(
            config,
            ShellOptions {
                stylish: false,
                interaction: false,
                print_ast: false,
            },
        )
        .expect("shell");
        shell.paths.config_dir = config_dir;

        shell
            .load_session_startup(&SessionStartup {
                load_profile: false,
                load_rc: true,
                profile_file: None,
                rc_file: None,
            })
            .expect("load rc");

        assert_eq!(shell.env.get("TEAM_RC"), Some(&"loaded".to_string()));
    }

    #[test]
    fn default_candidates_load_from_config_dir_and_home() {
        let dir = tempdir().expect("tempdir");
        let config_dir = dir.path().join("config");
        let home_dir = dir.path().join("home");
        fs::create_dir_all(&config_dir).expect("mkdir config");
        fs::create_dir_all(&home_dir).expect("mkdir home");
        fs::write(
            config_dir.join("profile.sh"),
            "export PROFILE_ORDER=config\n",
        )
        .expect("write config profile");
        fs::write(home_dir.join(".ush_profile"), "export PROFILE_ORDER=home\n")
            .expect("write home profile");
        fs::write(config_dir.join("rc.sh"), "alias ll='echo config'\n").expect("write config rc");
        fs::write(home_dir.join(".ushrc"), "alias ll='echo home'\n").expect("write home rc");

        let mut shell = new_shell();
        shell.paths.config_dir = config_dir;
        shell
            .env
            .insert("HOME".to_string(), home_dir.display().to_string());

        shell
            .load_session_startup(&SessionStartup {
                load_profile: true,
                load_rc: true,
                profile_file: None,
                rc_file: None,
            })
            .expect("load startup");

        assert_eq!(shell.env.get("PROFILE_ORDER"), Some(&"home".to_string()));
        assert_eq!(shell.aliases.get("ll"), Some(&"echo home".to_string()));
    }

    #[test]
    fn missing_explicit_startup_file_is_reported() {
        let mut shell = new_shell();
        let error = shell
            .load_session_startup(&SessionStartup {
                load_profile: true,
                load_rc: false,
                profile_file: Some(PathBuf::from("missing-profile.sh")),
                rc_file: None,
            })
            .expect_err("missing file should fail");

        assert!(error.to_string().contains("missing profile file"));
    }
}
