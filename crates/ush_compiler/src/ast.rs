use alloc::boxed::Box;

use crate::errors::ErrorSet;
use crate::types::{AstString as String, HeapVec as Vec};

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    Enum(EnumDef),
    Trait(TraitDef),
    Impl(TraitImpl),
    Function(FunctionDef),
    Alias {
        name: String,
        value: Expr,
    },
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
    TryShell(Expr),
    Raise(Expr),
    Expr(Expr),
    Call(Call),
    TryCall(Call),
    Return(Expr),
    Match {
        expr: Expr,
        arms: Vec<(Pattern, Box<Statement>)>,
        returns_value: bool,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    String(String),
    Int(i64),
    Bool(bool),
    Unit,
    Var(String),
    Add(Vec<Expr>),
    Compare {
        lhs: Box<Expr>,
        op: CompareOp,
        rhs: Box<Expr>,
    },
    Try(Box<Expr>),
    Call(Call),
    Variant(VariantExpr),
}

#[derive(Debug, Clone)]
pub(crate) enum Pattern {
    Wildcard,
    Binding(String),
    String(String),
    Int(i64),
    Bool(bool),
    Unit,
    Variant(VariantPattern),
}

#[derive(Debug, Clone)]
pub(crate) struct EnumDef {
    pub name: String,
    pub variants: Vec<VariantDef>,
}

#[derive(Debug, Clone)]
pub(crate) struct TraitDef {
    pub name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TraitImpl {
    pub trait_name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub(crate) struct Attribute {
    pub name: String,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionDef {
    pub attrs: Vec<Attribute>,
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<Type>,
    pub declared_errors: Option<ErrorSet>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub(crate) struct Call {
    pub name: String,
    pub args: Vec<CallArg>,
    pub asynchronous: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionParam {
    pub name: String,
    pub ty: Type,
    pub default: Option<Expr>,
    pub cli_alias: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct CallArg {
    pub label: Option<String>,
    pub expr: Expr,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Type {
    String,
    Int,
    Bool,
    Unit,
    Adt(String),
    Task(Box<Type>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Type {
    pub(crate) fn render(&self) -> String {
        match self {
            Self::String => "String".into(),
            Self::Int => "Int".into(),
            Self::Bool => "Bool".into(),
            Self::Unit => "()".into(),
            Self::Adt(name) => name.clone(),
            Self::Task(inner) => format!("Task<{}>", inner.render()).into(),
        }
    }
}
