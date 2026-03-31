use anyhow::Result;

use super::{
    super::{
        ast::{Call, Expr},
        env::{CodegenState, EnumRegistry, Env},
    },
    calls::rendered_call,
    compile_primitive_expr,
    functions::FunctionRegistry,
    runtime_support::{FailureMode, hoist_call, hoist_expr},
};
use crate::sourcemap::OutputBuffer;
use crate::traits::TraitImplRegistry;
use crate::types::OutputString as String;

pub(crate) fn compile_runtime_primitive_expr(
    expr: &Expr,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    inside_function: bool,
    out: &mut OutputBuffer,
) -> Result<String> {
    let mut runtime_env = env.clone();
    let prepared = hoist_expr(
        expr,
        &mut runtime_env,
        functions,
        impls,
        enums,
        state,
        FailureMode::Abort,
        inside_function,
        out,
    )?;
    compile_primitive_expr(&prepared, &runtime_env, functions, impls, enums)
}

pub(crate) fn rendered_call_runtime(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    inside_function: bool,
    out: &mut OutputBuffer,
) -> Result<String> {
    let mut runtime_env = env.clone();
    let prepared = hoist_call(
        call,
        &mut runtime_env,
        functions,
        impls,
        enums,
        state,
        FailureMode::Abort,
        inside_function,
        out,
    )?;
    rendered_call(&prepared, &runtime_env, functions, impls, enums)
}
