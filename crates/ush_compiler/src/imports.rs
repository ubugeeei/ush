use anyhow::{Result, bail};

use crate::{
    ast::{Call, Expr, FunctionDef, Statement, StatementKind},
    types::{Map as HashMap, Set as HashSet},
};

pub(crate) fn resolve_program(mut program: Vec<Statement>) -> Result<Vec<Statement>> {
    let locals = local_functions(&program);
    let imports = collect_imports(&program, &locals)?;
    for statement in &mut program {
        resolve_statement(statement, &imports);
    }
    Ok(program)
}

fn local_functions(program: &[Statement]) -> HashSet<crate::types::AstString> {
    let mut names = HashSet::with_hasher(Default::default());
    for statement in program {
        if let StatementKind::Function(def) = &statement.kind {
            names.insert(def.name.clone());
        }
    }
    names
}

fn collect_imports(
    program: &[Statement],
    locals: &HashSet<crate::types::AstString>,
) -> Result<HashMap<crate::types::AstString, crate::types::AstString>> {
    let mut imports = HashMap::with_hasher(Default::default());
    for statement in program {
        let StatementKind::Use(items) = &statement.kind else {
            continue;
        };
        for item in items {
            if locals.contains(&item.alias) {
                bail!("import alias conflicts with local function: {}", item.alias);
            }
            if imports
                .insert(item.alias.clone(), item.path.clone())
                .is_some()
            {
                bail!("duplicate import alias: {}", item.alias);
            }
        }
    }
    Ok(imports)
}

fn resolve_statement(
    statement: &mut Statement,
    imports: &HashMap<crate::types::AstString, crate::types::AstString>,
) {
    match &mut statement.kind {
        StatementKind::Use(_)
        | StatementKind::Enum(_)
        | StatementKind::Trait(_)
        | StatementKind::Impl(_)
        | StatementKind::Await { .. } => {}
        StatementKind::Function(def) => resolve_function(def, imports),
        StatementKind::Alias { value, .. }
        | StatementKind::Let { expr: value, .. }
        | StatementKind::Print(value)
        | StatementKind::Shell(value)
        | StatementKind::TryShell(value)
        | StatementKind::Raise(value)
        | StatementKind::Expr(value)
        | StatementKind::Return(value) => resolve_expr(value, imports),
        StatementKind::Spawn { call, .. }
        | StatementKind::Call(call)
        | StatementKind::TryCall(call) => resolve_call(call, imports),
        StatementKind::Match { expr, arms, .. } => {
            resolve_expr(expr, imports);
            for (_, arm) in arms {
                resolve_statement(arm, imports);
            }
        }
    }
}

fn resolve_function(
    def: &mut FunctionDef,
    imports: &HashMap<crate::types::AstString, crate::types::AstString>,
) {
    for param in &mut def.params {
        if let Some(default) = &mut param.default {
            resolve_expr(default, imports);
        }
    }
    for statement in &mut def.body {
        resolve_statement(statement, imports);
    }
}

fn resolve_expr(
    expr: &mut Expr,
    imports: &HashMap<crate::types::AstString, crate::types::AstString>,
) {
    match expr {
        Expr::Add(parts) => {
            for part in parts {
                resolve_expr(part, imports);
            }
        }
        Expr::Compare { lhs, rhs, .. } => {
            resolve_expr(lhs, imports);
            resolve_expr(rhs, imports);
        }
        Expr::Try(inner) => resolve_expr(inner, imports),
        Expr::Call(call) => resolve_call(call, imports),
        Expr::Variant(variant) => match &mut variant.fields {
            crate::ast::ExprFields::Unit => {}
            crate::ast::ExprFields::Tuple(items) => {
                for item in items {
                    resolve_expr(item, imports);
                }
            }
            crate::ast::ExprFields::Struct(items) => {
                for item in items {
                    resolve_expr(&mut item.expr, imports);
                }
            }
        },
        Expr::String(_) | Expr::Int(_) | Expr::Bool(_) | Expr::Unit | Expr::Var(_) => {}
    }
}

fn resolve_call(
    call: &mut Call,
    imports: &HashMap<crate::types::AstString, crate::types::AstString>,
) {
    if !call.name.contains("::")
        && let Some(path) = imports.get(&call.name)
    {
        call.name = path.clone();
    }
    for arg in &mut call.args {
        resolve_expr(&mut arg.expr, imports);
    }
}
