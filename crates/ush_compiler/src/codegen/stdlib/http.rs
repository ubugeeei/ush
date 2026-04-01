use crate::types::HeapVec as Vec;
use crate::{
    ast::{FunctionDef, Type},
    sourcemap::OutputBuffer,
};

use super::{builtin, emit_fn, param};

pub(super) fn definitions() -> Vec<FunctionDef> {
    vec![
        builtin(
            "std::http::get",
            vec![param("url", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::http::download",
            vec![param("url", Type::String), param("path", Type::String)],
            Some(Type::Unit),
        ),
    ]
}

pub(super) fn emit(out: &mut OutputBuffer) {
    emit_fn(
        out,
        "std::http::get",
        "  if command -v curl >/dev/null 2>&1; then\n    curl -fsSL \"$1\"\n    return \"$?\"\n  fi\n  if command -v wget >/dev/null 2>&1; then\n    wget -qO- \"$1\"\n    return \"$?\"\n  fi\n  printf '%s\\n' 'ush std::http::get: curl or wget is required' >&2\n  return 1\n",
    );
    emit_fn(
        out,
        "std::http::download",
        "  __ush_target=$(ush_fn_std__path__resolve \"$2\")\n  if command -v curl >/dev/null 2>&1; then\n    curl -fsSL \"$1\" -o \"$__ush_target\"\n    return \"$?\"\n  fi\n  if command -v wget >/dev/null 2>&1; then\n    wget -qO \"$__ush_target\" \"$1\"\n    return \"$?\"\n  fi\n  printf '%s\\n' 'ush std::http::download: curl or wget is required' >&2\n  return 1\n",
    );
}
