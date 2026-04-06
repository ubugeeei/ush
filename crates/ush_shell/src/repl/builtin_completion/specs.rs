mod basic_specs;
mod common;
mod extra_specs;
mod types;

pub(super) use common::{COMMAND_OPTIONS, SIGNAL_CHOICES};
pub(super) use types::{ArgKind, BuiltinOptionSpec, BuiltinSpec};

pub(super) fn command_summary(name: &str) -> Option<&'static str> {
    resolve_builtin_spec(name).map(|spec| spec.summary)
}

pub(super) fn resolve_builtin_spec(name: &str) -> Option<&'static BuiltinSpec> {
    basic_specs::resolve(name).or_else(|| extra_specs::resolve(name))
}
