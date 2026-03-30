mod emit;
mod pattern;

use anyhow::{Result, bail};

use super::{
    ast::{Expr, Pattern, Type},
    codegen::infer,
    env::{Binding, CodegenState, EnumRegistry, Env, Storage, expect_adt},
};

use crate::types::{AstVec as Vec, OutputString as String};
use emit::{emit_copy, emit_variant};
use pattern::bind_pattern;

pub(crate) struct PatternPlan {
    pub condition: String,
    pub prelude: Vec<String>,
    pub env: Env,
}

pub(crate) fn materialize_expr(
    expr: &Expr,
    env: &Env,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    out: &mut String,
) -> Result<Binding> {
    let ty = infer(expr, env, enums)?;
    match ty.clone() {
        Type::Adt(enum_name) => {
            let prefix = state.temp_var("match");
            emit_value_to_target(
                &prefix,
                expr,
                &Type::Adt(enum_name.clone()),
                env,
                enums,
                state,
                out,
            )?;
            Ok(Binding {
                ty: Type::Adt(enum_name),
                storage: Storage::Adt(prefix),
            })
        }
        primitive => {
            let temp = state.temp_var("match");
            out.push_str(&format!(
                "{temp}={}\n",
                super::codegen::compile_primitive_expr(expr, env, enums)?
            ));
            Ok(Binding {
                ty: primitive,
                storage: Storage::Primitive(temp),
            })
        }
    }
}

pub(crate) fn emit_value_to_target(
    target: &str,
    expr: &Expr,
    expected: &Type,
    env: &Env,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    out: &mut String,
) -> Result<()> {
    match expected {
        Type::String | Type::Int | Type::Bool => {
            out.push_str(&format!(
                "{target}={}\n",
                super::codegen::compile_primitive_expr(expr, env, enums)?
            ));
        }
        Type::Adt(enum_name) => match expr {
            Expr::Variant(variant) => {
                emit_variant(target, variant, enum_name, env, enums, state, out)?
            }
            Expr::Var(name) => {
                let binding = env
                    .get(name)
                    .ok_or_else(|| anyhow::anyhow!("unknown variable: {name}"))?;
                if binding.ty != *expected {
                    bail!("type mismatch for {name}");
                }
                emit_copy(target, expect_adt(&binding.ty)?, binding, enums, state, out)?;
            }
            _ => bail!("expected ADT expression for {enum_name}"),
        },
        Type::Task(_) => bail!("task handles cannot be materialized as match values"),
    }
    Ok(())
}

pub(crate) fn compile_pattern(
    pattern: &Pattern,
    subject: &Binding,
    env: &Env,
    enums: &EnumRegistry,
) -> Result<PatternPlan> {
    let mut plan = PatternPlan {
        condition: String::from(":"),
        prelude: Vec::new(),
        env: env.clone(),
    };
    bind_pattern(pattern, subject, enums, &mut plan)?;
    Ok(plan)
}
