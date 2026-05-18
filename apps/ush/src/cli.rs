use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ush", version, about = "ush (ubugeeei shell)")]
pub struct Cli {
    /// Force the stylish (human-formatted) output renderer.
    #[arg(short = 's', long = "stylish")]
    pub stylish: bool,

    /// Force plain (POSIX-friendly) output, overriding stylish.
    #[arg(long = "plain")]
    pub plain: bool,

    /// Suppress interactive prompts (the same as USH_INTERACTION=false).
    #[arg(long = "no-interaction")]
    pub no_interaction: bool,

    /// Path to the ush config file. Defaults to ~/.config/ush/config.toml.
    #[arg(long = "config", value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Treat this invocation as a login shell (loads the profile file).
    #[arg(short = 'l', long = "login")]
    pub login: bool,

    /// Skip the profile file even in login mode.
    #[arg(long = "no-profile", conflicts_with = "profile_file")]
    pub no_profile: bool,

    /// Use FILE as the profile file instead of the default lookup.
    #[arg(
        long = "profile-file",
        value_name = "FILE",
        conflicts_with = "no_profile"
    )]
    pub profile_file: Option<PathBuf>,

    /// Skip the rc file (~/.config/ush/.config.ush etc.).
    #[arg(long = "no-rc", conflicts_with = "rc_file")]
    pub no_rc: bool,

    /// Use FILE as the rc file instead of the default lookup.
    #[arg(long = "rc-file", value_name = "FILE", conflicts_with = "no_rc")]
    pub rc_file: Option<PathBuf>,

    /// Run COMMAND non-interactively and exit (POSIX `sh -c`).
    #[arg(short = 'c', value_name = "COMMAND")]
    pub command: Option<String>,

    /// Print the parsed `.ush` AST and exit (debugging aid).
    #[arg(long = "print-ast")]
    pub print_ast: bool,

    #[command(subcommand)]
    pub action: Option<Action>,

    /// `.ush` or `.sh` script to execute (positional).
    #[arg(value_name = "SCRIPT")]
    pub script: Option<PathBuf>,

    /// Arguments forwarded to the script as `$1`, `$2`, ….
    #[arg(
        value_name = "ARGS",
        allow_hyphen_values = true,
        trailing_var_arg = true
    )]
    pub script_args: Vec<String>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    /// Lower a `.ush` source file to POSIX `sh`.
    Compile {
        input: PathBuf,
        /// Write generated `sh` to FILE instead of stdout.
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Also emit a sourcemap to FILE.
        #[arg(long = "sourcemap", value_name = "FILE")]
        sourcemap: Option<PathBuf>,
    },
    /// Format a `.ush` source file in-place (or print the result).
    Format {
        input: PathBuf,
        /// Exit non-zero if the file is not already formatted.
        #[arg(long)]
        check: bool,
        /// Print the formatted file to stdout instead of writing back.
        #[arg(long)]
        stdout: bool,
    },
    /// Type-check a `.ush` source file without producing output.
    Check { input: PathBuf },
    /// Run inline `#[test]` blocks in one or more `.ush` files.
    Test {
        #[arg(value_name = "TARGET")]
        targets: Vec<String>,
    },
}
