use crate::types::HeapVec as Vec;
use crate::{
    ast::{FunctionDef, Type},
    sourcemap::OutputBuffer,
};

use super::{builtin, emit_fn, param};

pub(super) fn definitions() -> Vec<FunctionDef> {
    vec![
        builtin(
            "std::regex::is_match",
            vec![param("value", Type::String), param("pattern", Type::String)],
            Some(Type::Bool),
        ),
        builtin(
            "std::regex::find",
            vec![param("value", Type::String), param("pattern", Type::String)],
            Some(Type::String),
        ),
        builtin(
            "std::regex::replace",
            vec![
                param("value", Type::String),
                param("pattern", Type::String),
                param("replacement", Type::String),
            ],
            Some(Type::String),
        ),
    ]
}

pub(super) fn emit(out: &mut OutputBuffer) {
    emit_fn(
        out,
        "std::regex::is_match",
        "  VALUE=\"$1\" PATTERN=\"$2\" awk 'BEGIN { value = ENVIRON[\"VALUE\"]; pattern = ENVIRON[\"PATTERN\"]; if (value ~ pattern) { printf \"true\" } else { printf \"false\" } }'\n",
    );
    emit_fn(
        out,
        "std::regex::find",
        "  VALUE=\"$1\" PATTERN=\"$2\" awk 'BEGIN { value = ENVIRON[\"VALUE\"]; pattern = ENVIRON[\"PATTERN\"]; if (match(value, pattern)) { printf \"%s\", substr(value, RSTART, RLENGTH) } }'\n",
    );
    emit_fn(
        out,
        "std::regex::replace",
        "  VALUE=\"$1\" PATTERN=\"$2\" REPLACEMENT=\"$3\" awk 'BEGIN { value = ENVIRON[\"VALUE\"]; pattern = ENVIRON[\"PATTERN\"]; replacement = ENVIRON[\"REPLACEMENT\"]; gsub(pattern, replacement, value); printf \"%s\", value }'\n",
    );
}
