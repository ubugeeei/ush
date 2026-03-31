use crate::{
    ast::{FunctionDef, Type},
    sourcemap::OutputBuffer,
    types::HeapVec as Vec,
};

use super::{builtin, emit_fn, param};

pub(super) fn definitions() -> Vec<FunctionDef> {
    vec![
        builtin(
            "std::command::exists",
            vec![param("name", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::command::capture",
            vec![param("command", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::command::capture_stderr",
            vec![param("command", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::command::status",
            vec![param("command", Type::String)],
            Some(Type::Int),
        ),
        builtin(
            "std::command::run",
            vec![param("command", Type::String)],
            Some(Type::Unit),
        ),
    ]
}

pub(super) fn emit(out: &mut OutputBuffer) {
    emit_fn(
        out,
        "std::command::exists",
        "  if command -v \"$1\" >/dev/null 2>&1; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(out, "std::command::capture", "  /bin/sh -c \"$1\"\n");
    emit_fn(
        out,
        "std::command::capture_stderr",
        "  /bin/sh -c \"$1\" 2>&1 1>/dev/null\n",
    );
    emit_fn(
        out,
        "std::command::status",
        "  if /bin/sh -c \"$1\" >/dev/null 2>/dev/null; then\n    __ush_status=0\n  else\n    __ush_status=$?\n  fi\n  printf '%s' \"$__ush_status\"\n",
    );
    emit_fn(out, "std::command::run", "  /bin/sh -c \"$1\"\n");
}
