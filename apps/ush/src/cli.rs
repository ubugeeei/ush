use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ush", version, about = "ush (ubugeeei shell)")]
pub struct Cli {
    #[arg(short = 's', long = "stylish")]
    pub stylish: bool,

    #[arg(long = "plain")]
    pub plain: bool,

    #[arg(long = "no-interaction")]
    pub no_interaction: bool,

    #[arg(long = "config", value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[arg(short = 'l', long = "login")]
    pub login: bool,

    #[arg(long = "no-profile", conflicts_with = "profile_file")]
    pub no_profile: bool,

    #[arg(
        long = "profile-file",
        value_name = "FILE",
        conflicts_with = "no_profile"
    )]
    pub profile_file: Option<PathBuf>,

    #[arg(long = "no-rc", conflicts_with = "rc_file")]
    pub no_rc: bool,

    #[arg(long = "rc-file", value_name = "FILE", conflicts_with = "no_rc")]
    pub rc_file: Option<PathBuf>,

    #[arg(short = 'c', value_name = "COMMAND")]
    pub command: Option<String>,

    #[arg(long = "print-ast")]
    pub print_ast: bool,

    #[command(subcommand)]
    pub action: Option<Action>,

    #[arg(value_name = "SCRIPT")]
    pub script: Option<PathBuf>,

    #[arg(
        value_name = "ARGS",
        allow_hyphen_values = true,
        trailing_var_arg = true
    )]
    pub script_args: Vec<String>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Action {
    Compile {
        input: PathBuf,
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        #[arg(long = "sourcemap", value_name = "FILE")]
        sourcemap: Option<PathBuf>,
    },
    Format {
        input: PathBuf,
        #[arg(long)]
        check: bool,
        #[arg(long)]
        stdout: bool,
    },
    Check {
        input: PathBuf,
    },
    Test {
        #[arg(value_name = "TARGET")]
        targets: Vec<String>,
    },
}
