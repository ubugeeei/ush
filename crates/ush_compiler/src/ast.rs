use alloc::boxed::Box;

use crate::types::{AstString as String, HeapVec as Vec};

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    Enum(EnumDef),
    Function(FunctionDef),
    Let {
        name: String,
        expr: Expr,
    },
    Spawn {
        name: String,
        call: Call,
    },
    Await {
        name: String,
        task: String,
    },
    Print(Expr),
    Shell(Expr),
    Call(Call),
    Return(Expr),
    Match {
        expr: Expr,
        arms: Vec<(Pattern, Box<Statement>)>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    String(String),
    Int(i64),
    Bool(bool),
    Var(String),
    Add(Vec<Expr>),
    Variant(VariantExpr),
}

#[derive(Debug, Clone)]
pub(crate) enum Pattern {
    Wildcard,
    Binding(String),
    String(String),
    Int(i64),
    Bool(bool),
    Variant(VariantPattern),
}

#[derive(Debug, Clone)]
pub(crate) struct EnumDef {
    pub name: String,
    pub variants: Vec<VariantDef>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionDef {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub(crate) struct Call {
    pub name: String,
    pub args: Vec<Expr>,
    pub asynchronous: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionParam {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub(crate) struct VariantDef {
    pub name: String,
    pub fields: VariantFields,
}

#[derive(Debug, Clone)]
pub(crate) enum VariantFields {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<NamedFieldType>),
}

#[derive(Debug, Clone)]
pub(crate) struct NamedFieldType {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub(crate) struct VariantExpr {
    pub enum_name: String,
    pub variant_name: String,
    pub fields: ExprFields,
}

#[derive(Debug, Clone)]
pub(crate) enum ExprFields {
    Unit,
    Tuple(Vec<Expr>),
    Struct(Vec<NamedExpr>),
}

#[derive(Debug, Clone)]
pub(crate) struct NamedExpr {
    pub name: String,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub(crate) struct VariantPattern {
    pub enum_name: String,
    pub variant_name: String,
    pub fields: PatternFields,
}

#[derive(Debug, Clone)]
pub(crate) enum PatternFields {
    Unit,
    Tuple(Vec<Pattern>),
    Struct(Vec<NamedPattern>),
}

#[derive(Debug, Clone)]
pub(crate) struct NamedPattern {
    pub name: String,
    pub pattern: Pattern,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Type {
    String,
    Int,
    Bool,
    Adt(String),
    Task(Box<Type>),
}
