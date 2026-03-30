use anyhow::{Result, anyhow, bail};

use super::{
    super::{
        ast::{Call, FunctionDef, Type},
        env::{EnumRegistry, Env},
    },
    compile_primitive_expr,
    functions::FunctionRegistry,
    infer,
};
use crate::types::{AstVec as Vec, OutputString as String};

pub(crate) fn compile_call(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    enums: &EnumRegistry,
    out: &mut String,
) -> Result<()> {
    let rendered = rendered_call(call, env, functions, enums)?;
    out.push_str(&rendered);
    if call.asynchronous {
        out.push_str(" &\n__ush_jobs=\"${__ush_jobs}${__ush_jobs:+ }$!\"\n");
    } else {
        out.push('\n');
    }
    Ok(())
}

pub(crate) fn rendered_call(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    enums: &EnumRegistry,
) -> Result<String> {
    let def = function_for_call(&call.name, functions)?;
    let args = rendered_args(call, env, enums, def)?;
    let target = format!("ush_fn_{}", call.name);
    Ok(if args.is_empty() {
        target
    } else {
        format!("{target} {}", args.join(" "))
    })
}

pub(crate) fn call_return_type(name: &str, functions: &FunctionRegistry) -> Result<Option<Type>> {
    Ok(function_for_call(name, functions)?.return_type.clone())
}

pub(crate) fn ensure_signature(def: &FunctionDef) -> Result<()> {
    for param in &def.params {
        ensure_value_type(&param.ty)?;
    }
    if let Some(ty) = &def.return_type {
        ensure_value_type(ty)?;
    }
    Ok(())
}

pub(crate) fn ensure_value_type(ty: &Type) -> Result<()> {
    match ty {
        Type::String | Type::Int | Type::Bool => Ok(()),
        Type::Adt(name) => bail!("ADT values are not supported here yet: {name}"),
        Type::Task(_) => bail!("nested task types are not supported"),
    }
}

fn rendered_args(
    call: &Call,
    env: &Env,
    enums: &EnumRegistry,
    def: &FunctionDef,
) -> Result<Vec<String>> {
    ensure_signature(def)?;
    if call.args.len() != def.params.len() {
        bail!(
            "function `{}` expects {} arguments, found {}",
            call.name,
            def.params.len(),
            call.args.len()
        );
    }
    call.args
        .iter()
        .zip(&def.params)
        .map(|(arg, param)| {
            let actual = infer(arg, env, enums)?;
            if actual != param.ty {
                bail!(
                    "type mismatch for `{}`: expected {:?}, found {:?}",
                    param.name,
                    param.ty,
                    actual
                );
            }
            compile_primitive_expr(arg, env, enums)
        })
        .collect()
}

fn function_for_call<'a>(name: &str, functions: &'a FunctionRegistry) -> Result<&'a FunctionDef> {
    functions
        .get(name)
        .ok_or_else(|| anyhow!("unknown function: {name}"))
}
