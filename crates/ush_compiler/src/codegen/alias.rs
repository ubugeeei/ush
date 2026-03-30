use anyhow::{Result, bail};

use super::{
    super::{
        ast::{Expr, Type},
        env::{CodegenState, EnumRegistry, Env},
    },
    compile_runtime_primitive_expr,
    functions::FunctionRegistry,
    infer,
};
use crate::{sourcemap::OutputBuffer, traits::TraitImplRegistry};

pub(crate) fn compile_alias(
    name: &str,
    value: &Expr,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    inside_function: bool,
    out: &mut OutputBuffer,
) -> Result<()> {
    if infer(value, env, functions, impls, enums)? != Type::String {
        bail!("alias values must evaluate to string");
    }
    out.push_str("alias ");
    out.push_str(name);
    out.push('=');
    let rendered = compile_runtime_primitive_expr(
        value,
        env,
        functions,
        impls,
        enums,
        state,
        inside_function,
        out,
    )?;
    out.push_str(&rendered);
    out.push('\n');
    Ok(())
}
