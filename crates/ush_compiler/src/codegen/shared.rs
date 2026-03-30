use super::super::{
    ast::Type,
    env::{Binding, Storage},
};
use crate::types::{AstString as NameString, OutputString as String};

pub(crate) fn binding_for_name(name: &str, ty: Type) -> Binding {
    let storage = match ty {
        Type::Adt(_) => Storage::Adt(NameString::from(name)),
        Type::String | Type::Int | Type::Bool | Type::Unit => {
            Storage::Primitive(NameString::from(name))
        }
        Type::Task(_) => Storage::Task(NameString::from(name)),
    };
    Binding { ty, storage }
}

pub(crate) fn push_block(out: &mut String, block: &str, indent: usize) {
    for line in block.lines() {
        push_line(out, line, indent);
    }
}

pub(crate) fn push_line(out: &mut String, line: &str, indent: usize) {
    out.push_str(&" ".repeat(indent));
    out.push_str(line);
    out.push('\n');
}
