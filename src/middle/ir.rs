use serde::Serialize;

use crate::frontend::ast::Expr;

// -------------------------------------------------
// TYPE SYSTEM
// -------------------------------------------------
#[derive(Debug, Clone)]
pub enum Type {
    I32,
    F64,
    Bool,
    Str,
    Void,
    Ptr(Box<Type>),
    Unknown, // useful for partial inference / dynamic
}

// -------------------------------------------------
// TYPED EXPRESSION (semantic result of lowering)
// -------------------------------------------------
#[derive(Clone)]
pub struct TypedExpr(pub Expr, pub Type);

impl std::fmt::Debug for TypedExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedExpr")
            .field("expr", &self.0)
            .field("ty", &self.1)
            .finish()
    }
}
// #[derive(Debug, Serialize, Clone)]

#[derive(Debug, Serialize, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Cmp,
}
// -------------------------------------------------
// IR ROOT
// -------------------------------------------------
#[derive(Debug, Clone)]
pub enum IR {
    Module {
        body: Vec<IR>,
    },

    // VARIABLES
    Declare {
        name: String,
        value: TypedExpr,
        mutable: bool,
        dynamic: bool,
    },

    Assign {
        name: String,
        value: TypedExpr,
    },

    Load {
        name: String,
    },

    // EXPRESSIONS
    ExprStmt {
        expr: TypedExpr,
    },

    // I/O
    Print {
        value: TypedExpr,
    },

    // CONTROL FLOW
    If {
        condition: TypedExpr,
        then_branch: Vec<IR>,
        else_branch: Vec<IR>,
    },

    While {
        condition: TypedExpr,
        body: Vec<IR>,
    },

    Block {
        body: Vec<IR>,
    },

    // FUNCTIONS
    Function {
        name: String,
        params: Vec<(String, Type)>,
        body: Vec<IR>,
        return_type: Type,
    },

    Call {
        name: String,
        args: Vec<TypedExpr>,
    },

    Return {
        value: Option<TypedExpr>,
    },

    // Binary operations: target = left op right
    Binary {
        target: String,
        left: String,
        op: Op,
        right: String,
    },
    // Assignment: target = source
    Move {
        target: String,
        source: String,
    },
    // Control flow
    Label(String),
    Jump(String),
    JumpIf {
        condition: String,
        label: String,
    },

    Nop,
}
