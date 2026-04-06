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
            paths.push(home.join(".config.ush"));
        }
        paths.push(self.paths.config_dir.join(".config.ush"));
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
mod tests;
