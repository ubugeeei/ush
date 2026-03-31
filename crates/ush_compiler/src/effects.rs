mod support;

use alloc::boxed::Box;

use anyhow::{Result, anyhow, bail};

use crate::traits::TraitImplRegistry;
use crate::types::{AstString as String, Map as HashMap};
use crate::{
    ast::{FunctionDef, Statement, StatementKind, Type},
    codegen::{FunctionRegistry, infer},
    env::{Binding, EnumRegistry, Env, Storage},
    errors::{ErrorSet, ErrorType},
    matching::compile_pattern,
};
use support::{
    binding_for_type, call_arg_errors, call_errors, exposed_errors, expr_errors, raised_error,
    validate_function_errors,
};

pub(crate) type FunctionErrorRegistry = HashMap<String, ErrorSet>;
type TaskErrorRegistry = HashMap<String, ErrorSet>;

pub(crate) fn analyze_function_errors(
    program: &[Statement],
    globals: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
) -> Result<FunctionErrorRegistry> {
    let mut registry = FunctionErrorRegistry::default();
    for statement in program {
        if let StatementKind::Function(def) = &statement.kind {
            registry.insert(def.name.clone(), exposed_errors(def, &ErrorSet::default()));
        }
    }

    for _ in 0..=registry.len() {
        let mut changed = false;
        for statement in program {
            let StatementKind::Function(def) = &statement.kind else {
                continue;
            };
            let inferred = analyze_function(def, globals, functions, impls, enums, &registry)?;
            let errors = exposed_errors(def, &inferred);
            if registry.get(&def.name) != Some(&errors) {
                registry.insert(def.name.clone(), errors);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    for statement in program {
        let StatementKind::Function(def) = &statement.kind else {
            continue;
        };
        let inferred = analyze_function(def, globals, functions, impls, enums, &registry)?;
        validate_function_errors(def, &inferred, enums)?;
    }

    Ok(registry)
}

fn analyze_function(
    def: &FunctionDef,
    globals: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
) -> Result<ErrorSet> {
    let mut env = globals.clone();
    let mut tasks = TaskErrorRegistry::default();
    for param in &def.params {
        env.insert(
            param.name.clone(),
            binding_for_type(&param.name, param.ty.clone()),
        );
    }
    block_errors(
        &def.body,
        &mut env,
        &mut tasks,
        functions,
        impls,
        enums,
        function_errors,
    )
}

fn block_errors(
    statements: &[Statement],
    env: &mut Env,
    tasks: &mut TaskErrorRegistry,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
) -> Result<ErrorSet> {
    let mut errors = ErrorSet::default();
    for statement in statements {
        let current = statement_errors(
            statement,
            env,
            tasks,
            functions,
            impls,
            enums,
            function_errors,
        )?;
        errors.extend(&current);
    }
    Ok(errors)
}

fn statement_errors(
    statement: &Statement,
    env: &mut Env,
    tasks: &mut TaskErrorRegistry,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
) -> Result<ErrorSet> {
    match &statement.kind {
        StatementKind::Use(_)
        | StatementKind::Enum(_)
        | StatementKind::Trait(_)
        | StatementKind::Impl(_)
        | StatementKind::Function(_) => Ok(ErrorSet::default()),
        StatementKind::Alias { value, .. } => {
            expr_errors(value, env, functions, impls, enums, function_errors)
        }
        StatementKind::Let { name, expr } => {
            let errors = expr_errors(expr, env, functions, impls, enums, function_errors)?;
            let ty = infer(expr, env, functions, impls, enums)?;
            env.insert(name.clone(), binding_for_type(name, ty));
            Ok(errors)
        }
        StatementKind::Spawn { name, call } => {
            let errors = call_arg_errors(call, env, functions, impls, enums, function_errors)?;
            let def = functions
                .get(&call.name)
                .ok_or_else(|| anyhow!("unknown function: {}", call.name))?;
            let ty = def
                .return_type
                .clone()
                .ok_or_else(|| anyhow!("async bindings require a return type: {}", call.name))?;
            env.insert(
                name.clone(),
                binding_for_type(name, Type::Task(Box::new(ty))),
            );
            tasks.insert(
                name.clone(),
                function_errors.get(&call.name).cloned().unwrap_or_default(),
            );
            Ok(errors)
        }
        StatementKind::Await { name, task } => {
            let binding = env
                .get(task)
                .ok_or_else(|| anyhow!("unknown task: {task}"))?;
            let Type::Task(inner) = &binding.ty else {
                bail!("await expects a task handle: {task}");
            };
            let errors = tasks.get(task).cloned().unwrap_or_default();
            env.insert(name.clone(), binding_for_type(name, *inner.clone()));
            Ok(errors)
        }
        StatementKind::Print(expr) | StatementKind::Expr(expr) | StatementKind::Return(expr) => {
            expr_errors(expr, env, functions, impls, enums, function_errors)
        }
        StatementKind::Shell(expr) | StatementKind::TryShell(expr) => {
            let mut errors = expr_errors(expr, env, functions, impls, enums, function_errors)?;
            errors.insert(ErrorType::Unknown);
            Ok(errors)
        }
        StatementKind::Call(call) | StatementKind::TryCall(call) => {
            call_errors(call, env, functions, impls, enums, function_errors)
        }
        StatementKind::Raise(expr) => {
            let mut errors = expr_errors(expr, env, functions, impls, enums, function_errors)?;
            errors.insert(raised_error(expr, env, functions, impls, enums)?);
            Ok(errors)
        }
        StatementKind::Match { expr, arms, .. } => {
            let mut errors = expr_errors(expr, env, functions, impls, enums, function_errors)?;
            let subject = match infer(expr, env, functions, impls, enums)? {
                Type::Adt(name) => Binding {
                    ty: Type::Adt(name),
                    storage: Storage::Adt("__ush_effect_match".into()),
                },
                ty => Binding {
                    ty,
                    storage: Storage::Primitive("__ush_effect_match".into()),
                },
            };
            for (pattern, arm) in arms {
                let plan = compile_pattern(pattern, &subject, env, enums)?;
                let mut arm_tasks = tasks.clone();
                let mut arm_env = plan.env;
                let arm_errors = statement_errors(
                    arm,
                    &mut arm_env,
                    &mut arm_tasks,
                    functions,
                    impls,
                    enums,
                    function_errors,
                )?;
                errors.extend(&arm_errors);
            }
            Ok(errors)
        }
    }
}
