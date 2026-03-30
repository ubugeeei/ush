use alloc::boxed::Box;

use anyhow::{Result, anyhow, bail};

use super::{
    super::{
        ast::{EnumDef, Expr, Statement, Type, VariantFields},
        env::{CodegenState, EnumRegistry, Env},
        matching::{compile_pattern, emit_value_to_target, materialize_expr},
    },
    calls::compile_call,
    compile_primitive_expr,
    functions::{FunctionRegistry, compile_function},
    infer,
    shared::{binding_for_name, push_block, push_line},
    tasks::{compile_await, compile_return, compile_spawn},
};
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
    enums: &EnumRegistry,
    state: &mut CodegenState,
    return_type: Option<&Type>,
    out: &mut String,
) -> Result<()> {
    match statement {
        Statement::Enum(_) => {}
        Statement::Function(def) => compile_function(def, globals, functions, enums, state, out)?,
        Statement::Let { name, expr } => compile_let(name, expr, env, enums, state, out)?,
        Statement::Spawn { name, call } => {
            let binding = compile_spawn(call, env, functions, enums, state, out)?;
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
        Statement::Print(expr) => push_print(expr, env, enums, out)?,
        Statement::Shell(expr) => compile_shell(expr, env, enums, out)?,
        Statement::Call(call) => compile_call(call, env, functions, enums, out)?,
        Statement::Return(expr) => compile_return(expr, env, enums, return_type, out)?,
        Statement::Match { expr, arms } => compile_match(
            expr,
            arms,
            env,
            globals,
            functions,
            enums,
            state,
            return_type,
            out,
        )?,
    }
    Ok(())
}

fn compile_let(
    name: &str,
    expr: &Expr,
    env: &mut Env,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    out: &mut String,
) -> Result<()> {
    let ty = infer(expr, env, enums)?;
    match &ty {
        Type::String | Type::Int | Type::Bool => {
            out.push_str(name);
            out.push('=');
            out.push_str(&compile_primitive_expr(expr, env, enums)?);
            out.push('\n');
        }
        Type::Adt(_) => emit_value_to_target(name, expr, &ty, env, enums, state, out)?,
        Type::Task(_) => bail!("task expressions must be bound via `let name = async ...`"),
    }
    env.insert(name.into(), binding_for_name(name, ty));
    Ok(())
}

fn push_print(expr: &Expr, env: &Env, enums: &EnumRegistry, out: &mut String) -> Result<()> {
    out.push_str("printf '%s\\n' ");
    out.push_str(&compile_primitive_expr(expr, env, enums)?);
    out.push('\n');
    Ok(())
}

fn compile_shell(expr: &Expr, env: &Env, enums: &EnumRegistry, out: &mut String) -> Result<()> {
    if infer(expr, env, enums)? != Type::String {
        bail!("shell statements must evaluate to string");
    }
    if let Expr::String(value) = expr {
        out.push_str(value);
    } else {
        out.push_str("eval ");
        out.push_str(&compile_primitive_expr(expr, env, enums)?);
    }
    out.push('\n');
    Ok(())
}

fn compile_match(
    expr: &Expr,
    arms: &[(super::super::ast::Pattern, Box<Statement>)],
    env: &Env,
    globals: &Env,
    functions: &FunctionRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    return_type: Option<&Type>,
    out: &mut String,
) -> Result<()> {
    if arms.is_empty() {
        bail!("match must have at least one arm");
    }
    let subject = materialize_expr(expr, env, enums, state, out)?;
    for (index, (pattern, arm)) in arms.iter().enumerate() {
        let plan = compile_pattern(pattern, &subject, env, enums)?;
        out.push_str(if index == 0 { "if " } else { "elif " });
        out.push_str(&plan.condition);
        out.push_str("; then\n");
        for line in &plan.prelude {
            push_line(out, line, 2);
        }
        let mut arm_env = plan.env;
        let body = compile_one(
            arm,
            &mut arm_env,
            globals,
            functions,
            enums,
            state,
            return_type,
        )?;
        push_block(out, &body, 2);
    }
    out.push_str("fi\n");
    Ok(())
}

fn compile_one(
    statement: &Statement,
    env: &mut Env,
    globals: &Env,
    functions: &FunctionRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    return_type: Option<&Type>,
) -> Result<String> {
    let mut buffer = String::new();
    compile_statement(
        statement,
        env,
        globals,
        functions,
        enums,
        state,
        return_type,
        &mut buffer,
    )?;
    if buffer.is_empty() {
        return Err(anyhow!("empty statement body"));
    }
    Ok(buffer)
}
