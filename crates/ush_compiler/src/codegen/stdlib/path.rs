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
        builtin("std::path::source_root", Vec::new(), Some(Type::String)),
        builtin("std::path::tmpfile", Vec::new(), Some(Type::String)),
        builtin(
            "std::path::from_cwd",
            vec![param("path", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::path::from_source",
            vec![param("path", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::path::resolve",
            vec![param("path", Type::String)],
            Some(Type::String),
        ),
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
    out.push_str(
        "__ush_std_path_join_parts() {\n  __ush_left=$1\n  __ush_right=$2\n  if [ -z \"$__ush_left\" ]; then\n    printf '%s' \"$__ush_right\"\n    return 0\n  fi\n  if [ -z \"$__ush_right\" ]; then\n    printf '%s' \"$__ush_left\"\n    return 0\n  fi\n  while [ \"$__ush_left\" != \"/\" ] && [ \"${__ush_left%/}\" != \"$__ush_left\" ]; do\n    __ush_left=${__ush_left%/}\n  done\n  while [ \"${__ush_right#/}\" != \"$__ush_right\" ]; do\n    __ush_right=${__ush_right#/}\n  done\n  case \"$__ush_left\" in\n    /) printf '/%s' \"$__ush_right\" ;;\n    *) printf '%s/%s' \"$__ush_left\" \"$__ush_right\" ;;\n  esac\n}\n\n",
    );
    out.push_str(
        "__ush_std_path_make_ref() {\n  __ush_kind=$1\n  __ush_value=$2\n  case \"$__ush_value\" in\n    ''|/*|__ush_pathref__:*) printf '%s' \"$__ush_value\" ;;\n    *) printf '__ush_pathref__:%s:%s' \"$__ush_kind\" \"$__ush_value\" ;;\n  esac\n}\n\n",
    );
    out.push_str(
        "__ush_std_path_resolve() {\n  __ush_value=$1\n  case \"$__ush_value\" in\n    __ush_pathref__:cwd:*)\n      __ush_std_path_join_parts \"$(pwd)\" \"${__ush_value#__ush_pathref__:cwd:}\"\n      ;;\n    __ush_pathref__:source:*)\n      __ush_base=${__ush_source_dir:-}\n      if [ -z \"$__ush_base\" ]; then\n        __ush_base=$(pwd)\n      fi\n      __ush_std_path_join_parts \"$__ush_base\" \"${__ush_value#__ush_pathref__:source:}\"\n      ;;\n    *) printf '%s' \"$__ush_value\" ;;\n  esac\n}\n\n",
    );
    out.push_str(
        "__ush_std_path_dirname_ref() {\n  __ush_value=$1\n  case \"$__ush_value\" in\n    __ush_pathref__:cwd:*)\n      printf '__ush_pathref__:cwd:%s' \"$(dirname \"${__ush_value#__ush_pathref__:cwd:}\")\"\n      ;;\n    __ush_pathref__:source:*)\n      printf '__ush_pathref__:source:%s' \"$(dirname \"${__ush_value#__ush_pathref__:source:}\")\"\n      ;;\n    *) dirname \"$__ush_value\" ;;\n  esac\n}\n\n",
    );
    out.push_str(
        "__ush_std_path_join() {\n  __ush_left=$1\n  __ush_right=$2\n  case \"$__ush_left\" in\n    __ush_pathref__:cwd:*)\n      case \"$__ush_right\" in\n        /*) printf '%s' \"$__ush_right\" ;;\n        __ush_pathref__:*) __ush_std_path_join_parts \"$(__ush_std_path_resolve \"$__ush_left\")\" \"$(__ush_std_path_resolve \"$__ush_right\")\" ;;\n        *) printf '__ush_pathref__:cwd:%s' \"$(__ush_std_path_join_parts \"${__ush_left#__ush_pathref__:cwd:}\" \"$__ush_right\")\" ;;\n      esac\n      ;;\n    __ush_pathref__:source:*)\n      case \"$__ush_right\" in\n        /*) printf '%s' \"$__ush_right\" ;;\n        __ush_pathref__:*) __ush_std_path_join_parts \"$(__ush_std_path_resolve \"$__ush_left\")\" \"$(__ush_std_path_resolve \"$__ush_right\")\" ;;\n        *) printf '__ush_pathref__:source:%s' \"$(__ush_std_path_join_parts \"${__ush_left#__ush_pathref__:source:}\" \"$__ush_right\")\" ;;\n      esac\n      ;;\n    *) __ush_std_path_join_parts \"$(__ush_std_path_resolve \"$__ush_left\")\" \"$(__ush_std_path_resolve \"$__ush_right\")\" ;;\n  esac\n}\n\n",
    );
    emit_fn(out, "std::path::cwd", "  pwd\n");
    emit_fn(out, "std::path::home", "  printf '%s' \"${HOME:-}\"\n");
    emit_fn(
        out,
        "std::path::source_root",
        "  if [ -n \"${__ush_source_dir:-}\" ]; then\n    printf '%s' \"$__ush_source_dir\"\n  else\n    pwd\n  fi\n",
    );
    emit_fn(
        out,
        "std::path::tmpfile",
        "  __ush_tmp_seq=$((__ush_tmp_seq + 1))\n  __ush_tmp_path=\"${TMPDIR:-/tmp}/ush.$$.$__ush_tmp_seq.tmp\"\n  : > \"$__ush_tmp_path\"\n  printf '%s' \"$__ush_tmp_path\"\n",
    );
    emit_fn(
        out,
        "std::path::from_cwd",
        "  __ush_std_path_make_ref cwd \"$1\"\n",
    );
    emit_fn(
        out,
        "std::path::from_source",
        "  __ush_std_path_make_ref source \"$1\"\n",
    );
    emit_fn(
        out,
        "std::path::resolve",
        "  __ush_std_path_resolve \"$1\"\n",
    );
    emit_fn(
        out,
        "std::path::join",
        "  __ush_std_path_join \"$1\" \"$2\"\n",
    );
    emit_fn(
        out,
        "std::path::dirname",
        "  __ush_std_path_dirname_ref \"$1\"\n",
    );
    emit_fn(
        out,
        "std::path::basename",
        "  basename \"$(__ush_std_path_resolve \"$1\")\"\n",
    );
    emit_fn(
        out,
        "std::path::exists",
        "  __ush_path=$(__ush_std_path_resolve \"$1\")\n  if [ -e \"$__ush_path\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::path::is_file",
        "  __ush_path=$(__ush_std_path_resolve \"$1\")\n  if [ -f \"$__ush_path\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::path::is_dir",
        "  __ush_path=$(__ush_std_path_resolve \"$1\")\n  if [ -d \"$__ush_path\" ]; then printf '%s' 'true'; else printf '%s' 'false'; fi\n",
    );
    emit_fn(
        out,
        "std::path::mkdir_p",
        "  mkdir -p \"$(__ush_std_path_resolve \"$1\")\"\n",
    );
    emit_fn(
        out,
        "std::path::prepend_env",
        "  __ush_std_env_validate \"$1\"\n  __ush_segment=$(__ush_std_path_resolve \"$2\")\n  __ush_current=\"$(ush_fn_std__env__get \"$1\")\"\n  if [ -n \"$__ush_current\" ]; then\n    export \"$1=$__ush_segment:$__ush_current\"\n  else\n    export \"$1=$__ush_segment\"\n  fi\n",
    );
}

pub(super) fn runs_in_parent(name: &str) -> bool {
    matches!(name, "std::path::prepend_env")
}
