use alloc::boxed::Box;

use anyhow::{Result, anyhow, bail};

use super::{
    super::{
        ast::{Call, Expr, Type},
        env::{Binding, CodegenState, EnumRegistry, Env, Storage},
    },
    calls::{call_return_type, ensure_value_type, rendered_call},
    compile_primitive_expr,
    functions::FunctionRegistry,
    infer,
};
use crate::traits::TraitImplRegistry;
use crate::types::OutputString as String;

pub(crate) fn compile_spawn(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    out: &mut String,
) -> Result<Binding> {
    let return_type = call_return_type(&call.name, functions)?
        .ok_or_else(|| anyhow!("async bindings require a return type: {}", call.name))?;
    ensure_value_type(&return_type)?;

    let prefix = state.temp_var("task");
    let rendered = rendered_call(call, env, functions, impls, enums)?;
    out.push_str("__ush_task_seq=$((__ush_task_seq + 1))\n");
    out.push_str(&format!(
        "{prefix}__result=\"${{TMPDIR:-/tmp}}/{prefix}.$$.$__ush_task_seq\"\n"
    ));
    out.push_str(&format!(": > \"${{{prefix}__result}}\"\n"));
    out.push_str(&format!(
        "__ush_task_files=\"${{__ush_task_files}}${{__ush_task_files:+ }}${{{prefix}__result}}\"\n"
    ));
    out.push_str(&format!(
        "( __ush_return_path=\"${{{prefix}__result}}\"; {rendered} ) &\n"
    ));
    out.push_str(&format!("{prefix}__pid=\"$!\"\n"));
    out.push_str(&format!("{prefix}__awaited='0'\n"));
    out.push_str("__ush_jobs=\"${__ush_jobs}${__ush_jobs:+ }$!\"\n");

    Ok(Binding {
        ty: Type::Task(Box::new(return_type)),
        storage: Storage::Task(prefix),
    })
}

pub(crate) fn compile_await(task: &str, env: &Env, out: &mut String) -> Result<Binding> {
    let binding = env
        .get(task)
        .ok_or_else(|| anyhow!("unknown task: {task}"))?;
    let (Type::Task(inner), Storage::Task(prefix)) = (&binding.ty, &binding.storage) else {
        bail!("await expects a task handle: {task}");
    };

    out.push_str(&format!("if [ \"${{{prefix}__awaited}}\" = '0' ]; then\n"));
    out.push_str(&format!("  wait \"${{{prefix}__pid}}\"\n"));
    out.push_str(&format!("  {prefix}__awaited='1'\n"));
    out.push_str("fi\n");

    Ok(Binding {
        ty: (*inner.clone()),
        storage: Storage::Primitive(format!("\"$(cat \"${{{prefix}__result}}\")\"").into()),
    })
}

pub(crate) fn compile_return(
    expr: &Expr,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    return_type: Option<&Type>,
    out: &mut String,
) -> Result<()> {
    let Some(declared) = return_type else {
        bail!("return is only valid inside functions");
    };
    let actual = infer(expr, env, functions, impls, enums)?;
    if actual != *declared {
        bail!("return type mismatch: expected {declared:?}, found {actual:?}");
    }
    ensure_value_type(declared)?;

    out.push_str("if [ -n \"${__ush_return_path:-}\" ]; then\n");
    out.push_str("  printf '%s' ");
    out.push_str(&compile_primitive_expr(expr, env, functions, impls, enums)?);
    out.push_str(" > \"$__ush_return_path\"\n");
    out.push_str("elif [ \"${__ush_capture_return:-0}\" = '1' ]; then\n");
    out.push_str("  printf '%s' ");
    out.push_str(&compile_primitive_expr(expr, env, functions, impls, enums)?);
    out.push('\n');
    out.push_str("fi\n");
    out.push_str("return 0\n");
    Ok(())
}
