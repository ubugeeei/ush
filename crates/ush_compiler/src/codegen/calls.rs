use anyhow::{Result, anyhow, bail};

use super::{
    super::{
        ast::{Call, Expr, FunctionDef, Type},
        env::{CodegenState, EnumRegistry, Env},
    },
    compile_primitive_expr,
    functions::FunctionRegistry,
    infer, rendered_call_runtime,
};
use crate::traits::TraitImplRegistry;
use crate::types::{AstVec as Vec, OutputString as String};

pub(crate) fn compile_call(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    inside_function: bool,
    out: &mut String,
) -> Result<()> {
    let rendered = rendered_call_runtime(
        call,
        env,
        functions,
        impls,
        enums,
        state,
        inside_function,
        out,
    )?;
    out.push_str("( __ush_capture_return='0' __ush_return_path=''; ");
    out.push_str(&rendered);
    out.push_str(" )");
    if call.asynchronous {
        out.push_str(" &\n__ush_jobs=\"${__ush_jobs}${__ush_jobs:+ }$!\"\n");
    } else {
        out.push('\n');
    }
    Ok(())
}

pub(crate) fn compile_try_call(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    inside_function: bool,
    out: &mut String,
) -> Result<()> {
    let rendered = rendered_call_runtime(
        call,
        env,
        functions,
        impls,
        enums,
        state,
        inside_function,
        out,
    )?;
    out.push_str("( __ush_capture_return='0' __ush_return_path=''; ");
    out.push_str(&rendered);
    out.push_str(" )");
    out.push_str(" || ");
    out.push_str(if inside_function {
        "return \"$?\""
    } else {
        "exit \"$?\""
    });
    out.push('\n');
    Ok(())
}

pub(crate) fn rendered_call(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
) -> Result<String> {
    let def = function_for_call(&call.name, functions)?;
    let args = rendered_args(call, env, functions, impls, enums, def)?;
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

pub(crate) fn call_expr_type(call: &Call, functions: &FunctionRegistry) -> Result<Type> {
    call_return_type(&call.name, functions)?
        .ok_or_else(|| anyhow!("function `{}` does not return a value", call.name))
}

pub(crate) fn capture_call(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    expected: &Type,
) -> Result<String> {
    let actual = call_expr_type(call, functions)?;
    ensure_value_type(&actual)?;
    if &actual != expected {
        bail!(
            "function `{}` returns {:?}, but {:?} was required",
            call.name,
            actual,
            expected
        );
    }
    Ok(format!(
        "$(__ush_capture_return='1' {})",
        rendered_call(call, env, functions, impls, enums)?
    ))
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
        Type::String | Type::Int | Type::Bool | Type::Unit => Ok(()),
        Type::Adt(name) => bail!("ADT values are not supported here yet: {name}"),
        Type::Task(_) => bail!("nested task types are not supported"),
    }
}

fn rendered_args(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    def: &FunctionDef,
) -> Result<Vec<String>> {
    ensure_signature(def)?;
    def.params
        .iter()
        .zip(resolve_call_args(call, def)?)
        .map(|(param, arg)| {
            let actual = infer(arg, env, functions, impls, enums)?;
            if actual != param.ty {
                bail!(
                    "type mismatch for `{}`: expected {:?}, found {:?}",
                    param.name,
                    param.ty,
                    actual
                );
            }
            compile_primitive_expr(arg, env, functions, impls, enums)
        })
        .collect()
}

fn resolve_call_args<'a>(call: &'a Call, def: &'a FunctionDef) -> Result<Vec<&'a Expr>> {
    let mut resolved = vec![None; def.params.len()];
    let mut next = 0usize;

    for arg in &call.args {
        let index = match &arg.label {
            Some(label) => def
                .params
                .iter()
                .position(|param| param.name == *label)
                .ok_or_else(|| anyhow!("unknown argument label `{label}` for `{}`", call.name))?,
            None => {
                while next < resolved.len() && resolved[next].is_some() {
                    next += 1;
                }
                if next >= resolved.len() {
                    bail!(
                        "function `{}` expects at most {} arguments",
                        call.name,
                        def.params.len()
                    );
                }
                next
            }
        };
        if resolved[index].is_some() {
            bail!(
                "duplicate argument for `{}`: {}",
                call.name,
                def.params[index].name
            );
        }
        resolved[index] = Some(&arg.expr);
        if arg.label.is_none() {
            next += 1;
        }
    }

    def.params
        .iter()
        .enumerate()
        .map(|(index, param)| {
            resolved[index]
                .or(param.default.as_ref())
                .ok_or_else(|| anyhow!("missing argument for `{}`: {}", call.name, param.name))
        })
        .collect()
}

fn function_for_call<'a>(name: &str, functions: &'a FunctionRegistry) -> Result<&'a FunctionDef> {
    functions
        .get(name)
        .ok_or_else(|| anyhow!("unknown function: {name}"))
}
