mod calls;
mod functions;
mod primitive;
mod shared;
mod statement;
mod tasks;

use anyhow::Result;

use super::{
    ast::Statement,
    env::{CodegenState, EnumRegistry, Env},
};
use crate::types::OutputString as String;

pub(crate) use primitive::{compile_primitive_expr, infer};

pub(crate) fn compile_program(program: &[Statement]) -> Result<String> {
    let mut env = Env::default();
    let mut functions = functions::FunctionRegistry::default();
    let mut enums = EnumRegistry::default();
    let mut state = CodegenState::default();
    let mut out = String::from(
        "#!/bin/sh\nset -eu\n\n__ush_jobs=''\n__ush_task_seq='0'\n__ush_task_files=''\n\n",
    );

    for statement in program {
        match statement {
            Statement::Enum(def) => statement::register_enum(def, &mut enums)?,
            Statement::Function(def) => functions::register_function(def, &mut functions)?,
            _ => {}
        }
    }

    let globals = functions::analyze_globals(program, &enums)?;
    for statement in program {
        if matches!(statement, Statement::Enum(_)) {
            continue;
        }
        statement::compile_statement(
            statement, &mut env, &globals, &functions, &enums, &mut state, None, &mut out,
        )?;
    }
    functions::push_wait_footer(&mut out);
    Ok(out)
}
