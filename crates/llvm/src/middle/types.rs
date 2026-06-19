use crate::{
    frontend::ast::{Expr, HashF64, Stmt},
    middle::ir::IROp,
};
use serde::Serialize;
use std::path::PathBuf;

// --- 1. Primitive Support Types ---
#[derive(Debug, Hash, Clone, Default, Eq, PartialEq)]
pub struct Span {
    pub file: PathBuf,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Hash)]
pub enum Type {
    None,
    Empty,
    I32,
    F64,
    Bool,
    Str,
    Void,
    Ptr(Box<Type>),
    Array(Box<Type>),
    Return,
    Unknown,
    Function,
}

// --- 2. Intermediate Representation (IR) ---
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum IRVal {
    Number(HashF64),
    Bool(bool),
    Str(String),
    Var(String),
    Temp(String),
    Unit,
    Function(String),
}

impl IRVal {
    pub fn inferred_type(&self) -> Type {
        match self {
            IRVal::Number(_) => Type::F64,
            IRVal::Bool(_) => Type::Bool,
            IRVal::Str(_) => Type::Str,
            IRVal::Var(_) | IRVal::Temp(_) => Type::Unknown,
            IRVal::Unit => Type::Void,
            IRVal::Function(_) => Type::Function,
        }
    }
}

pub enum LoweredExpr {
    Value(IRVal),
    Op(IROp),
}

// --- 3. Declarations (Building Blocks) ---
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Import {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub glob: bool,
}

// --- 4. AST Nodes (Logic & Statements) ---
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<Variant>,
}

// --- 5. High-Level Containers ---
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ModuleItem {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Const(Stmt),
    Use(Import),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TopLevelItem {
    Module(Module),
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Const(Stmt),
    Use(Import),
    Stmt(Stmt),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<ModuleItem>,
}
