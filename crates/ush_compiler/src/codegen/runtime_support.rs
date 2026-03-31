use alloc::boxed::Box;

use anyhow::Result;

use super::{
    super::{
        ast::{Call, Expr},
        env::{Binding, CodegenState, EnumRegistry, Env, Storage},
    },
    FunctionRegistry,
    calls::{call_expr_type, ensure_value_type, rendered_call},
    runtime_member,
};
use crate::sourcemap::OutputBuffer;
use crate::traits::TraitImplRegistry;

#[derive(Clone, Copy)]
pub(super) enum FailureMode {
    Abort,
    Propagate { inside_function: bool },
}

pub(super) fn hoist_expr(
    expr: &Expr,
    env: &mut Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    mode: FailureMode,
    inside_function: bool,
    out: &mut OutputBuffer,
) -> Result<Expr> {
    Ok(match expr {
        Expr::Add(parts) => Expr::Add(
            parts
                .iter()
                .map(|part| {
                    hoist_expr(
                        part,
                        env,
                        functions,
                        impls,
                        enums,
                        state,
                        mode,
                        inside_function,
                        out,
                    )
                })
                .collect::<Result<_>>()?,
        ),
        Expr::Compare { lhs, op, rhs } => Expr::Compare {
            lhs: Box::new(hoist_expr(
                lhs,
                env,
                functions,
                impls,
                enums,
                state,
                mode,
                inside_function,
                out,
            )?),
            op: *op,
            rhs: Box::new(hoist_expr(
                rhs,
                env,
                functions,
                impls,
                enums,
                state,
                mode,
                inside_function,
                out,
            )?),
        },
        Expr::Try(inner) => hoist_expr(
            inner,
            env,
            functions,
            impls,
            enums,
            state,
            FailureMode::Propagate { inside_function },
            inside_function,
            out,
        )?,
        Expr::Field { base, name } => runtime_member::hoist_field_access(
            base,
            name,
            env,
            functions,
            impls,
            enums,
            state,
            inside_function,
            out,
        )?,
        Expr::MethodCall(call) => runtime_member::hoist_method_capture(
            call,
            env,
            functions,
            impls,
            enums,
            state,
            matches!(mode, FailureMode::Propagate { .. }),
            inside_function,
            out,
        )?,
        Expr::Call(call) if call.name == "format" => runtime_member::hoist_format_capture(
            call,
            env,
            functions,
            impls,
            enums,
            state,
            matches!(mode, FailureMode::Propagate { .. }),
            inside_function,
            out,
        )?,
        Expr::Call(call) => hoist_call_capture(
            call,
            env,
            functions,
            impls,
            enums,
            state,
            mode,
            inside_function,
            out,
        )?,
        other => other.clone(),
    })
}

pub(super) fn hoist_call(
    call: &Call,
    env: &mut Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    mode: FailureMode,
    inside_function: bool,
    out: &mut OutputBuffer,
) -> Result<Call> {
    let mut prepared = call.clone();
    for arg in &mut prepared.args {
        arg.expr = hoist_expr(
            &arg.expr,
            env,
            functions,
            impls,
            enums,
            state,
            mode,
            inside_function,
            out,
        )?;
    }
    Ok(prepared)
}

fn hoist_call_capture(
    call: &Call,
    env: &mut Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    mode: FailureMode,
    inside_function: bool,
    out: &mut OutputBuffer,
) -> Result<Expr> {
    let prepared = hoist_call(
        call,
        env,
        functions,
        impls,
        enums,
        state,
        mode,
        inside_function,
        out,
    )?;
    let ty = call_expr_type(&prepared, functions)?;
    ensure_value_type(&ty)?;
    let rendered = rendered_call(&prepared, env, functions, impls, enums)?;

    let temp = state.temp_var("value");
    out.push_str(&temp);
    out.push_str("=\"$(__ush_capture_return='1' ");
    out.push_str(&rendered);
    out.push_str(")\"");
    if let FailureMode::Propagate { inside_function } = mode {
        out.push_str(" || ");
        out.push_str(if inside_function {
            "return \"$?\""
        } else {
            "exit \"$?\""
        });
    }
    out.push('\n');
    env.insert(
        temp.clone(),
        Binding {
            ty,
            storage: Storage::Primitive(temp.clone()),
        },
    );
    Ok(Expr::Var(temp))
}
