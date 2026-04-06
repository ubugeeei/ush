use rustyline::{Context, completion::Completer, history::DefaultHistory};
use tempfile::tempdir;

use super::*;

#[test]
fn completes_additional_tool_commands_and_scripts() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("package.json"),
        r#"{"scripts":{"build":"vite build","lint":"eslint .","test:unit":"vitest"}}"#,
    )
    .expect("write package");
    let history = DefaultHistory::new();
    let ctx = Context::new(&history);
    let helper = helper(dir.path());

    let (_, cargo_pairs) = helper
        .complete("cargo bu", 8, &ctx)
        .expect("cargo commands");
    let (_, moon_pairs) = helper.complete("moon ru", 7, &ctx).expect("moon commands");
    let (_, nix_pairs) = helper.complete("nix fl", 6, &ctx).expect("nix commands");
    let (_, nix_flake_pairs) = helper
        .complete("nix flake sh", 12, &ctx)
        .expect("nix flake commands");
    let (_, nix_store_pairs) = helper
        .complete("nix store de", 12, &ctx)
        .expect("nix store commands");
    let (_, go_pairs) = helper.complete("go mo", 5, &ctx).expect("go commands");
    let (_, zig_pairs) = helper.complete("zig bu", 6, &ctx).expect("zig commands");
    let (_, bun_pairs) = helper
        .complete("bun run li", 10, &ctx)
        .expect("bun scripts");
    let (_, pnpm_pairs) = helper
        .complete("pnpm run te", 11, &ctx)
        .expect("pnpm scripts");
    let (_, yarn_pairs) = helper.complete("yarn ru", 7, &ctx).expect("yarn commands");
    let (_, claude_pairs) = helper
        .complete("claude up", 9, &ctx)
        .expect("claude commands");
    let (_, codex_pairs) = helper
        .complete("codex re", 8, &ctx)
        .expect("codex commands");

    assert!(cargo_pairs.iter().any(|pair| pair.replacement == "build"));
    assert!(moon_pairs.iter().any(|pair| pair.replacement == "run"));
    assert!(nix_pairs.iter().any(|pair| pair.replacement == "flake"));
    assert!(
        nix_pairs
            .iter()
            .any(|pair| pair.display.contains("manage Nix flakes"))
    );
    assert!(
        nix_flake_pairs
            .iter()
            .any(|pair| pair.replacement == "show")
    );
    assert!(
        nix_store_pairs
            .iter()
            .any(|pair| pair.replacement == "delete")
    );
    assert!(go_pairs.iter().any(|pair| pair.replacement == "mod"));
    assert!(zig_pairs.iter().any(|pair| pair.replacement == "build"));
    assert!(bun_pairs.iter().any(|pair| pair.replacement == "lint"));
    assert!(
        pnpm_pairs
            .iter()
            .any(|pair| pair.replacement == "test:unit")
    );
    assert!(yarn_pairs.iter().any(|pair| pair.replacement == "run"));
    assert!(claude_pairs.iter().any(|pair| pair.replacement == "update"));
    assert!(codex_pairs.iter().any(|pair| pair.replacement == "review"));
}

#[test]
fn discovers_tasks_across_supported_sources() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("Makefile"),
        ".PHONY: build test\nbuild:\n\t@echo build\ntest:\n\t@echo test\n",
    )
    .expect("write makefile");
    fs::write(dir.path().join("justfile"), "fmt:\n  echo fmt\n").expect("write justfile");
    fs::write(
        dir.path().join("mise.toml"),
        "[tasks.lint]\nrun = \"cargo clippy\"\n",
    )
    .expect("write mise toml");
    fs::write(
        dir.path().join("package.json"),
        r#"{"scripts":{"build":"vite build"},"devDependencies":{"vite":"^7.0.0"}}"#,
    )
    .expect("write package");

    let entries = super::super::discover_tasks(dir.path());

    assert!(entries.iter().any(|entry| entry.command() == "make build"));
    assert!(entries.iter().any(|entry| entry.command() == "just fmt"));
    assert!(
        entries
            .iter()
            .any(|entry| entry.command() == "mise run lint")
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry.command() == "npm run build")
    );
    assert!(entries.iter().any(|entry| entry.command() == "vp dev"));
}
