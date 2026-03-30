use anyhow::{Result, bail};

use super::{
    super::{
        ast::{EnumDef, Statement, Type, VariantFields},
        effects::FunctionErrorRegistry,
        env::{CodegenState, EnumRegistry, Env},
    },
    alias::compile_alias,
    blocks::{compile_let, compile_match},
    calls::{compile_call, compile_try_call},
    functions::{FunctionRegistry, compile_function},
    io::{compile_raise, compile_shell, push_print},
    shared::binding_for_name,
    tasks::{compile_await, compile_expr_statement, compile_return, compile_spawn},
};
use crate::traits::TraitImplRegistry;
use crate::types::{OutputString as String, Set as HashSet};

pub(crate) fn register_enum(def: &EnumDef, enums: &mut EnumRegistry) -> Result<()> {
    if enums.contains_key(&def.name) {
        bail!("duplicate enum: {}", def.name);
    }
    let mut variants = HashSet::with_hasher(Default::default());
    for variant in &def.variants {
        if !variants.insert(variant.name.clone()) {
            bail!("duplicate variant: {}::{}", def.name, variant.name);
        }
        if let VariantFields::Struct(fields) = &variant.fields {
            let mut names = HashSet::with_hasher(Default::default());
            for field in fields {
                if !names.insert(field.name.clone()) {
                    bail!(
                        "duplicate field: {}::{}::{}",
                        def.name,
                        variant.name,
                        field.name
                    );
                }
            }
        }
    }
    enums.insert(def.name.clone(), def.clone());
    Ok(())
}

pub(crate) fn compile_statement(
    statement: &Statement,
    env: &mut Env,
    globals: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
    state: &mut CodegenState,
    return_type: Option<&Type>,
    inside_function: bool,
    tail_position: bool,
    out: &mut String,
) -> Result<()> {
    match statement {
        Statement::Enum(_) => {}
        Statement::Trait(_) | Statement::Impl(_) => {}
        Statement::Function(def) => compile_function(
            def,
            globals,
            functions,
            impls,
            enums,
            function_errors,
            state,
            out,
        )?,
        Statement::Alias { name, value } => compile_alias(
            name,
            value,
            env,
            functions,
            impls,
            enums,
            state,
            inside_function,
            out,
        )?,
        Statement::Let { name, expr } => compile_let(
            name,
            expr,
            env,
            functions,
            impls,
            enums,
            state,
            inside_function,
            out,
        )?,
        Statement::Spawn { name, call } => {
            let binding = compile_spawn(
                call,
                env,
                functions,
                impls,
                enums,
                state,
                inside_function,
                out,
            )?;
            env.insert(name.clone(), binding);
        }
        Statement::Await { name, task } => {
            let binding = compile_await(task, env, out)?;
            env.insert(name.clone(), binding_for_name(name, binding.ty));
            if let super::super::env::Storage::Primitive(value) = binding.storage {
                out.push_str(name);
                out.push('=');
                out.push_str(&value);
                out.push('\n');
            }
        }
        Statement::Print(expr) => push_print(
            expr,
            env,
            functions,
            impls,
            enums,
            state,
            inside_function,
            out,
        )?,
        Statement::Shell(expr) => compile_shell(
            expr,
            env,
            functions,
            impls,
            enums,
            state,
            inside_function,
            false,
            out,
        )?,
        Statement::TryShell(expr) => compile_shell(
            expr,
            env,
            functions,
            impls,
            enums,
            state,
            inside_function,
            true,
            out,
        )?,
        Statement::Raise(expr) => {
            compile_raise(expr, env, functions, impls, enums, inside_function, out)?
        }
        Statement::Expr(expr) => {
            if tail_position {
                compile_return(expr, env, functions, impls, enums, state, return_type, out)?;
            } else {
                compile_expr_statement(
                    expr,
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
        Statement::Call(call) => compile_call(
            call,
            env,
            functions,
            impls,
            enums,
            state,
            inside_function,
            out,
        )?,
        Statement::TryCall(call) => compile_try_call(
            call,
            env,
            functions,
            impls,
            enums,
            state,
            inside_function,
            out,
        )?,
        Statement::Return(expr) => {
            compile_return(expr, env, functions, impls, enums, state, return_type, out)?
        }
        Statement::Match {
            expr,
            arms,
            returns_value,
        } => compile_match(
            expr,
            arms,
            *returns_value,
            env,
            globals,
            functions,
            impls,
            enums,
            function_errors,
            state,
            return_type,
            inside_function,
            out,
        )?,
    }
    Ok(())
}
