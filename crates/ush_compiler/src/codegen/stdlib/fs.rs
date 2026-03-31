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
    emit_fn(out, "std::fs::read_text", "  cat \"$1\"\n");
    emit_fn(
        out,
        "std::fs::write_text",
        "  printf '%s' \"$2\" > \"$1\"\n",
    );
    emit_fn(
        out,
        "std::fs::append_text",
        "  printf '%s' \"$2\" >> \"$1\"\n",
    );
    emit_fn(out, "std::fs::remove", "  rm -f \"$1\"\n");
    emit_fn(out, "std::fs::move", "  mv \"$1\" \"$2\"\n");
    emit_fn(out, "std::fs::copy", "  cp \"$1\" \"$2\"\n");
    emit_fn(
        out,
        "std::fs::exists",
        "  if [ -e \"$1\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::fs::is_file",
        "  if [ -f \"$1\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::fs::is_dir",
        "  if [ -d \"$1\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(out, "std::fs::mkdir_p", "  mkdir -p \"$1\"\n");
    emit_fn(
        out,
        "std::fs::tmpfile",
        "  __ush_tmp_seq=$((__ush_tmp_seq + 1))\n  __ush_tmp_path=\"${TMPDIR:-/tmp}/ush.$$.$__ush_tmp_seq.tmp\"\n  : > \"$__ush_tmp_path\"\n  printf '%s' \"$__ush_tmp_path\"\n",
    );
    emit_fn(
        out,
        "std::fs::sha256",
        "  if command -v sha256sum >/dev/null 2>&1; then\n    sha256sum \"$1\" | awk '{print $1}'\n    return 0\n  fi\n  if command -v shasum >/dev/null 2>&1; then\n    shasum -a 256 \"$1\" | awk '{print $1}'\n    return 0\n  fi\n  if command -v openssl >/dev/null 2>&1; then\n    openssl dgst -sha256 \"$1\" | awk '{print $NF}'\n    return 0\n  fi\n  printf '%s\\n' 'ush std::fs::sha256: no hash tool available' >&2\n  return 1\n",
    );
    emit_fn(
        out,
        "std::fs::mime_type",
        "  if ! command -v file >/dev/null 2>&1; then\n    printf '%s' 'application/octet-stream'\n    return 0\n  fi\n  if file -b --mime-type \"$1\" >/dev/null 2>&1; then\n    file -b --mime-type \"$1\"\n    return 0\n  fi\n  file -I \"$1\" | awk -F': ' '{print $2}' | awk -F';' '{print $1}'\n",
    );
}

pub(super) fn runs_in_parent(_name: &str) -> bool {
    false
}
