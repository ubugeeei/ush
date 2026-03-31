use crate::{
    ast::{FunctionDef, Type},
    sourcemap::OutputBuffer,
    types::HeapVec as Vec,
};

use super::{builtin, emit_fn, param};

pub(super) fn definitions() -> Vec<FunctionDef> {
    vec![
        builtin(
            "std::fs::read_text",
            vec![param("path", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::fs::write_text",
            vec![param("path", Type::String), param("content", Type::String)],
            Some(Type::Unit),
        ),
        builtin(
            "std::fs::append_text",
            vec![param("path", Type::String), param("content", Type::String)],
            Some(Type::Unit),
        ),
        builtin(
            "std::fs::remove",
            vec![param("path", Type::String)],
            Some(Type::Unit),
        ),
        builtin(
            "std::fs::move",
            vec![param("from", Type::String), param("to", Type::String)],
            Some(Type::Unit),
        ),
        builtin(
            "std::fs::copy",
            vec![param("from", Type::String), param("to", Type::String)],
            Some(Type::Unit),
        ),
        builtin(
            "std::fs::exists",
            vec![param("path", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::fs::is_file",
            vec![param("path", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::fs::is_dir",
            vec![param("path", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::fs::mkdir_p",
            vec![param("path", Type::String)],
            Some(Type::Unit),
        ),
        builtin("std::fs::tmpfile", Vec::new(), Some(Type::String)),
        builtin(
            "std::fs::sha256",
            vec![param("path", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::fs::mime_type",
            vec![param("path", Type::String)],
            Some(Type::String),
        ),
    ]
}

pub(super) fn emit(out: &mut OutputBuffer) {
    emit_fn(
        out,
        "std::fs::read_text",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  cat \"$__ush_path\"\n",
    );
    emit_fn(
        out,
        "std::fs::write_text",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  printf '%s' \"$2\" > \"$__ush_path\"\n",
    );
    emit_fn(
        out,
        "std::fs::append_text",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  printf '%s' \"$2\" >> \"$__ush_path\"\n",
    );
    emit_fn(
        out,
        "std::fs::remove",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  rm -f \"$__ush_path\"\n",
    );
    emit_fn(
        out,
        "std::fs::move",
        "  __ush_from=$(ush_fn_std__path__resolve \"$1\")\n  __ush_to=$(ush_fn_std__path__resolve \"$2\")\n  mv \"$__ush_from\" \"$__ush_to\"\n",
    );
    emit_fn(
        out,
        "std::fs::copy",
        "  __ush_from=$(ush_fn_std__path__resolve \"$1\")\n  __ush_to=$(ush_fn_std__path__resolve \"$2\")\n  cp \"$__ush_from\" \"$__ush_to\"\n",
    );
    emit_fn(
        out,
        "std::fs::exists",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  if [ -e \"$__ush_path\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::fs::is_file",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  if [ -f \"$__ush_path\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::fs::is_dir",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  if [ -d \"$__ush_path\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::fs::mkdir_p",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  mkdir -p \"$__ush_path\"\n",
    );
    emit_fn(
        out,
        "std::fs::tmpfile",
        "  __ush_tmp_seq=$((__ush_tmp_seq + 1))\n  __ush_tmp_path=\"${TMPDIR:-/tmp}/ush.$$.$__ush_tmp_seq.tmp\"\n  : > \"$__ush_tmp_path\"\n  printf '%s' \"$__ush_tmp_path\"\n",
    );
    emit_fn(
        out,
        "std::fs::sha256",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  if command -v sha256sum >/dev/null 2>&1; then\n    sha256sum \"$__ush_path\" | awk '{print $1}'\n    return 0\n  fi\n  if command -v shasum >/dev/null 2>&1; then\n    shasum -a 256 \"$__ush_path\" | awk '{print $1}'\n    return 0\n  fi\n  if command -v openssl >/dev/null 2>&1; then\n    openssl dgst -sha256 \"$__ush_path\" | awk '{print $NF}'\n    return 0\n  fi\n  printf '%s\\n' 'ush std::fs::sha256: no hash tool available' >&2\n  return 1\n",
    );
    emit_fn(
        out,
        "std::fs::mime_type",
        "  __ush_path=$(ush_fn_std__path__resolve \"$1\")\n  if ! command -v file >/dev/null 2>&1; then\n    printf '%s' 'application/octet-stream'\n    return 0\n  fi\n  if file -b --mime-type \"$__ush_path\" >/dev/null 2>&1; then\n    file -b --mime-type \"$__ush_path\"\n    return 0\n  fi\n  file -I \"$__ush_path\" | awk -F': ' '{print $2}' | awk -F';' '{print $1}'\n",
    );
}

pub(super) fn runs_in_parent(_name: &str) -> bool {
    false
}
