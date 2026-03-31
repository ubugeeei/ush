use anyhow::{Result, bail};

use crate::sourcemap::OutputBuffer;
use crate::traits::TraitImplRegistry;

use super::{
    super::{
        ast::{Expr, Type},
        env::{CodegenState, EnumRegistry, Env},
    },
    FunctionRegistry, compile_runtime_primitive_expr, infer,
    methods::compile_display_expr,
};

pub(crate) fn push_print(
    expr: &Expr,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    inside_function: bool,
    out: &mut OutputBuffer,
) -> Result<()> {
    let rendered = compile_display_expr(
        expr,
        env,
        functions,
        impls,
        enums,
        state,
        inside_function,
        out,
    )?;
    out.push_str("printf '%s\\n' ");
    out.push_str(&rendered);
    out.push('\n');
    Ok(())
}

pub(crate) fn compile_shell(
    expr: &Expr,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    inside_function: bool,
    propagate: bool,
    out: &mut OutputBuffer,
) -> Result<()> {
    if infer(expr, env, functions, impls, enums)? != Type::String {
        bail!("shell statements must evaluate to string");
    }
    if let Expr::String(value) = expr {
        out.push_str(value);
    } else {
        let rendered = compile_runtime_primitive_expr(
            expr,
            env,
            functions,
            impls,
            enums,
            state,
            inside_function,
            out,
        )?;
        out.push_str("eval ");
        out.push_str(&rendered);
    }
    if propagate {
        out.push_str(" || ");
        out.push_str(if inside_function {
            "return \"$?\""
        } else {
            "exit \"$?\""
        });
    }
    out.push('\n');
    Ok(())
}

pub(crate) fn compile_raise(
    expr: &Expr,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    inside_function: bool,
    out: &mut OutputBuffer,
) -> Result<()> {
    let Type::Adt(name) = infer(expr, env, functions, impls, enums)? else {
        bail!("raise expects an error ADT value");
    };
    out.push_str("printf '%s\\n' ");
    out.push_str(&super::super::util::shell_quote(&format!(
        "ush raise: {name}"
    )));
    out.push_str(" >&2\n");
    out.push_str(if inside_function {
        "return 1\n"
    } else {
        "exit 1\n"
    });
    Ok(())
}
