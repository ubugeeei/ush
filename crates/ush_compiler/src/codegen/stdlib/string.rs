use crate::types::HeapVec as Vec;
use crate::{
    ast::{FunctionDef, Type},
    sourcemap::OutputBuffer,
};

use super::{builtin, emit_fn, param};

pub(super) fn definitions() -> Vec<FunctionDef> {
    vec![
        builtin(
            "std::string::starts_with",
            vec![param("value", Type::String), param("prefix", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::string::ends_with",
            vec![param("value", Type::String), param("suffix", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::string::replace",
            vec![
                param("value", Type::String),
                param("from", Type::String),
                param("to", Type::String),
            ],
            Some(Type::String),
        ),
        builtin(
            "std::string::trim_prefix",
            vec![param("value", Type::String), param("prefix", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::string::trim_suffix",
            vec![param("value", Type::String), param("suffix", Type::String)],
            Some(Type::String),
        ),
    ]
}

pub(super) fn emit(out: &mut OutputBuffer) {
    emit_fn(
        out,
        "std::string::starts_with",
        "  VALUE=\"$1\" PREFIX=\"$2\" awk 'BEGIN { value = ENVIRON[\"VALUE\"]; prefix = ENVIRON[\"PREFIX\"]; if (substr(value, 1, length(prefix)) == prefix) { printf \"true\" } else { printf \"false\" } }'\n",
    );
    emit_fn(
        out,
        "std::string::ends_with",
        "  VALUE=\"$1\" SUFFIX=\"$2\" awk 'BEGIN { value = ENVIRON[\"VALUE\"]; suffix = ENVIRON[\"SUFFIX\"]; if (length(suffix) == 0) { printf \"true\"; exit } start = length(value) - length(suffix) + 1; if (start < 1) { printf \"false\"; exit } if (substr(value, start) == suffix) { printf \"true\" } else { printf \"false\" } }'\n",
    );
    emit_fn(
        out,
        "std::string::replace",
        "  VALUE=\"$1\" FROM=\"$2\" TO=\"$3\" awk 'BEGIN { value = ENVIRON[\"VALUE\"]; from = ENVIRON[\"FROM\"]; to = ENVIRON[\"TO\"]; if (from == \"\") { printf \"%s\", value; exit } out = \"\"; while ((idx = index(value, from)) > 0) { out = out substr(value, 1, idx - 1) to; value = substr(value, idx + length(from)) } printf \"%s\", out value }'\n",
    );
    emit_fn(
        out,
        "std::string::trim_prefix",
        "  VALUE=\"$1\" PREFIX=\"$2\" awk 'BEGIN { value = ENVIRON[\"VALUE\"]; prefix = ENVIRON[\"PREFIX\"]; if (substr(value, 1, length(prefix)) == prefix) { value = substr(value, length(prefix) + 1) } printf \"%s\", value }'\n",
    );
    emit_fn(
        out,
        "std::string::trim_suffix",
        "  VALUE=\"$1\" SUFFIX=\"$2\" awk 'BEGIN { value = ENVIRON[\"VALUE\"]; suffix = ENVIRON[\"SUFFIX\"]; if (length(suffix) > 0 && length(value) >= length(suffix) && substr(value, length(value) - length(suffix) + 1) == suffix) { value = substr(value, 1, length(value) - length(suffix)) } printf \"%s\", value }'\n",
    );
}
