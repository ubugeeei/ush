use anyhow::{Result, bail};

use super::{
    super::{
        ast::{FunctionDef, Statement},
        env::{CodegenState, EnumRegistry, Env},
    },
    calls::ensure_signature,
    shared::{binding_for_name, push_block, push_line},
    statement::compile_statement,
};
use crate::types::{AstString as NameString, Map as HashMap, OutputString as String};

pub(crate) type FunctionRegistry = HashMap<NameString, FunctionDef>;

pub(crate) fn register_function(def: &FunctionDef, functions: &mut FunctionRegistry) -> Result<()> {
    if functions.contains_key(&def.name) {
        bail!("duplicate function: {}", def.name);
    }
    for (index, param) in def.params.iter().enumerate() {
        if def.params[..index]
            .iter()
            .any(|other| other.name == param.name)
        {
            bail!("duplicate parameter: {}::{}", def.name, param.name);
        }
    }
    functions.insert(def.name.clone(), def.clone());
    Ok(())
}

pub(crate) fn analyze_globals(program: &[Statement], enums: &EnumRegistry) -> Result<Env> {
    let mut env = Env::default();
    for statement in program {
        if let Statement::Let { name, expr } = statement {
            let ty = super::infer(expr, &env, enums)?;
            env.insert(name.clone(), binding_for_name(name, ty));
        }
    }
    Ok(env)
}

pub(crate) fn compile_function(
    def: &FunctionDef,
    globals: &Env,
    functions: &FunctionRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    out: &mut String,
) -> Result<()> {
    ensure_signature(def)?;
    out.push_str("ush_fn_");
    out.push_str(&def.name);
    out.push_str("() {\n");

    let mut env = globals.clone();
    for (index, param) in def.params.iter().enumerate() {
        let storage = param_storage(&def.name, index);
        push_line(out, &format!("{storage}=\"${}\"", index + 1), 2);
        env.insert(
            param.name.clone(),
            binding_for_name(&storage, param.ty.clone()),
        );
    }

    let body = compile_many(
        &def.body,
        &mut env,
        globals,
        functions,
        enums,
        state,
        def.return_type.as_ref(),
    )?;
    if body.is_empty() {
        push_line(out, ":", 2);
    } else {
        push_block(out, &body, 2);
    }
    out.push_str("}\n\n");
    Ok(())
}

pub(crate) fn push_wait_footer(out: &mut String) {
    out.push_str("\nif [ -n \"$__ush_jobs\" ]; then\n");
    out.push_str("  for __ush_job in $__ush_jobs; do\n");
    out.push_str("    wait \"$__ush_job\" 2>/dev/null || true\n");
    out.push_str("  done\n");
    out.push_str("fi\n");
    out.push_str("if [ -n \"$__ush_task_files\" ]; then\n");
    out.push_str("  for __ush_task_file in $__ush_task_files; do\n");
    out.push_str("    rm -f \"$__ush_task_file\"\n");
    out.push_str("  done\n");
    out.push_str("fi\n");
}

fn compile_many(
    statements: &[Statement],
    env: &mut Env,
    globals: &Env,
    functions: &FunctionRegistry,
    enums: &EnumRegistry,
    state: &mut CodegenState,
    return_type: Option<&super::super::ast::Type>,
) -> Result<String> {
    let mut buffer = String::new();
    for statement in statements {
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
    }
    Ok(buffer)
}

fn param_storage(function_name: &str, index: usize) -> String {
    format!("__ush_fn_{}_arg_{}", function_name, index)
}
