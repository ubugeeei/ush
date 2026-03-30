#![cfg_attr(not(feature = "std"), no_std)]

mod ast;
mod codegen;
mod env;
mod matching;
mod parse;
mod types;
mod util;

#[macro_use]
extern crate alloc;

use anyhow::Result;
use types::OutputString;

#[cfg(feature = "std")]
use anyhow::Context;
#[cfg(feature = "std")]
use std::{fs, path::Path};

#[derive(Debug, Clone, Default)]
pub struct UshCompiler;

impl UshCompiler {
    #[cfg(feature = "std")]
    pub fn compile_file(&self, path: &Path) -> Result<OutputString> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        self.compile_source(&source)
            .with_context(|| format!("failed to compile {}", path.display()))
    }

    pub fn compile_source(&self, source: &str) -> Result<OutputString> {
        let program = parse::parse_program(source)?;
        codegen::compile_program(&program)
    }
}

#[cfg(test)]
mod tests {
    use super::UshCompiler;

    #[test]
    fn compiles_basic_program() {
        let compiler = UshCompiler;
        let output = compiler
            .compile_source(
                r#"
                let greeting = "hello"
                print greeting + " world"
                shell "printf '%s\n' hi"
            "#,
            )
            .expect("compile");

        assert!(output.contains("greeting='hello'"));
        assert!(output.contains("printf '%s\\n'"));
        assert!(output.contains("printf '%s\\n' hi"));
    }

    #[test]
    fn concat_is_shell_quoted() {
        let compiler = UshCompiler;
        let output = compiler
            .compile_source(
                r#"
                let greeting = "hello"
                print greeting + " world"
            "#,
            )
            .expect("compile");

        assert!(output.contains("\"$(printf '%s' \"${greeting}\" ' world')\""));
    }

    #[test]
    fn enum_is_lowered_to_tagged_shell_bindings() {
        let compiler = UshCompiler;
        let output = compiler
            .compile_source(
                r#"
                enum Option {
                  None,
                  Some(String),
                }
                let value = Option::Some("hello")
            "#,
            )
            .expect("compile");

        assert!(output.contains("value__tag='Option::Some'"));
        assert!(output.contains("value__0='hello'"));
    }

    #[test]
    fn async_await_lowers_to_task_handle_and_result_read() {
        let compiler = UshCompiler;
        let output = compiler
            .compile_source(
                r#"
                fn worker(message: String) -> String {
                  return message
                }
                let task = async worker("ok")
                let result = await task
                print result
            "#,
            )
            .expect("compile");

        assert!(output.starts_with("#!/bin/sh\nset -eu\n"));
        assert!(output.contains("ush_fn_worker()"));
        assert!(output.contains("__ush_task_seq='0'"));
        assert!(
            output.contains(
                "__ush_task_0__result=\"${TMPDIR:-/tmp}/__ush_task_0.$$.$__ush_task_seq\""
            )
        );
        assert!(
            output.contains(
                "( __ush_return_path=\"${__ush_task_0__result}\"; ush_fn_worker 'ok' ) &"
            )
        );
        assert!(output.contains("result=\"$(cat \"${__ush_task_0__result}\")\""));
        assert!(output.contains("rm -f \"$__ush_task_file\""));
        assert!(!output.contains("mktemp"));
        assert!(output.contains("wait \"$__ush_job\""));
    }
}
