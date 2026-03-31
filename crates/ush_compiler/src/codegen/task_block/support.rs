use anyhow::{Result, anyhow, bail};

use crate::traits::TraitImplRegistry;
use crate::{
    ast::{Condition, Type},
    env::{EnumRegistry, Env},
    matching::compile_pattern,
};

use super::super::{
    control_support::homogeneous_type,
    functions::FunctionRegistry,
    infer,
    shared::binding_for_name,
};

pub(super) fn condition_env(
    condition: &Condition,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
) -> Result<Env> {
    match condition {
        Condition::Expr(expr) => {
            if infer(expr, env, functions, impls, enums)? != Type::Bool {
                bail!("if/while conditions must be boolean");
            }
            Ok(env.clone())
        }
        Condition::Let { pattern, expr } => Ok(
            compile_pattern(
                pattern,
                &binding_for_name("__ush_cond", infer(expr, env, functions, impls, enums)?),
                env,
                enums,
            )?
            .env,
        ),
        Condition::And(items) => items.iter().try_fold(env.clone(), |scope, item| {
            condition_env(item, &scope, functions, impls, enums)
        }),
        Condition::Or(items) => {
            if items.iter().any(has_pattern_binding) {
                bail!("pattern bindings are only supported in `&&` conditions");
            }
            Ok(env.clone())
        }
    }
}

pub(super) fn iterable_item_type(ty: &Type) -> Result<Type> {
    match ty {
        Type::List(item) => Ok((**item).clone()),
        Type::Tuple(items) => homogeneous_type(items)
            .ok_or_else(|| anyhow!("for-in over tuples requires homogeneous item types")),
        _ => bail!("for-in expects a list, tuple, or range"),
    }
}

pub(super) fn mismatch(scope: &str, expected: &Type, actual: &Type) -> Result<Type> {
    bail!(
        "{scope} type mismatch in async block: {} vs {}",
        expected.render(),
        actual.render()
    )
}

fn has_pattern_binding(condition: &Condition) -> bool {
    match condition {
        Condition::Let { .. } => true,
        Condition::And(items) | Condition::Or(items) => items.iter().any(has_pattern_binding),
        Condition::Expr(_) => false,
    }
}
