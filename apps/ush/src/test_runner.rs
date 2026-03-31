use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

pub fn run(targets: &[String], config: Option<&Path>) -> Result<i32> {
    let cwd = env::current_dir().context("failed to read current directory")?;
    let tests = discover_tests(targets, &cwd)?;
    if tests.is_empty() {
        eprintln!("ush test: no test scripts found");
        return Ok(1);
    }

    let exe = env::current_exe().context("failed to locate ush executable")?;
    let mut passed = 0usize;
    let mut failed = 0usize;

    for script in tests {
        let output = command_for_script(&exe, &script, config)
            .output()
            .with_context(|| format!("failed to run {}", script.display()))?;
        let label = display_path(&script, &cwd);
        if output.status.success() {
            println!("ok   {label}");
            passed += 1;
            continue;
        }

        println!("fail {label}");
        print_stream("stdout", &output.stdout);
        print_stream("stderr", &output.stderr);
        failed += 1;
    }

    println!("\n{passed} passed; {failed} failed");
    Ok(if failed == 0 { 0 } else { 1 })
}

fn discover_tests(targets: &[String], cwd: &Path) -> Result<Vec<PathBuf>> {
    let mut files = BTreeSet::new();
    if targets.is_empty() {
        let tests_dir = cwd.join("tests");
        if tests_dir.is_dir() {
            collect_ush_files(&tests_dir, &mut files)?;
        }
        return Ok(files.into_iter().collect());
    }

    for target in targets {
        let path = cwd.join(target);
        if path.exists() {
            collect_target(&path, &mut files)?;
            continue;
        }
        for entry in glob::glob(target).with_context(|| format!("invalid glob: {target}"))? {
            let path = entry.with_context(|| format!("invalid path from glob: {target}"))?;
            collect_target(&path, &mut files)?;
        }
    }
    Ok(files.into_iter().collect())
}

fn collect_target(path: &Path, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    if path.is_dir() {
        collect_ush_files(path, files)?;
        return Ok(());
    }
    if is_ush(path) {
        files.insert(fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()));
        return Ok(());
    }
    bail!(
        "ush test: expected a .ush file or directory: {}",
        path.display()
    )
}

fn collect_ush_files(dir: &Path, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_ush_files(&path, files)?;
        } else if is_ush(&path) {
            files.insert(fs::canonicalize(&path).unwrap_or(path));
        }
    }
    Ok(())
}

fn command_for_script(exe: &Path, script: &Path, config: Option<&Path>) -> Command {
    let mut command = Command::new(exe);
    command.arg("--plain").arg("--no-interaction");
    if let Some(config) = config {
        command.arg("--config").arg(config);
    }
    command.arg(script);
    command
}

fn display_path<'a>(path: &'a Path, cwd: &'a Path) -> String {
    path.strip_prefix(cwd).unwrap_or(path).display().to_string()
}

fn print_stream(label: &str, bytes: &[u8]) {
    if bytes.is_empty() {
        return;
    }
    let text = String::from_utf8_lossy(bytes);
    for line in text.lines() {
        println!("  {label}: {line}");
    }
}

fn is_ush(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("ush")
}
