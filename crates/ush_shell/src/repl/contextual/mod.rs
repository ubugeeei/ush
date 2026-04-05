mod candidates;
mod catalog;
mod complete;
mod discover;
mod git;
mod nix;
mod options;
mod parse;
#[cfg(test)]
mod tests;
mod tool_catalog;
mod tools;
mod types;

pub(crate) use complete::complete;
pub(crate) use discover::discover_tasks;
pub(crate) use types::{ContextualCompletion, TaskEntry, TaskSource};
