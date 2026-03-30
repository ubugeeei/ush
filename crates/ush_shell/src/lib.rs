mod builtins;
mod execute;
mod expand;
mod helpers;
mod options;
mod parser;
mod process;
mod prompt;
mod repl;
mod style;

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use ush_config::{RuntimePaths, UshConfig};

pub use self::{
    helpers::{HelperInvocation, ValueStream},
    options::{ShellOptions, run_posix_script},
    parser::{ParsedLine, parse_line},
};

pub struct Shell {
    config: UshConfig,
    options: ShellOptions,
    env: HashMap<String, String>,
    aliases: BTreeMap<String, String>,
    cwd: PathBuf,
    last_status: i32,
    paths: RuntimePaths,
}
