use anyhow::{Result, bail};

use crate::traits::TraitImplRegistry;
use crate::{
    ast::{Call, Expr, ExprFields, FunctionDef, Type},
    codegen::{FunctionRegistry, infer},
    env::{Binding, EnumRegistry, Env, Storage},
    errors::{ErrorSet, ErrorType},
};

use super::FunctionErrorRegistry;

pub(super) fn expr_errors(
    expr: &Expr,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
) -> Result<ErrorSet> {
    let mut errors = ErrorSet::default();
    match expr {
        Expr::String(_) | Expr::Int(_) | Expr::Bool(_) | Expr::Unit | Expr::Var(_) | Expr::AsyncBlock(_) => {}
        Expr::Tuple(items) | Expr::List(items) => {
            for item in items {
                errors.extend(&expr_errors(
                    item,
                    env,
                    functions,
                    impls,
                    enums,
                    function_errors,
                )?);
            }
        }
        Expr::Range { start, end } => {
            errors.extend(&expr_errors(
                start,
                env,
                functions,
                impls,
                enums,
                function_errors,
            )?);
            errors.extend(&expr_errors(
                end,
                env,
                functions,
                impls,
                enums,
                function_errors,
            )?);
        }
        Expr::Add(parts) => {
            for part in parts {
                errors.extend(&expr_errors(
                    part,
                    env,
                    functions,
                    impls,
                    enums,
                    function_errors,
                )?);
            }
        }
        Expr::Compare { lhs, rhs, .. } => {
            errors.extend(&expr_errors(
                lhs,
                env,
                functions,
                impls,
                enums,
                function_errors,
            )?);
            errors.extend(&expr_errors(
                rhs,
                env,
                functions,
                impls,
                enums,
                function_errors,
            )?);
        }
        Expr::Try(inner) => {
            errors.extend(&expr_errors(
                inner,
                env,
                functions,
                impls,
                enums,
                function_errors,
            )?);
        }
        Expr::Call(call) => {
            errors.extend(&call_errors(
                call,
                env,
                functions,
                impls,
                enums,
                function_errors,
            )?);
        }
        Expr::Variant(variant) => match &variant.fields {
            ExprFields::Unit => {}
            ExprFields::Tuple(values) => {
                for value in values {
                    errors.extend(&expr_errors(
                        value,
                        env,
                        functions,
                        impls,
                        enums,
                        function_errors,
                    )?);
                }
            }
            ExprFields::Struct(values) => {
                for value in values {
                    errors.extend(&expr_errors(
                        &value.expr,
                        env,
                        functions,
                        impls,
                        enums,
                        function_errors,
                    )?);
                }
            }
        },
    }
    Ok(errors)
}

pub(super) fn call_errors(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
) -> Result<ErrorSet> {
    let mut errors = call_arg_errors(call, env, functions, impls, enums, function_errors)?;
    if let Some(raised) = function_errors.get(&call.name) {
        errors.extend(raised);
    }
    Ok(errors)
}

pub(super) fn call_arg_errors(
    call: &Call,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
) -> Result<ErrorSet> {
    let mut errors = ErrorSet::default();
    for arg in &call.args {
        errors.extend(&expr_errors(
            &arg.expr,
            env,
            functions,
            impls,
            enums,
            function_errors,
        )?);
    }
    Ok(errors)
}

pub(super) fn raised_error(
    expr: &Expr,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
) -> Result<ErrorType> {
    match infer(expr, env, functions, impls, enums)? {
        Type::Adt(name) => Ok(ErrorType::Known(name)),
        other => bail!("raise expects an error ADT value, found {other:?}"),
    }
}

pub(super) fn binding_for_type(name: &str, ty: Type) -> Binding {
    let storage = match &ty {
        Type::Adt(_) => Storage::Adt(name.into()),
        Type::Tuple(_) | Type::List(_) => Storage::Aggregate(name.into()),
        Type::String | Type::Int | Type::Bool | Type::Unit => Storage::Primitive(name.into()),
        Type::Task(_) => Storage::Task(name.into()),
    };
    Binding { ty, storage }
}

pub(super) fn exposed_errors(def: &FunctionDef, inferred: &ErrorSet) -> ErrorSet {
    def.declared_errors
        .clone()
        .unwrap_or_else(|| inferred.clone())
}

pub(super) fn validate_function_errors(
    def: &FunctionDef,
    inferred: &ErrorSet,
    enums: &EnumRegistry,
) -> Result<()> {
    match &def.declared_errors {
        Some(declared) => {
            validate_declared_error_types(def, declared, enums)?;
            if !inferred.is_subset_of(declared) {
                bail!(
                    "function `{}` declares `{}` but can raise `{}`",
                    def.name,
                    declared.render_union(def.return_type.as_ref()),
                    inferred.render()
                );
            }
            Ok(())
        }
        None if inferred.is_empty() => Ok(()),
        None => bail!(
            "function `{}` can raise `{}` but its signature does not declare an error set; write `-> {}`",
            def.name,
            inferred.render(),
            inferred.render_union(def.return_type.as_ref())
        ),
    }
}

fn validate_declared_error_types(
    def: &FunctionDef,
    declared: &ErrorSet,
    enums: &EnumRegistry,
) -> Result<()> {
    for error in declared.iter() {
        match error {
            ErrorType::Known(name) => {
                if !enums.contains_key(name) {
                    bail!(
                        "function `{}` declares unknown error type `{}`",
                        def.name,
                        name
                    );
                }
            }
            ErrorType::Unknown => {}
        }
    }
    Ok(())
}
