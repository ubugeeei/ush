use std::{
    fs,
    path::{Path, PathBuf},
};

use compact_str::CompactString;
use rustc_hash::FxHashSet;
use serde_json::Value as JsonValue;
use smallvec::SmallVec;

use super::{
    catalog::VP_COMMANDS,
    options::explicit_makefile_path,
    parse::{
        package_json_has_name, package_json_scripts, package_json_uses_vp, parse_just_recipes,
        parse_make_targets, parse_mise_toml_tasks, push_unique,
    },
    types::{Names, TaskEntry, TaskSource},
};

pub(crate) fn discover_tasks(cwd: &Path) -> Vec<TaskEntry> {
    let mut entries = SmallVec::<[TaskEntry; 16]>::new();
    append_tasks(&mut entries, TaskSource::Make, load_make_targets(cwd, &[]));
    append_tasks(&mut entries, TaskSource::Just, load_just_recipes(cwd));
    append_tasks(&mut entries, TaskSource::Mise, load_mise_tasks(cwd));
    append_tasks(&mut entries, TaskSource::Npm, load_npm_scripts(cwd));
    append_tasks(&mut entries, TaskSource::Vp, load_vp_tasks(cwd));

    let mut entries = entries.into_vec();
    entries.sort_unstable();
    entries.dedup();
    entries
}

pub(crate) fn load_make_targets(cwd: &Path, args: &[CompactString]) -> Names {
    let path = explicit_makefile_path(cwd, args).or_else(|| {
        ["GNUmakefile", "makefile", "Makefile"]
            .into_iter()
            .map(|name| cwd.join(name))
            .find(|path| path.is_file())
    });
    path.and_then(|path| fs::read_to_string(path).ok())
        .map(|source| parse_make_targets(&source))
        .unwrap_or_default()
}

pub(crate) fn load_just_recipes(cwd: &Path) -> Names {
    find_upward(cwd, &["justfile", ".justfile", "Justfile"])
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|source| parse_just_recipes(&source))
        .unwrap_or_default()
}

pub(crate) fn load_mise_tasks(cwd: &Path) -> Names {
    let mut dirs = SmallVec::<[&Path; 16]>::new();
    dirs.extend(cwd.ancestors());

    let mut seen = FxHashSet::default();
    let mut names = Names::new();
    for dir in dirs.into_iter().rev() {
        for file in ["mise.toml", ".mise.toml"] {
            let path = dir.join(file);
            if let Ok(source) = fs::read_to_string(path) {
                for name in parse_mise_toml_tasks(&source) {
                    push_unique(&name, &mut seen, &mut names);
                }
            }
        }
        for root in [
            "mise-tasks",
            ".mise-tasks",
            ".mise/tasks",
            "mise/tasks",
            ".config/mise/tasks",
        ] {
            let task_root = dir.join(root);
            collect_mise_task_scripts(&task_root, &task_root, &mut seen, &mut names);
        }
    }

    names.sort_unstable();
    names
}

pub(crate) fn load_npm_scripts(cwd: &Path) -> Names {
    load_package_json(cwd)
        .map(|json| package_json_scripts(&json))
        .unwrap_or_default()
}

pub(crate) fn load_vp_tasks(cwd: &Path) -> Names {
    if !has_vp_project(cwd) {
        return Names::new();
    }
    let mut names = Names::new();
    for command in VP_COMMANDS {
        names.push(CompactString::from(*command));
    }
    names
}

pub(crate) fn find_upward(cwd: &Path, names: &[&str]) -> Option<PathBuf> {
    for dir in cwd.ancestors() {
        for name in names {
            let path = dir.join(name);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn append_tasks(entries: &mut SmallVec<[TaskEntry; 16]>, source: TaskSource, names: Names) {
    for name in names {
        entries.push(TaskEntry::new(source, name));
    }
}

fn collect_mise_task_scripts(
    base: &Path,
    dir: &Path,
    seen: &mut FxHashSet<CompactString>,
    names: &mut Names,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_mise_task_scripts(base, &path, seen, names);
            continue;
        }
        let Ok(relative) = path.strip_prefix(base) else {
            continue;
        };
        let raw = relative.to_string_lossy();
        if raw.is_empty() {
            continue;
        }
        if raw.contains('\\') {
            let normalized = raw.replace('\\', "/");
            push_unique(&normalized, seen, names);
        } else {
            push_unique(raw.as_ref(), seen, names);
        }
    }
}

fn load_package_json(cwd: &Path) -> Option<JsonValue> {
    let path = find_upward(cwd, &["package.json"])?;
    let source = fs::read_to_string(path).ok()?;
    serde_json::from_str(&source).ok()
}

fn has_vp_project(cwd: &Path) -> bool {
    if find_upward(
        cwd,
        &[
            "vite.config.js",
            "vite.config.cjs",
            "vite.config.mjs",
            "vite.config.ts",
            "vite.config.cts",
            "vite.config.mts",
        ],
    )
    .is_some()
    {
        return true;
    }

    let Some(json) = load_package_json(cwd) else {
        return false;
    };
    package_json_has_name(&json, "vite") || package_json_uses_vp(&json)
}
