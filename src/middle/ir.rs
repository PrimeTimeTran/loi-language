use std::collections::HashMap;

use serde::Serialize;

use frontend::ast::Expr;

use crate::frontend;

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
    Unknown,
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

// 1. The container that holds the "Global" metadata for the module/file
pub struct IR {
    pub body: Vec<IROp>,
    pub symbols: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

impl IR {
    pub fn new() -> Self {
        Self {
            body: Vec::new(),
            symbols: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IROp {
    Module {
        body: Vec<IROp>,
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
        then_branch: Vec<IROp>,
        else_branch: Vec<IROp>,
    },
    While {
        condition: TypedExpr,
        body: Vec<IROp>,
    },
    Block {
        body: Vec<IROp>,
    },

    // FUNCTIONS
    Function {
        name: String,
        params: Vec<(String, Type)>,
        body: Vec<IROp>,
        return_type: Type,
    },
    Call {
        name: String,
        args: Vec<TypedExpr>,
    },
    Return {
        value: Option<TypedExpr>,
    },

    // LOW-LEVEL / LLVM-ISH
    Binary {
        target: String,
        left: String,
        op: Op,
        right: String,
    },
    Move {
        target: String,
        source: String,
    },
    Label(String),
    Jump(String),
    JumpIf {
        condition: String,
        label: String,
    },

    Nop,
}
