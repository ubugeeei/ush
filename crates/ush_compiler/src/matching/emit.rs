use anyhow::{Result, bail};

use super::super::{
    ast::{ExprFields, Type, VariantExpr, VariantFields},
    codegen::FunctionRegistry,
    env::{Binding, CodegenState, EnumRegistry, Env, Storage, lookup_variant},
};
use crate::traits::TraitImplRegistry;
use crate::types::{AstString as NameString, OutputString as String};

use super::emit_value_to_target;

pub(super) fn emit_variant(
    target: &str,
    variant: &VariantExpr,
    expected_enum: &str,
    env: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    inside_function: bool,
    out: &mut String,
) -> Result<()> {
    if variant.enum_name != expected_enum {
        bail!("expected {expected_enum}, found {}", variant.enum_name);
    }
    let def = lookup_variant(enums, &variant.enum_name, &variant.variant_name)?;
    out.push_str(&format!(
        "{target}__tag='{}::{}'\n",
        variant.enum_name, variant.variant_name
    ));
    match (&def.fields, &variant.fields) {
        (VariantFields::Unit, ExprFields::Unit) => {}
        (VariantFields::Tuple(types), ExprFields::Tuple(values)) if types.len() == values.len() => {
            for (index, (ty, expr)) in types.iter().zip(values).enumerate() {
                emit_value_to_target(
                    &format!("{target}__{index}"),
                    expr,
                    ty,
                    env,
                    functions,
                    impls,
                    enums,
                    state,
                    inside_function,
                    out,
                )?;
            }
        }
        (VariantFields::Struct(fields), ExprFields::Struct(values)) => {
            for value in values {
                if !fields.iter().any(|field| field.name == value.name) {
                    bail!("unknown field {}", value.name);
                }
            }
            for field in fields {
                let expr = values
                    .iter()
                    .find(|item| item.name == field.name)
                    .ok_or_else(|| anyhow::anyhow!("missing field {}", field.name))?;
                emit_value_to_target(
                    &format!("{target}__{}", field.name),
                    &expr.expr,
                    &field.ty,
                    env,
                    functions,
                    impls,
                    enums,
                    state,
                    inside_function,
                    out,
                )?;
            }
        }
        _ => bail!(
            "variant field shape mismatch for {}::{}",
            variant.enum_name,
            variant.variant_name
        ),
    }
    Ok(())
}

pub(super) fn emit_copy(
    target: &str,
    enum_name: &str,
    binding: &Binding,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    out: &mut String,
) -> Result<()> {
    let Storage::Adt(source) = &binding.storage else {
        bail!("cannot copy primitive into ADT target");
    };
    out.push_str(&format!("{target}__tag=\"${{{source}__tag}}\"\n"));
    let enum_def = enums
        .get(enum_name)
        .ok_or_else(|| anyhow::anyhow!("unknown enum: {enum_name}"))?;
    out.push_str(&format!("case \"${{{source}__tag}}\" in\n"));
    for variant in &enum_def.variants {
        out.push_str(&format!("  '{}::{}')\n", enum_name, variant.name));
        match &variant.fields {
            VariantFields::Unit => {}
            VariantFields::Tuple(types) => {
                for (index, ty) in types.iter().enumerate() {
                    copy_field(
                        &format!("{source}__{index}"),
                        &format!("{target}__{index}"),
                        ty,
                        enums,
                        state,
                        out,
                    )?;
                }
            }
            VariantFields::Struct(fields) => {
                for field in fields {
                    copy_field(
                        &format!("{source}__{}", field.name),
                        &format!("{target}__{}", field.name),
                        &field.ty,
                        enums,
                        state,
                        out,
                    )?;
                }
            }
        }
        out.push_str("    ;;\n");
    }
    out.push_str("esac\n");
    Ok(())
}

fn copy_field(
    source: &str,
    target: &str,
    ty: &Type,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    out: &mut String,
) -> Result<()> {
    match ty {
        Type::String | Type::Int | Type::Bool | Type::Unit => {
            out.push_str(&format!("{target}=\"${{{source}}}\"\n"))
        }
        Type::Adt(enum_name) => emit_copy(
            target,
            enum_name,
            &Binding {
                ty: ty.clone(),
                storage: Storage::Adt(NameString::from(source)),
            },
            enums,
            state,
            out,
        )?,
        Type::Task(_) => bail!("task handles cannot be stored inside ADT values"),
    }
    Ok(())
}
