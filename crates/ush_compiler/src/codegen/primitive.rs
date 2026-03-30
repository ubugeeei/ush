use alloc::string::ToString;

use anyhow::{Result, anyhow, bail};

use super::super::{
    ast::{Expr, ExprFields, Type, VariantExpr, VariantFields},
    env::{EnumRegistry, Env, Storage, lookup_variant},
    util::shell_quote,
};
use crate::types::{AstVec as Vec, OutputString as String};

pub(crate) fn infer(expr: &Expr, env: &Env, enums: &EnumRegistry) -> Result<Type> {
    match expr {
        Expr::String(_) => Ok(Type::String),
        Expr::Int(_) => Ok(Type::Int),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::Var(name) => env
            .get(name)
            .map(|binding| binding.ty.clone())
            .ok_or_else(|| anyhow!("unknown variable: {name}")),
        Expr::Add(parts) => infer_add(parts, env, enums),
        Expr::Variant(variant) => infer_variant(variant, env, enums),
    }
}

pub(crate) fn compile_primitive_expr(
    expr: &Expr,
    env: &Env,
    enums: &EnumRegistry,
) -> Result<String> {
    match infer(expr, env, enums)? {
        Type::String => compile_string_expr(expr, env, enums),
        Type::Int => compile_int_expr(expr, env, enums),
        Type::Bool => compile_bool_expr(expr, env),
        Type::Adt(name) => bail!("cannot use {name} as a primitive shell value"),
        Type::Task(_) => bail!("task handles cannot be used as primitive values"),
    }
}

fn infer_add(parts: &[Expr], env: &Env, enums: &EnumRegistry) -> Result<Type> {
    let mut saw_string = false;
    for part in parts {
        match infer(part, env, enums)? {
            Type::String => saw_string = true,
            Type::Int => {}
            Type::Bool => bail!("booleans cannot participate in `+` expressions"),
            Type::Adt(name) => bail!("ADT `{name}` cannot participate in `+` expressions"),
            Type::Task(_) => bail!("task handles cannot participate in `+` expressions"),
        }
    }
    Ok(if saw_string { Type::String } else { Type::Int })
}

fn infer_variant(variant: &VariantExpr, env: &Env, enums: &EnumRegistry) -> Result<Type> {
    let def = lookup_variant(enums, &variant.enum_name, &variant.variant_name)?;
    match (&def.fields, &variant.fields) {
        (VariantFields::Unit, ExprFields::Unit) => {}
        (VariantFields::Tuple(types), ExprFields::Tuple(values)) if types.len() == values.len() => {
            for (ty, expr) in types.iter().zip(values) {
                if infer(expr, env, enums)? != *ty {
                    bail!(
                        "tuple constructor type mismatch for {}::{}",
                        variant.enum_name,
                        variant.variant_name
                    );
                }
            }
        }
        (VariantFields::Struct(fields), ExprFields::Struct(values)) => {
            for value in values {
                if !fields.iter().any(|field| field.name == value.name) {
                    bail!("unknown field {}", value.name);
                }
            }
            for field in fields {
                let value = values
                    .iter()
                    .find(|value| value.name == field.name)
                    .ok_or_else(|| anyhow!("missing field {}", field.name))?;
                if infer(&value.expr, env, enums)? != field.ty {
                    bail!(
                        "struct constructor type mismatch for {}::{}",
                        variant.enum_name,
                        variant.variant_name
                    );
                }
            }
        }
        _ => bail!(
            "variant field shape mismatch for {}::{}",
            variant.enum_name,
            variant.variant_name
        ),
    }
    Ok(Type::Adt(variant.enum_name.clone()))
}

fn compile_string_expr(expr: &Expr, env: &Env, enums: &EnumRegistry) -> Result<String> {
    match expr {
        Expr::Add(parts) => Ok(format!(
            "\"$(printf '%s' {})\"",
            parts
                .iter()
                .map(|part| compile_string_fragment(part, env, enums))
                .collect::<Result<Vec<_>>>()?
                .join(" ")
        )),
        _ => compile_string_fragment(expr, env, enums),
    }
}

fn compile_string_fragment(expr: &Expr, env: &Env, enums: &EnumRegistry) -> Result<String> {
    match expr {
        Expr::String(value) => Ok(shell_quote(value)),
        Expr::Int(value) => Ok(shell_quote(&value.to_string())),
        Expr::Bool(value) => Ok(shell_quote(if *value { "true" } else { "false" })),
        Expr::Var(name) => match env.get(name) {
            Some(binding) => primitive_var(binding, name).map(|var| format!("\"${{{var}}}\"")),
            None => bail!("unknown variable: {name}"),
        },
        Expr::Add(_) => compile_string_expr(expr, env, enums),
        Expr::Variant(_) => bail!("ADT values cannot be stringified implicitly"),
    }
}

fn compile_int_expr(expr: &Expr, env: &Env, enums: &EnumRegistry) -> Result<String> {
    match expr {
        Expr::Int(value) => Ok(value.to_string()),
        Expr::Var(name) => Ok(format!("$(({}))", primitive_int_var(env, name)?)),
        Expr::Add(parts) => Ok(format!(
            "$(({}))",
            int_terms(parts, env, enums)?.join(" + ")
        )),
        _ => bail!("unsupported integer expression"),
    }
}

fn compile_bool_expr(expr: &Expr, env: &Env) -> Result<String> {
    match expr {
        Expr::Bool(value) => Ok(shell_quote(if *value { "true" } else { "false" })),
        Expr::Var(name) => Ok(format!("\"${{{}}}\"", primitive_bool_var(env, name)?)),
        _ => bail!("unsupported boolean expression"),
    }
}

fn int_terms(parts: &[Expr], env: &Env, enums: &EnumRegistry) -> Result<Vec<String>> {
    let mut compiled = Vec::with_capacity(parts.len());
    for part in parts {
        match part {
            Expr::Int(value) => compiled.push(value.to_string()),
            Expr::Var(name) => compiled.push(primitive_int_var(env, name)?.to_string()),
            Expr::Add(_) if infer(part, env, enums)? == Type::Int => {
                compiled.push(format!("({})", compile_int_expr(part, env, enums)?));
            }
            _ => bail!("only integer addition is supported"),
        }
    }
    Ok(compiled)
}

fn primitive_var<'a>(binding: &'a super::super::env::Binding, name: &str) -> Result<&'a str> {
    match &binding.storage {
        Storage::Primitive(var) => Ok(var.as_str()),
        Storage::Adt(_) => bail!("`{name}` is not a primitive value"),
        Storage::Task(_) => bail!("`{name}` is a task handle"),
    }
}

fn primitive_int_var<'a>(env: &'a Env, name: &str) -> Result<&'a str> {
    match env.get(name) {
        Some(binding) if binding.ty == Type::Int => primitive_var(binding, name),
        Some(_) => bail!("`{name}` is not an integer"),
        None => bail!("unknown variable: {name}"),
    }
}

fn primitive_bool_var<'a>(env: &'a Env, name: &str) -> Result<&'a str> {
    match env.get(name) {
        Some(binding) if binding.ty == Type::Bool => primitive_var(binding, name),
        Some(_) => bail!("`{name}` is not a boolean"),
        None => bail!("unknown variable: {name}"),
    }
}
