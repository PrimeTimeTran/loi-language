use serde::Serialize;
use std::{fmt, path::PathBuf};

use crate::frontend::ast::{Expr, Stmt};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<ModuleItem>,
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
pub struct Import {
    pub path: Vec<String>, // e.g. ["std", "io", "println"]
    pub alias: Option<String>,
    pub glob: bool, // use std::io::*;
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ModuleItem {
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Const(Stmt),
    Use(Import),
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Hash)]
pub enum Type {
    I32,
    F64,
    Bool,
    Str,
    Void,
    Ptr(Box<Type>),
    Array(Box<Type>),
    Return,
    Unknown,
}

#[derive(Debug, Hash, Clone, Default, Eq, PartialEq)]
pub struct Span {
    pub file: PathBuf,
    pub start: usize,
    pub end: usize,
}
