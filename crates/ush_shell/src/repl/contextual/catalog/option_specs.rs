use crate::repl::contextual::options::{OptionSpec, option_spec};

pub(crate) const MAKE_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["-C", "--directory"], 1, true, false),
    option_spec(&["-f", "--file", "--makefile"], 1, true, true),
    option_spec(
        &["-I", "--include-dir", "-o", "--old-file", "--assume-old"],
        1,
        true,
        true,
    ),
    option_spec(
        &["-W", "--what-if", "--new-file", "--assume-new"],
        1,
        true,
        true,
    ),
    option_spec(
        &["-j", "--jobs", "-l", "--load-average", "--max-load"],
        1,
        false,
        true,
    ),
    option_spec(&["--debug", "-N", "--NeXT-option"], 1, false, true),
];

pub(crate) const JUST_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["--chooser"], 1, false, false),
    option_spec(&["--command"], 1, false, false),
    option_spec(&["--dump-format"], 1, false, false),
    option_spec(&["--justfile", "-f"], 1, true, true),
    option_spec(&["--set"], 2, false, false),
    option_spec(&["--shell"], 1, false, false),
    option_spec(&["--shell-arg"], 1, false, false),
    option_spec(&["--show", "-s"], 1, false, false),
    option_spec(&["--tempdir"], 1, true, false),
    option_spec(&["--usage"], 1, false, false),
    option_spec(&["--working-directory", "-d"], 1, true, true),
];

pub(crate) const MISE_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["-C", "--cd"], 1, true, false),
    option_spec(&["-E", "--env"], 1, false, false),
    option_spec(&["-j", "--jobs"], 1, false, false),
    option_spec(&["--output"], 1, false, false),
];

pub(crate) const NPM_OPTION_SPECS: &[OptionSpec] = &[
    option_spec(&["--workspace", "-w"], 1, false, false),
    option_spec(&["--prefix"], 1, true, false),
    option_spec(&["--userconfig"], 1, true, false),
];
