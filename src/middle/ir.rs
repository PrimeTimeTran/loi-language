use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

use crate::frontend::ast::{BinOp, Expr};

use crate::{backend::symbol::registry::Symbol, frontend};

pub type IrInstruction = IROp;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedExpr {
    pub expr: Expr,
    pub ty: Type,
    pub span: Span,
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

pub struct IR {
    pub raw: String,
    pub nodes: Vec<IROp>,
    pub symbols: HashMap<String, Symbol>,
    pub metadata: HashMap<String, String>,
}

impl Default for IR {
    fn default() -> Self {
        Self::raw(String::new())
    }
}

impl fmt::Display for IR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // -------------------------
        // RAW BLOCK (passthrough)
        // -------------------------
        if !self.raw.is_empty() {
            writeln!(f, "--- Raw Block ---")?;
            writeln!(f, "{}", self.raw)?;
            return Ok(());
        }

        // -------------------------
        // METADATA
        // -------------------------
        writeln!(f, "--- Metadata ---")?;
        for (k, v) in &self.metadata {
            writeln!(f, "{}: {}", k, v)?;
        }

        // -------------------------
        // SYMBOLS
        // -------------------------
        writeln!(f, "--- Symbols ---")?;
        for name in self.symbols.keys() {
            writeln!(f, "Export: {}", name)?;
        }

        // -------------------------
        // BODY
        // -------------------------
        writeln!(f, "--- Body ---")?;
        for op in &self.nodes {
            writeln!(f, "{:?}", op)?;
        }

        Ok(())
    }
}

impl IR {
    /// Empty structured IR
    pub fn new() -> Self {
        Self {
            raw: String::new(),
            nodes: Vec::new(),
            symbols: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// IR containing foreign/raw block
    pub fn raw(content: impl Into<String>) -> Self {
        Self {
            raw: content.into(),
            nodes: Vec::new(),
            symbols: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Explicit structured IR
    pub fn structured() -> Self {
        Self::new()
    }

    /// Helper: check if this is a passthrough block
    pub fn is_raw(&self) -> bool {
        !self.raw.is_empty()
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
    Nop,
    Binary {
        target: String,
        left: TypedExpr,
        op: BinOp,
        right: TypedExpr,
    },
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
    If {
        condition: TypedExpr,
        then_branch: Vec<IROp>,
        else_branch: Vec<IROp>,
        scope_id: usize,
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

    Lowered(LoweredOp),
}

//         PARSER
//            ↓
// AST with RegionBlocks
//            ↓
//     IR NORMALIZATION
//            ↓
// Region Processor Dispatch
//  ├── JS → esbuild
//  ├── TS → swc
//  ├── SQL → sql engine
//            ↓
// Injected back into IR
//            ↓
//       BACKEND
pub enum RegionKind {
    Native,
    JavaScript,
    TypeScript,
    JSX,
    SQL,
    Python,
    Shader,
    Unknown(String),
}
pub enum RegionMode {
    /// ignore, pass through
    Passthrough,

    /// compile externally, then inject result
    CompileAndInject,

    /// compile but keep source too (debuggable)
    Dual,
}

pub struct RegionBlock {
    /// language or mode of the block
    pub kind: RegionKind,

    /// raw source inside the region
    pub source: String,

    /// result after processing (optional)
    pub output: Option<Vec<u8>>,

    /// whether it should be compiled or just passed through
    pub mode: RegionMode,
}

pub trait RegionProcessor {
    fn process(&self, block: &RegionBlock) -> Vec<u8>;
}
