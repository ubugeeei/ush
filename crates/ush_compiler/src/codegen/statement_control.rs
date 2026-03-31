use anyhow::Result;

use super::{
    super::{
        ast::StatementKind,
        effects::FunctionErrorRegistry,
        env::{CodegenState, Env},
    },
    control::{ControlContext, compile_for, compile_if, compile_loop, compile_while},
    functions::FunctionRegistry,
};
use crate::sourcemap::OutputBuffer;
use crate::traits::TraitImplRegistry;
use crate::{ast::Type, env::EnumRegistry};

pub(crate) fn compile_control_statement(
    kind: &StatementKind,
    env: &mut Env,
    globals: &Env,
    functions: &FunctionRegistry,
    impls: &TraitImplRegistry,
    enums: &EnumRegistry,
    function_errors: &FunctionErrorRegistry,
    state: &mut CodegenState,
    return_type: Option<&Type>,
    inside_function: bool,
    out: &mut OutputBuffer,
) -> Option<Result<()>> {
    let mut ctx = ControlContext {
        globals,
        functions,
        impls,
        enums,
        function_errors,
        state,
        return_type,
        inside_function,
    };
    Some(match kind {
        StatementKind::If {
            branch,
            returns_value,
        } => compile_if(branch, *returns_value, env, &mut ctx, out),
        StatementKind::While { condition, body } => {
            compile_while(condition, body, env, &mut ctx, out)
        }
        StatementKind::For {
            name,
            iterable,
            body,
        } => compile_for(name, iterable, env, body, &mut ctx, out),
        StatementKind::Loop { body } => compile_loop(body, env, &mut ctx, out),
        StatementKind::Break => {
            out.push_str("break\n");
            Ok(())
        }
        StatementKind::Continue => {
            out.push_str("continue\n");
            Ok(())
        }
        _ => return None,
    })
}
