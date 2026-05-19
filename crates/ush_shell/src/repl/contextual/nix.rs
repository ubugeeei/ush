use compact_str::CompactString;
use phf::phf_map;

use super::{
    candidates::{described_candidate_pairs, typed_candidate_pairs},
    options::{OptionSpec, option_spec, pending_value_kind, positional_args},
    types::ContextualCompletion,
};
use crate::repl::descriptions;

const NIX_COMMANDS: &[&str] = &[
    "help",
    "help-stores",
    "build",
    "develop",
    "flake",
    "profile",
    "run",
    "search",
    "repl",
    "bundle",
    "copy",
    "edit",
    "eval",
    "fmt",
    "log",
    "path-info",
    "registry",
    "why-depends",
    "config",
    "daemon",
    "derivation",
    "env",
    "hash",
    "key",
    "nar",
    "print-dev-env",
    "realisation",
    "store",
    "upgrade-nix",
];
const NIX_OPTIONS: &[&str] = &[
    "--help",
    "--version",
    "--offline",
    "--refresh",
    "--option",
    "--log-format",
    "-L",
    "--print-build-logs",
    "--debug",
    "--quiet",
    "-v",
    "--verbose",
    "--extra-experimental-features",
    "--accept-flake-config",
    "--impure",
    "--system",
    "-f",
    "--file",
    "--expr",
    "--arg",
    "--argstr",
    "-I",
    "--override-flake",
];
const NIX_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["--log-format", "--system", "--expr"], 1, false, false),
    option_spec(
        &["--option", "--arg", "--argstr", "--override-flake"],
        2,
        false,
        false,
    ),
    option_spec(&["-f", "--file", "-I"], 1, true, true),
];
const NIX_FLAKE_COMMANDS: &[&str] = &[
    "archive", "check", "clone", "info", "init", "lock", "metadata", "new", "prefetch", "show",
    "update",
];
const NIX_PROFILE_COMMANDS: &[&str] = &[
    "diff-closures",
    "history",
    "install",
    "list",
    "remove",
    "rollback",
    "upgrade",
    "wipe-history",
];
const NIX_STORE_COMMANDS: &[&str] = &[
    "add",
    "add-file",
    "add-path",
    "cat",
    "copy-log",
    "copy-sigs",
    "delete",
    "diff-closures",
    "dump-path",
    "gc",
    "info",
    "ls",
    "make-content-addressed",
    "optimise",
    "path-from-hash-part",
    "ping",
    "prefetch-file",
    "repair",
    "sign",
    "verify",
];

static NIX_COMMAND_HELP: phf::Map<&'static str, &'static str> = phf_map! {
    "help" => "show help about nix commands",
    "help-stores" => "show help about Nix store backends",
    "build" => "build a derivation or fetch a store path",
    "develop" => "start a shell for a derivation build env",
    "flake" => "manage Nix flakes",
    "profile" => "manage Nix profiles",
    "run" => "run a Nix application",
    "search" => "search for packages",
    "repl" => "start an interactive Nix repl",
    "bundle" => "bundle an app outside the Nix store",
    "copy" => "copy paths between Nix stores",
    "edit" => "open a package expression in $EDITOR",
    "eval" => "evaluate a Nix expression",
    "fmt" => "format Nix code",
    "log" => "show build logs",
    "path-info" => "query Nix store path metadata",
    "registry" => "manage the flake registry",
    "why-depends" => "show why a package depends on another",
    "config" => "manipulate the Nix configuration",
    "daemon" => "run the Nix daemon",
    "derivation" => "work with derivations",
    "env" => "manipulate the process environment",
    "hash" => "compute and convert hashes",
    "key" => "generate and convert signing keys",
    "nar" => "create or inspect NAR archives",
    "print-dev-env" => "print shell code for a dev env",
    "realisation" => "manipulate a Nix realisation",
    "store" => "manipulate a Nix store",
    "upgrade-nix" => "upgrade Nix to the latest stable release",
};

pub(crate) fn complete_nix(args: &[CompactString], word: &str) -> Option<ContextualCompletion> {
    let pending = pending_value_kind(args, NIX_OPTION_SPECS);
    if matches!(pending, Some(true)) {
        return Some(ContextualCompletion::Path);
    }
    if pending.is_some() {
        return None;
    }
    if word.starts_with('-') {
        return Some(ContextualCompletion::Pairs(typed_candidate_pairs(
            word,
            NIX_OPTIONS.iter().copied(),
            descriptions::OPTION,
        )));
    }

    let positionals = positional_args(args, NIX_OPTION_SPECS);
    let pairs = match scope(&positionals) {
        NixScope::TopLevel => {
            described_candidate_pairs(word, NIX_COMMANDS.iter().copied(), |item| {
                NIX_COMMAND_HELP
                    .get(item)
                    .copied()
                    .or(Some(descriptions::NIX_COMMAND))
            })
        }
        NixScope::Flake => typed_candidate_pairs(
            word,
            NIX_FLAKE_COMMANDS.iter().copied(),
            descriptions::NIX_FLAKE_COMMAND,
        ),
        NixScope::Profile => typed_candidate_pairs(
            word,
            NIX_PROFILE_COMMANDS.iter().copied(),
            descriptions::NIX_PROFILE_COMMAND,
        ),
        NixScope::Store => typed_candidate_pairs(
            word,
            NIX_STORE_COMMANDS.iter().copied(),
            descriptions::NIX_STORE_COMMAND,
        ),
    };
    Some(ContextualCompletion::Pairs(pairs))
}

enum NixScope {
    TopLevel,
    Flake,
    Profile,
    Store,
}

fn scope(args: &[CompactString]) -> NixScope {
    match args.first().map(CompactString::as_str) {
        Some("help") => match args.get(1).map(CompactString::as_str) {
            Some("flake") => NixScope::Flake,
            Some("profile") => NixScope::Profile,
            Some("store") => NixScope::Store,
            _ => NixScope::TopLevel,
        },
        Some("flake") => NixScope::Flake,
        Some("profile") => NixScope::Profile,
        Some("store") => NixScope::Store,
        _ => NixScope::TopLevel,
    }
}
