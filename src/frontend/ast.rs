use serde::Serialize;

use crate::frontend::parser::{parse, parse_source};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DeclKind {
    MutableStatic,   // =
    ImmutableStatic, // =!
    Dynamic,         // =?
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        kind: DeclKind,
        value: Expr,
    },

    Print {
        expr: Expr,
    },

    ExprStmt {
        expr: Expr,
    },

    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    For {
        iterator: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
    },
    Block {
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    And,
    Or,
    Assign,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum UnOp {
    Neg,
    Not,
    AddrOf,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expr {
    Number(f64),
    Bool(bool),
    String(String),
    Var(String),
    Array(Vec<Expr>),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Member {
        target: Box<Expr>,
        field: String,
    },
    Assign {
        left: Box<Expr>,
        right: Box<Expr>,
    },
}
