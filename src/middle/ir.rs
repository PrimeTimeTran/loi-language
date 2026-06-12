use std::collections::HashMap;

use serde::Serialize;

use frontend::ast::Expr;

use crate::{backend::symbol::registry::Symbol, frontend};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I32,
    F64,
    Bool,
    Str,
    Void,
    Ptr(Box<Type>),
    Array(Box<Type>),

    Unknown,
}

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

#[derive(Debug, Serialize, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Cmp,
    Neg,
}
pub enum IR {
    Raw(String),
    Structured {
        body: Vec<IROp>,
        symbols: HashMap<String, Symbol>,
        metadata: HashMap<String, String>,
    },
}

impl Default for IR {
    fn default() -> Self {
        Self::Raw(String::new())
    }
}

use std::fmt;

impl fmt::Display for IR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IR::Raw(content) => write!(f, "{}", content),
            IR::Structured {
                body,
                symbols,
                metadata,
            } => {
                // Example of how to flatten structured IR into a string
                // You can customize this format based on your needs
                writeln!(f, "--- Metadata ---")?;
                for (k, v) in metadata {
                    writeln!(f, "{}: {}", k, v)?;
                }
                writeln!(f, "--- Symbols ---")?;
                for name in symbols.keys() {
                    writeln!(f, "Export: {}", name)?;
                }
                writeln!(f, "--- Body ---")?;
                for op in body {
                    writeln!(f, "{:?}", op)?; // Assumes IROp implements Debug
                }
                Ok(())
            }
        }
    }
}

impl IR {
    pub fn new() -> Self {
        IR::Structured {
            body: Vec::new(),
            symbols: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn raw(content: impl Into<String>) -> Self {
        IR::Raw(content.into())
    }

    pub fn structured() -> Self {
        IR::Structured {
            body: Vec::new(),
            symbols: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoweredOp {
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

#[derive(Debug, Clone)]
pub enum IROp {
    // --- Program Structure ---
    Module {
        body: Vec<IROp>,
    },
    Function {
        name: String,
        params: Vec<(String, Type)>,
        body: Vec<IROp>,
        return_type: Type,
    },
    Block {
        body: Vec<IROp>,
    },

    // --- High-Level Logic ---
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
    ExprStmt {
        expr: TypedExpr,
    },

    // --- Control Flow & I/O ---
    If {
        condition: TypedExpr,
        then_branch: Vec<IROp>,
        else_branch: Vec<IROp>,
    },
    Return {
        value: Option<TypedExpr>,
    },
    Call {
        name: String,
        args: Vec<TypedExpr>,
    },
    Print {
        value: TypedExpr,
    },
    ExternalCall {
        namespace: String,
        function: String,
        args: Vec<TypedExpr>,
    },

    ModuleScope {
        name: String,
        body: Vec<IROp>,
    },

    While {
        condition: TypedExpr,
        body: Vec<IROp>,
    },

    DoWhile {
        body: Vec<IROp>,
        condition: TypedExpr,
    },

    Loop {
        body: Vec<IROp>,
    },

    For {
        iterator: String,
        iterable: TypedExpr,
        body: Vec<IROp>,
    },

    // --- The Bridge: Lowered Instructions ---
    Lowered(LoweredOp),
}
