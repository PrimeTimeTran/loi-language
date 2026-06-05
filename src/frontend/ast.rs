use crate::middle::ir::{BinOp, UnOp};

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
#[derive(Debug, Clone)]
pub enum Stmt {
    Assign { name: String, expr: Expr },
    Print { expr: Expr },
    ExprStmt { expr: Expr },
}
#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Var(String),

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
        name: String,
        args: Vec<Expr>,
    },
}
