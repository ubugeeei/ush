use alloc::boxed::Box;

use anyhow::Result;

use super::infer::infer_async_block_type;
use crate::sourcemap::OutputBuffer;
use crate::traits::TraitImplRegistry;
use crate::{
    ast::{Statement, Type},
    effects::FunctionErrorRegistry,
    env::{Binding, CodegenState, EnumRegistry, Env, Storage},
};

use super::super::{
    calls::ensure_value_type,
    functions::FunctionRegistry,
    shared::{push_line, push_output, shell_function_name},
    statement::compile_statement,
};

pub(crate) fn compile_async_block(
    body: &[Statement],
    env: &Env,
    globals: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
    state: &mut CodegenState,
    out: &mut OutputBuffer,
) -> Result<Binding> {
    let return_type = infer_async_block_type(body, env, functions, impls, enums)?;
    ensure_value_type(&return_type)?;

    let helper = shell_function_name(&state.temp_var("task_block"));
    out.push_str(&helper);
    out.push_str("() {\n");
    let mut body_env = env.clone();
    let body_out = compile_body(
        body,
        &mut body_env,
        globals,
        functions,
        impls,
        enums,
        function_errors,
        state,
        &return_type,
    )?;
    if body_out.is_empty() {
        push_line(out, ":", 2);
    } else {
        push_output(out, &body_out, 2);
    }
    out.push_str("}\n");

    let prefix = state.temp_var("task");
    out.push_str("__ush_task_seq=$((__ush_task_seq + 1))\n");
    out.push_str(&format!(
        "{prefix}__result=\"${{TMPDIR:-/tmp}}/{prefix}.$$.$__ush_task_seq\"\n"
    ));
    out.push_str(&format!(": > \"${{{prefix}__result}}\"\n"));
    out.push_str(&format!(
        "__ush_task_files=\"${{__ush_task_files}}${{__ush_task_files:+ }}${{{prefix}__result}}\"\n"
    ));
    out.push_str(&format!(
        "( __ush_return_path=\"${{{prefix}__result}}\"; {helper} ) &\n"
    ));
    out.push_str(&format!("{prefix}__pid=\"$!\"\n"));
    out.push_str(&format!("{prefix}__awaited='0'\n"));
    out.push_str("__ush_jobs=\"${__ush_jobs}${__ush_jobs:+ }$!\"\n");
    Ok(Binding {
        ty: Type::Task(Box::new(return_type)),
        storage: Storage::Task(prefix),
    })
}

fn compile_body(
    body: &[Statement],
    env: &mut Env,
    globals: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
    state: &mut CodegenState,
    return_type: &Type,
) -> Result<OutputBuffer> {
    let mut out = OutputBuffer::default();
    for (index, statement) in body.iter().enumerate() {
        compile_statement(
            statement,
            env,
            globals,
            functions,
            impls,
            enums,
            function_errors,
            state,
            Some(return_type),
            true,
            index + 1 == body.len(),
            &mut out,
        )?;
    }
    Ok(out)
}
