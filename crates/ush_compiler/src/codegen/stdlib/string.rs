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
        "  awk -v value=\"$1\" -v prefix=\"$2\" 'BEGIN { if (substr(value, 1, length(prefix)) == prefix) { printf \"true\" } else { printf \"false\" } }'\n",
    );
    emit_fn(
        out,
        "std::string::ends_with",
        "  awk -v value=\"$1\" -v suffix=\"$2\" 'BEGIN { if (length(suffix) == 0) { printf \"true\"; exit } start = length(value) - length(suffix) + 1; if (start < 1) { printf \"false\"; exit } if (substr(value, start) == suffix) { printf \"true\" } else { printf \"false\" } }'\n",
    );
    emit_fn(
        out,
        "std::string::replace",
        "  awk -v value=\"$1\" -v from=\"$2\" -v to=\"$3\" 'BEGIN { if (from == \"\") { printf \"%s\", value; exit } out = \"\"; while ((idx = index(value, from)) > 0) { out = out substr(value, 1, idx - 1) to; value = substr(value, idx + length(from)) } printf \"%s\", out value }'\n",
    );
    emit_fn(
        out,
        "std::string::trim_prefix",
        "  awk -v value=\"$1\" -v prefix=\"$2\" 'BEGIN { if (substr(value, 1, length(prefix)) == prefix) { value = substr(value, length(prefix) + 1) } printf \"%s\", value }'\n",
    );
    emit_fn(
        out,
        "std::string::trim_suffix",
        "  awk -v value=\"$1\" -v suffix=\"$2\" 'BEGIN { if (length(suffix) > 0 && length(value) >= length(suffix) && substr(value, length(value) - length(suffix) + 1) == suffix) { value = substr(value, 1, length(value) - length(suffix)) } printf \"%s\", value }'\n",
    );
}
