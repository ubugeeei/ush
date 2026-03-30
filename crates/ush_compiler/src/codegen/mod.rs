mod alias;
mod bin;
mod blocks;
mod calls;
mod compare;
mod docs;
mod functions;
mod io;
mod primitive;
mod render;
mod runtime_expr;
mod shared;
mod statement;
mod tasks;

use anyhow::Result;

use super::{
    ast::{Statement, StatementKind},
    effects,
    env::{CodegenState, EnumRegistry, Env},
};
use crate::ScriptDocs;
use crate::sourcemap::{CompiledScript, OutputBuffer};
use crate::traits::{TraitImplRegistry, TraitRegistry, register_trait, register_trait_impl};

pub(crate) use functions::FunctionRegistry;
pub(crate) use primitive::{compile_primitive_expr, infer};
pub(crate) use runtime_expr::{compile_runtime_primitive_expr, rendered_call_runtime};

pub(crate) fn compile_program(
    program: &[Statement],
    docs: &ScriptDocs,
    script_name: Option<&str>,
) -> Result<CompiledScript> {
    let mut env = Env::default();
    let mut functions = functions::FunctionRegistry::default();
    let mut enums = EnumRegistry::default();
    let mut traits = TraitRegistry::default();
    let mut trait_impls = TraitImplRegistry::default();
    let mut state = CodegenState::default();
    let mut out = OutputBuffer::from_text(
        "#!/bin/sh\nset -eu\n\n__ush_jobs=''\n__ush_task_seq='0'\n__ush_task_files=''\n\n",
    );

    for statement in program {
        match &statement.kind {
            StatementKind::Enum(def) => statement::register_enum(def, &mut enums)?,
            StatementKind::Trait(def) => register_trait(def, &mut traits)?,
            StatementKind::Impl(item) => register_trait_impl(item, &traits, &mut trait_impls)?,
            StatementKind::Function(def) => functions::register_function(def, &mut functions)?,
            _ => {}
        }
    }

    let bin_entry = script_name
        .filter(|name| *name == "bin.ush")
        .and_then(|_| functions.get("bin"));
    let extra_completion = bin_entry
        .map(bin::completion_candidates)
        .unwrap_or_default();
    docs::push_doc_support(
        &mut out,
        docs,
        script_name.unwrap_or("script"),
        &extra_completion,
    );

    let globals = functions::analyze_globals(program, &functions, &trait_impls, &enums)?;
    let function_errors =
        effects::analyze_function_errors(program, &globals, &functions, &trait_impls, &enums)?;
    for statement in program {
        if matches!(statement.kind, StatementKind::Enum(_)) {
            continue;
        }
        statement::compile_statement(
            statement,
            &mut env,
            &globals,
            &functions,
            &trait_impls,
            &enums,
            &function_errors,
            &mut state,
            None,
            false,
            false,
            &mut out,
        )?;
    }
    if let Some(def) = bin_entry {
        bin::push_bin_entry(&mut out, def, &globals, &functions, &trait_impls, &enums)?;
    }
    functions::push_wait_footer(&mut out);
    Ok(out.into_compiled())
}
