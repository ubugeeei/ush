use crate::types::HeapVec as Vec;
use crate::{
    ast::{FunctionDef, Type},
    sourcemap::OutputBuffer,
};

use super::{builtin, emit_fn, param};

pub(super) fn definitions() -> Vec<FunctionDef> {
    vec![
        builtin(
            "std::env::get",
            vec![param("name", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::env::get_or",
            vec![param("name", Type::String), param("default", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::env::set",
            vec![param("name", Type::String), param("value", Type::String)],
            Some(Type::Unit),
        ),
        builtin(
            "std::env::unset",
            vec![param("name", Type::String)],
            Some(Type::Unit),
        ),
    ]
}

pub(super) fn emit(out: &mut OutputBuffer) {
    out.push_str(
        "__ush_std_env_validate() {\n  case \"$1\" in\n    ''|[0-9]*|*[!A-Za-z0-9_]*)\n      printf '%s\\n' \"ush std::env: invalid env name: $1\" >&2\n      return 1\n      ;;\n  esac\n}\n\n",
    );
    emit_fn(
        out,
        "std::env::get",
        "  __ush_std_env_validate \"$1\"\n  eval \"printf '%s' \\\"\\${$1-}\\\"\"\n",
    );
    emit_fn(
        out,
        "std::env::get_or",
        "  __ush_std_env_validate \"$1\"\n  __ush_default=$2\n  eval \"printf '%s' \\\"\\${$1-\\${__ush_default}}\\\"\"\n",
    );
    emit_fn(
        out,
        "std::env::set",
        "  __ush_std_env_validate \"$1\"\n  export \"$1=$2\"\n",
    );
    emit_fn(
        out,
        "std::env::unset",
        "  __ush_std_env_validate \"$1\"\n  unset \"$1\"\n",
    );
}

pub(super) fn runs_in_parent(name: &str) -> bool {
    matches!(name, "std::env::set" | "std::env::unset")
}
