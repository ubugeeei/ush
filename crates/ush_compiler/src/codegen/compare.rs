use anyhow::{Result, bail};

use super::{
    super::{
        ast::{CompareOp, Expr, Type},
        env::{EnumRegistry, Env},
    },
    compile_primitive_expr,
    functions::FunctionRegistry,
    infer,
};
use crate::{traits::TraitImplRegistry, traits::ensure_trait, types::OutputString as String};

pub(crate) fn infer_compare(
    lhs: &Expr,
    rhs: &Expr,
    op: CompareOp,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
) -> Result<Type> {
    let left = infer(lhs, env, functions, impls, enums)?;
    let right = infer(rhs, env, functions, impls, enums)?;
    if left != right {
        bail!("comparison type mismatch: left {left:?}, right {right:?}");
    }
    let trait_name = match op {
        CompareOp::Eq | CompareOp::Ne => "Eq",
        _ => "Ord",
    };
    ensure_trait(&left, trait_name, impls)?;
    if matches!(left, Type::Task(_)) {
        bail!("task handles cannot be compared");
    }
    Ok(Type::Bool)
}

pub(crate) fn compile_compare_expr(
    lhs: &Expr,
    rhs: &Expr,
    op: CompareOp,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
) -> Result<String> {
    infer_compare(lhs, rhs, op, env, functions, impls, enums)?;
    let ty = infer(lhs, env, functions, impls, enums)?;
    let lhs = compile_primitive_expr(lhs, env, functions, impls, enums)?;
    let rhs = compile_primitive_expr(rhs, env, functions, impls, enums)?;
    Ok(match ty {
        Type::Int => bool_capture(int_condition(op, lhs.as_str(), rhs.as_str())),
        Type::String => bool_capture(string_condition(op, lhs.as_str(), rhs.as_str())),
        Type::Bool => bool_capture(bool_condition(op, lhs.as_str(), rhs.as_str())),
        Type::Unit => bool_capture(unit_condition(op)),
        Type::Adt(name) => bail!("comparison for ADT `{name}` is not implemented yet"),
        Type::Tuple(_) | Type::List(_) => bail!("structured values cannot be compared yet"),
        Type::Task(_) => bail!("task handles cannot be compared"),
    })
}

fn bool_capture(condition: String) -> String {
    format!("\"$(if {condition}; then printf '%s' true; else printf '%s' false; fi)\"")
}

fn int_condition(op: CompareOp, lhs: &str, rhs: &str) -> String {
    let op = match op {
        CompareOp::Eq => "-eq",
        CompareOp::Ne => "-ne",
        CompareOp::Lt => "-lt",
        CompareOp::Le => "-le",
        CompareOp::Gt => "-gt",
        CompareOp::Ge => "-ge",
    };
    format!("[ {lhs} {op} {rhs} ]")
}

fn bool_condition(op: CompareOp, lhs: &str, rhs: &str) -> String {
    match op {
        CompareOp::Eq => format!("[ {lhs} = {rhs} ]"),
        CompareOp::Ne => format!("[ {lhs} != {rhs} ]"),
        _ => int_condition(op, &bool_rank(lhs), &bool_rank(rhs)),
    }
}

fn bool_rank(value: &str) -> String {
    format!("$(if [ {value} = 'true' ]; then printf '%s' 1; else printf '%s' 0; fi)")
}

fn string_condition(op: CompareOp, lhs: &str, rhs: &str) -> String {
    match op {
        CompareOp::Eq => format!("[ {lhs} = {rhs} ]"),
        CompareOp::Ne => format!("[ {lhs} != {rhs} ]"),
        CompareOp::Lt => string_lt(lhs, rhs),
        CompareOp::Le => format!("[ {lhs} = {rhs} ] || {}", string_lt(lhs, rhs)),
        CompareOp::Gt => string_lt(rhs, lhs),
        CompareOp::Ge => format!("[ {lhs} = {rhs} ] || {}", string_lt(rhs, lhs)),
    }
}

fn string_lt(lhs: &str, rhs: &str) -> String {
    format!(
        "[ {lhs} != {rhs} ] && [ \"$(printf '%s\\n%s\\n' {lhs} {rhs} | LC_ALL=C sort | head -n 1)\" = {lhs} ]"
    )
}

fn unit_condition(op: CompareOp) -> String {
    match op {
        CompareOp::Eq | CompareOp::Le | CompareOp::Ge => ":".into(),
        CompareOp::Ne | CompareOp::Lt | CompareOp::Gt => "false".into(),
    }
}
