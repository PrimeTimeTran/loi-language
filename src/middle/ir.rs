use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use crate::frontend::ast::{AssignOp, BinOp, Expr, Stmt, UnOp};

use crate::middle::types::{IRVal, Span, Type};
use crate::pipeline::CompileError;
use crate::{backend::symbol::registry::Symbol, frontend};

pub type IrInstruction = IROp;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TypedExpr {
    pub expr: Expr,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Hash, Serialize, Clone, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Cmp,
    Neg,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct IR {
    // ========================================
    // LEGACY / TRANSITIONAL (DO NOT EXPAND)
    // ========================================
    /// ⚠️ LEGACY: raw debug representation of IR.
    /// Eventually remove once debugging moves to structured tooling.
    pub raw: String,

    /// ⚠️ LEGACY: flat instruction stream.
    /// Migration target: FunctionIR → BasicBlock → ops
    pub nodes: Vec<IROp>,

    /// ⚠️ SHOULD MOVE TO CompileState (symbol system owns this)
    pub symbols: HashMap<String, Symbol>,

    /// ⚠️ REPLACE WITH STRUCTURED IR METADATA (IRMeta / CompilerState)
    pub metadata: HashMap<String, String>,

    /// ⚠️ DUPLICATE OF `nodes` (remove once migration completes)
    pub ops: Vec<IROp>,

    // ========================================
    // MODERN IR STRUCTURE (TARGET DESIGN)
    // ========================================
    /// ⭐ FINAL TARGET REPRESENTATION OF IR
    /// This is the ONLY structure you should ultimately keep.
    ///
    /// MIGRATION TARGET:
    /// - move all ops into BasicBlock
    /// - move blocks into functions
    /// - eliminate flat instruction forms
    pub modules: Vec<ModuleIR>,
    pub functions: Vec<FunctionIR>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleIR {
    /// Module name / path
    pub name: String,

    /// Functions defined in this module
    pub functions: Vec<FunctionIR>,

    /// Optional globals (consts, static values)
    pub globals: Vec<IROp>,

    /// Exported symbols (public API of module)
    pub exports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionIR {
    pub name: String,
    pub blocks: Vec<BasicBlock>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub ops: Vec<IROp>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum IROp {
    ExprStmt {
        expr: IRVal,
    },
    Temp {
        value: IRVal,
    },

    Unary {
        op: UnOp,
        value: IRVal,
        // target: Option<String>,
    },

    Array {
        values: Vec<IRVal>,
        // target: Option<String>,
    },
    Assign {
        name: String,
        value: IRVal,
    },
    // Assign {
    //     target: IRVal,
    //     value: IRVal,
    //     op: AssignOp,
    // },
    Expr {
        value: IRVal,
    },
    Print {
        value: IRVal,
    },
    Declare {
        name: String,
        value: IRVal,
        mutable: bool,
        dynamic: bool,
    },
    Return {
        value: Option<IRVal>,
    },
    Nop,
    Binary {
        left: IRVal,
        op: BinOp,
        right: IRVal,
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

    Load {
        name: String,
    },
    If {
        condition: IRVal,
        then_branch: Vec<IROp>,
        else_branch: Vec<IROp>,
        scope_id: usize,
    },

    Call {
        name: String,
        args: Vec<IRVal>,
    },

    ExternalCall {
        namespace: String,
        function: String,
        args: Vec<IRVal>,
    },

    ModuleScope {
        name: String,
        body: Vec<IROp>,
    },

    While {
        condition: IRVal,
        body: Vec<IROp>,
    },

    DoWhile {
        body: Vec<IROp>,
        condition: IRVal,
    },

    Loop {
        body: Vec<IROp>,
    },

    For {
        iterator: String,
        iterable: IRVal,
        body: Vec<IROp>,
    },

    ControlFlow,

    Lowered(LoweredOp),
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
    /// Create an empty IR (no modules)
    pub fn new() -> Self {
        Self {
            raw: String::new(),
            nodes: Vec::new(),
            symbols: HashMap::new(),
            metadata: HashMap::new(),
            ops: Vec::new(),
            modules: Vec::new(),
            functions: Vec::new(),
        }
    }

    /// Create IR from a single module
    pub fn from_module(module: ModuleIR) -> Self {
        Self {
            raw: String::new(),
            nodes: Vec::new(),
            symbols: HashMap::new(),
            metadata: HashMap::new(),
            ops: Vec::new(),
            modules: vec![module],
            functions: Vec::new(),
        }
    }

    /// Create IR from raw operations (legacy bridge only)
    ///
    /// ⚠️ Transitional API — will disappear once modules/functions are stable
    pub fn from_ops(ops: Vec<IROp>) -> Self {
        Self {
            raw: String::new(),
            nodes: ops.clone(),
            ops,
            symbols: HashMap::new(),
            metadata: HashMap::new(),
            modules: Vec::new(),
            functions: Vec::new(),
        }
    }

    /// Attach metadata (stage, debug, etc.)
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// ⚠️ Legacy debug helper (DO NOT build logic on this)
    pub fn raw(content: impl Into<String>) -> Self {
        Self {
            raw: content.into(),
            nodes: Vec::new(),
            ops: Vec::new(),
            symbols: HashMap::new(),
            metadata: HashMap::new(),
            modules: Vec::new(),
            functions: Vec::new(),
        }
    }
}

impl Type {
    pub fn to_llvm_type<'ctx>(
        &self,
        context: &'ctx inkwell::context::Context,
    ) -> inkwell::types::BasicTypeEnum<'ctx> {
        match self {
            Type::F64 => context.f64_type().into(),
            Type::I32 => context.i32_type().into(),
            // Add other variants here...
            _ => panic!("Type conversion not implemented yet"),
        }
    }
}

fn to_typed_expr(expr: Expr) -> TypedExpr {
    TypedExpr {
        span: expr.span(),
        expr,
        ty: Type::Unknown,
    }
}

fn expr_to_irval(expr: Expr) -> IRVal {
    match expr {
        Expr::Number(n) => IRVal::Number(n),
        Expr::Bool(b) => IRVal::Bool(b),
        Expr::String(s) => IRVal::Str(s),
        Expr::Var(v) => IRVal::Var(v),
        _ => {
            panic!("complex expressions must be lowered in analyze_stmt, not From<Stmt>")
        }
    }
}

impl From<Stmt> for IROp {
    fn from(stmt: Stmt) -> Self {
        match stmt {
            Stmt::ExprStmt { expr } => IROp::ExprStmt {
                expr: expr_to_irval(expr),
            },

            Stmt::Let { name, value, .. } => IROp::Assign {
                name,
                value: expr_to_irval(value),
            },

            Stmt::Return { value } => IROp::Return {
                value: value.map(expr_to_irval),
            },

            Stmt::If { .. } | Stmt::While { .. } => IROp::ControlFlow,

            Stmt::Function { .. } => IROp::ControlFlow,

            Stmt::Block { .. } => IROp::Block { body: vec![] },

            _ => todo!(),
        }
    }
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

impl IR {
    pub fn new_from_ops(ops: Vec<IROp>) -> Self {
        let nodes = ops.clone();
        Self {
            raw: String::new(),
            nodes,
            ops,
            symbols: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
            modules: Vec::new(),
            functions: Vec::new(),
        }
    }
}

impl IR {
    pub fn with_stage(mut self, stage: &str) -> Self {
        self.metadata.insert("stage".to_string(), stage.to_string());
        self
    }
}
