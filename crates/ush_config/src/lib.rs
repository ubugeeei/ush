mod starship;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use serde::Deserialize;

pub use self::starship::StarshipPromptConfig;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UshConfig {
    #[serde(default)]
    pub shell: ShellConfig,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShellConfig {
    #[serde(default, alias = "stylishDefault")]
    pub stylish_default: bool,
    #[serde(default = "default_interaction")]
    pub interaction: bool,
    #[serde(default = "default_history_size", alias = "historySize")]
    pub history_size: usize,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(skip)]
    pub starship: Option<StarshipPromptConfig>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            stylish_default: false,
            interaction: default_interaction(),
            history_size: default_history_size(),
            prompt: None,
            starship: None,
        }
    }
}

fn default_interaction() -> bool {
    true
}

fn default_history_size() -> usize {
    5_000
}

#[derive(Debug, Clone)]
pub struct RuntimePaths {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub history_file: PathBuf,
}

impl UshConfig {
    pub fn load(explicit: Option<&Path>) -> Result<Self> {
        let candidates = config_candidates(explicit);
        let mut config = Self::default();

        for candidate in candidates {
            if !candidate.exists() {
                continue;
            }

            config = load_one(&candidate)
                .with_context(|| format!("failed to load config from {}", candidate.display()))?;
            break;
        }

        config.shell.starship = starship::load_starship_prompt()?;
        Ok(config)
    }

    pub fn runtime_paths() -> Result<RuntimePaths> {
        let dirs = ProjectDirs::from("dev", "ubugeeei", "ush")
            .ok_or_else(|| anyhow!("failed to resolve ush project directories"))?;

        let config_dir = dirs.config_dir().to_path_buf();
        let cache_dir = dirs.cache_dir().to_path_buf();

        fs::create_dir_all(&config_dir)
            .with_context(|| format!("failed to create {}", config_dir.display()))?;
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("failed to create {}", cache_dir.display()))?;

        Ok(RuntimePaths {
            history_file: cache_dir.join("history.txt"),
            config_dir,
            cache_dir,
        })
    }
}

fn config_candidates(explicit: Option<&Path>) -> Vec<PathBuf> {
    if let Some(path) = explicit {
        return vec![path.to_path_buf()];
    }

    let mut candidates = Vec::new();
    if let Ok(paths) = UshConfig::runtime_paths() {
        candidates.push(paths.config_dir.join("config.pkl"));
        candidates.push(paths.config_dir.join("config.json"));
    }
    if let Some(legacy_dirs) = ProjectDirs::from("dev", "ubugeeei", "ubsh") {
        candidates.push(legacy_dirs.config_dir().join("config.pkl"));
        candidates.push(legacy_dirs.config_dir().join("config.json"));
    }
    candidates
}

fn load_one(path: &Path) -> Result<UshConfig> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            let source = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&source)?)
        }
        Some("pkl") => load_pkl(path),
        _ => Err(anyhow!("unsupported config format: {}", path.display())),
    }
}

fn load_pkl(path: &Path) -> Result<UshConfig> {
    let attempts = [
        ["eval", "--format", "json"],
        ["eval", "--output-format", "json"],
        ["eval", "-f", "json"],
    ];

    for attempt in attempts {
        let output = Command::new("pkl").args(attempt).arg(path).output();
        let Ok(output) = output else {
            continue;
        };

        if !output.status.success() {
            continue;
        }

        let stdout = String::from_utf8(output.stdout).context("pkl output was not utf-8")?;
        return Ok(serde_json::from_str(&stdout)?);
    }

    Err(anyhow!(
        "failed to evaluate {} with pkl; install `pkl` or provide config.json",
        path.display()
    ))
}
