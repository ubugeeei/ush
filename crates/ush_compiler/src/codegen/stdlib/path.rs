use crate::types::HeapVec as Vec;
use crate::{
    ast::{FunctionDef, Type},
    sourcemap::OutputBuffer,
};

use super::{builtin, emit_fn, param};

pub(super) fn definitions() -> Vec<FunctionDef> {
    vec![
        builtin("std::path::cwd", Vec::new(), Some(Type::String)),
        builtin("std::path::home", Vec::new(), Some(Type::String)),
        builtin("std::path::tmpfile", Vec::new(), Some(Type::String)),
        builtin(
            "std::path::join",
            vec![param("left", Type::String), param("right", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::path::dirname",
            vec![param("path", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::path::basename",
            vec![param("path", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::path::exists",
            vec![param("path", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::path::is_file",
            vec![param("path", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::path::is_dir",
            vec![param("path", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::path::mkdir_p",
            vec![param("path", Type::String)],
            Some(Type::Unit),
        ),
        builtin(
            "std::path::prepend_env",
            vec![param("name", Type::String), param("segment", Type::String)],
            Some(Type::Unit),
        ),
    ]
}

pub(super) fn emit(out: &mut OutputBuffer) {
    emit_fn(out, "std::path::cwd", "  pwd\n");
    emit_fn(out, "std::path::home", "  printf '%s' \"${HOME:-}\"\n");
    emit_fn(
        out,
        "std::path::tmpfile",
        "  __ush_tmp_seq=$((__ush_tmp_seq + 1))\n  __ush_tmp_path=\"${TMPDIR:-/tmp}/ush.$$.$__ush_tmp_seq.tmp\"\n  : > \"$__ush_tmp_path\"\n  printf '%s' \"$__ush_tmp_path\"\n",
    );
    emit_fn(
        out,
        "std::path::join",
        "  __ush_left=$1\n  __ush_right=$2\n  if [ -z \"$__ush_left\" ]; then\n    printf '%s' \"$__ush_right\"\n    return 0\n  fi\n  if [ -z \"$__ush_right\" ]; then\n    printf '%s' \"$__ush_left\"\n    return 0\n  fi\n  while [ \"$__ush_left\" != \"/\" ] && [ \"${__ush_left%/}\" != \"$__ush_left\" ]; do\n    __ush_left=${__ush_left%/}\n  done\n  while [ \"${__ush_right#/}\" != \"$__ush_right\" ]; do\n    __ush_right=${__ush_right#/}\n  done\n  case \"$__ush_left\" in\n    /) printf '/%s' \"$__ush_right\" ;;\n    *) printf '%s/%s' \"$__ush_left\" \"$__ush_right\" ;;\n  esac\n",
    );
    emit_fn(out, "std::path::dirname", "  dirname \"$1\"\n");
    emit_fn(out, "std::path::basename", "  basename \"$1\"\n");
    emit_fn(
        out,
        "std::path::exists",
        "  if [ -e \"$1\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::path::is_file",
        "  if [ -f \"$1\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::path::is_dir",
        "  if [ -d \"$1\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(out, "std::path::mkdir_p", "  mkdir -p \"$1\"\n");
    emit_fn(
        out,
        "std::path::prepend_env",
        "  __ush_std_env_validate \"$1\"\n  __ush_current=\"$(ush_fn_std__env__get \"$1\")\"\n  if [ -n \"$__ush_current\" ]; then\n    export \"$1=$2:$__ush_current\"\n  else\n    export \"$1=$2\"\n  fi\n",
    );
}

pub(super) fn runs_in_parent(name: &str) -> bool {
    matches!(name, "std::path::prepend_env")
}
