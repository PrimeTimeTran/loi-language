use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::backend::symbol::registry::SymbolRegistry;
use crate::compiler::config::CompileConfig;
use crate::compiler::diagnostic::DiagnosticStore;
use crate::compiler::engine::CompileEngine;
use crate::compiler::state::CompileState;
use crate::context::Context;
use crate::context::test::TestContext;
use crate::diagnostics;
use crate::frontend::ast::{AST, AssignOp, Expr, Stmt};
use crate::interface::CompileEngineProvider;
use crate::middle::ir::{IR, IROp, TypedExpr};
use crate::middle::types::{IRVal, LoweredExpr, Span, Type};
use crate::pipeline::{CompileError, Metadata, Pipeline};

/// MIDDLE PIPELINE
/// Converts AST → IR and performs semantic analysis.
///
/// This is where:
/// - type checking (future)
/// - symbol resolution
/// - IR construction
/// - macro expansion (future)
#[derive(Debug)]
pub struct MiddlePipeline {
    pub metadata: Metadata,
    pub context: Arc<Context>,
    pub config: Arc<RwLock<CompileConfig>>,
    pub state: Arc<RwLock<CompileState>>,
    pub ir_config: IRConfig,
    pub features: MiddleFeatures,
    pub temp_counter: std::sync::atomic::AtomicUsize,
    pub symbols: HashMap<String, SymbolInfo>,
}

impl MiddlePipeline {
    pub fn new(
        context: Arc<Context>,
        config: Arc<RwLock<CompileConfig>>,
        state: Arc<RwLock<CompileState>>,
    ) -> Self {
        Self::with_name("MiddlePipeline", context, config, state)
    }
    pub fn with_name(
        name: &str,
        context: Arc<Context>,
        config: Arc<RwLock<CompileConfig>>,
        state: Arc<RwLock<CompileState>>,
    ) -> Self {
        Self {
            symbols: HashMap::new(),
            metadata: Metadata {
                name: name.to_string(),
                version: "1.0.0".to_string(),
            },
            context,
            config,
            state,
            ir_config: IRConfig::default(),
            features: MiddleFeatures::default(),
            temp_counter: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    pub fn with_ir_config(mut self, config: IRConfig) -> Self {
        self.ir_config = config;
        self
    }

    pub fn with_features(mut self, features: MiddleFeatures) -> Self {
        self.features = features;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolInfo {
    is_mutable: bool,
    is_initialized: bool,
    declared_at: Option<Span>,
}

impl MiddlePipeline {
    // ===============================================================
    // MIDDLE PIPELINE (AST → IR LOWERING)
    // ===============================================================
    //
    // PURPOSE:
    // Converts a fully parsed AST into a linear IR (IROp stream).
    //
    // OUTPUT:
    // Vec<IROp> where each operation is already "flat" and safe
    // for backend codegen (LLVM or other backends).
    //
    // KEY IDEA:
    // - Expressions are NOT directly executed
    // - Complex expressions become temporaries (__t0, __t1, ...)
    // - Statements become IR operations (Declare / Assign / Print / Expr)
    //
    // ===============================================================

    // ===============================================================
    // 1. PIPELINE ENTRY POINT (AST → IR MODULE)
    // ===============================================================
    //
    // Takes AST from frontend state and produces IR module.
    //
    // Flow:
    // AST → Vec<Stmt> → Vec<IROp> → IR
    //
    // This is the ONLY function LLVM should ever call upstream.
    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError> {
        let ast = {
            let state = engine.state.read().unwrap();
            state.current_ast()
        }
        .ok_or_else(|| CompileError::Middle("missing AST".into()))?;
        let ir_nodes = self.lower_ast_to_ir(ast)?;
        let ir = IR::new_from_ops(ir_nodes).with_stage("middle");
        {
            let mut state = engine.state.write().unwrap();
            state.current_ir = Some(ir);
        }
        Ok(())
    }

    // ===============================================================
    // 2. TOP LEVEL LOWERING (AST → IR OPS)
    // ===============================================================
    //
    // Converts whole program into IR instruction stream.
    //
    // Flow:
    // AST { stmts } → lower_stmt(stmt)* → Vec<IROp>
    //
    // Each statement becomes one or more IR ops.
    pub fn lower_ast_to_ir(&mut self, ast: AST) -> Result<Vec<IROp>, CompileError> {
        ast.stmts
            .into_iter()
            .map(|stmt| self.lower_stmt(stmt))
            .collect::<Result<Vec<_>, _>>()
    }

    // ===============================================================
    // 3. STATEMENT LOWERING (Stmt → IROp)
    // ===============================================================
    //
    // This is the main dispatch layer.
    //
    // Responsibilities:
    // - Decide IR shape per statement
    // - Ensure expressions are lowered first
    // - Never leaves AST expressions unprocessed
    //
    // Examples:
    // let x = 1 + 2      → Declare + temp
    // print x + 1        → Print + temp
    // x = 3              → Assign
    pub fn lower_stmt(&mut self, stmt: Stmt) -> Result<IROp, CompileError> {
        println!("lower_stmt: {:?}", stmt);

        match stmt {
            Stmt::ExprStmt { expr } => self.lower_expr_stmt(expr),
            Stmt::Print { expr } => {
                let lowered = self.lower_expr(expr)?;
                match lowered {
                    LoweredExpr::Value(v) => Ok(IROp::Print { value: v }),

                    LoweredExpr::Op(op) => {
                        // IMPORTANT: do NOT fabricate temps here
                        Ok(op)
                    }
                }
            }
            Stmt::Let { name, value, .. } => {
                let lowered = self.lower_expr(value)?;

                match lowered {
                    LoweredExpr::Value(v) => Ok(IROp::Declare {
                        name,
                        value: v,
                        mutable: false,
                        dynamic: false,
                    }),

                    LoweredExpr::Op(op) => {
                        match op {
                            // already a valid assignment form
                            IROp::Assign { name: _, value } => Ok(IROp::Declare {
                                name,
                                value,
                                mutable: false,
                                dynamic: false,
                            }),

                            IROp::Binary { left, op, right } => {
                                let temp = self.emit_temp();

                                // convert binary into a binding
                                Ok(IROp::Declare {
                                    name,
                                    value: IRVal::Var(temp),
                                    mutable: false,
                                    dynamic: false,
                                })
                            }

                            // fallback safety
                            IROp::Expr { value } => Ok(IROp::Declare {
                                name,
                                value,
                                mutable: false,
                                dynamic: false,
                            }),

                            other => Err(CompileError::Middle(format!(
                                "unsupported op in let: {:?}",
                                other
                            ))),
                        }
                    }
                }
            }

            _ => Err(CompileError::Middle(format!(
                "Unsupported statement: {:?}",
                stmt
            ))),
        }
    }

    // ===============================================================
    // 4. EXPRESSION STATEMENT LOWERING
    // ===============================================================
    //
    // Handles standalone expressions used as statements.
    //
    // Flow:
    // Expr → LoweredExpr → IROp::Expr or IROp::Assign/etc
    pub fn lower_expr_stmt(&mut self, expr: Expr) -> Result<IROp, CompileError> {
        println!("lower_expr_stmt: {:?}", expr);

        match expr {
            Expr::Assign { left, right, op } => self.handle_assignment(*left, *right, op, None),
            other => {
                let lowered = self.lower_expr(other)?;
                match lowered {
                    LoweredExpr::Value(v) => Ok(IROp::Expr { value: v }),
                    LoweredExpr::Op(op) => Ok(op),
                }
            }
        }
    }

    // ===============================================================
    // 5. EXPRESSION LOWERING (CORE LOGIC)
    // ===============================================================
    //
    // Converts AST expressions into either:
    //
    // A) LoweredExpr::Value   → direct IR value (constant/var)
    // B) LoweredExpr::Op      → requires IR operation + temp
    //
    // IMPORTANT RULE:
    // This function MUST NOT produce IR ops directly.
    // It only decides VALUE vs OP.
    pub fn lower_expr(&self, expr: Expr) -> Result<LoweredExpr, CompileError> {
        println!("lower_expr: {:?}", expr);
        match expr {
            Expr::Number(_) | Expr::Bool(_) | Expr::String(_) | Expr::Var(_) => {
                Ok(LoweredExpr::Value(lower_atomic(expr)?))
            }

            Expr::Binary { left, op, right } => {
                let l = self.lower_expr(*left)?;
                let r = self.lower_expr(*right)?;

                match (l, r) {
                    (LoweredExpr::Value(lv), LoweredExpr::Value(rv)) => {
                        let tmp = self.emit_temp();

                        Ok(LoweredExpr::Value(IRVal::Var(tmp)))
                    }

                    _ => Err(CompileError::Middle("invalid binary lowering".into())),
                }
            }
            _ => Err(CompileError::Middle("unsupported complex expr".into())),
        }
    }

    // ===============================================================
    // 6. ASSIGNMENT HANDLING (SPECIAL CASE)
    // ===============================================================
    //
    // Handles:
    // - let x = ...
    // - x = ...
    // - x := dynamic / immutable forms
    pub fn handle_assignment(
        &mut self,
        left: Expr,
        right: Expr,
        op: AssignOp,
        span: Option<Span>,
    ) -> Result<IROp, CompileError> {
        let name = match left {
            Expr::Var(name) => name,
            _ => return Err(CompileError::Middle("invalid assignment target".into())),
        };

        match op {
            AssignOp::Assign => {
                let value = self.expr_to_irval(right)?;
                if self.symbols.contains_key(&name) {
                    Ok(IROp::Assign { name, value })
                } else {
                    self.symbols.insert(
                        name.clone(),
                        SymbolInfo {
                            is_mutable: true,
                            is_initialized: true,
                            declared_at: span,
                        },
                    );

                    Ok(IROp::Declare {
                        name,
                        value,
                        mutable: true,
                        dynamic: false,
                    })
                }
            }

            AssignOp::Immutable => {
                let value = self.expr_to_irval(right)?;

                self.symbols.insert(
                    name.clone(),
                    SymbolInfo {
                        is_mutable: false,
                        is_initialized: true,
                        declared_at: span,
                    },
                );

                Ok(IROp::Declare {
                    name,
                    value,
                    mutable: false,
                    dynamic: false,
                })
            }

            AssignOp::Dynamic => {
                let value = self.expr_to_irval(right)?;

                self.symbols.insert(
                    name.clone(),
                    SymbolInfo {
                        is_mutable: true,
                        is_initialized: true,
                        declared_at: span,
                    },
                );

                Ok(IROp::Declare {
                    name,
                    value,
                    mutable: true,
                    dynamic: true,
                })
            }
        }
    }
    // ===============================================================
    // 7. IR OP POST-PROCESSING (OPTIONAL PASS)
    // ===============================================================
    //
    // Takes already-created IR ops and:
    // - resolves symbol table info
    // - normalizes declare/assign
    // - applies final IR rules
    pub fn lower_op(&mut self, op: IROp) -> Result<IROp, CompileError> {
        match op {
            IROp::Assign { name, value } => {
                let value = self.expr_to_irval_from_value(value)?;
                self.symbols.insert(
                    name.clone(),
                    SymbolInfo {
                        is_mutable: true,
                        is_initialized: true,
                        declared_at: None,
                    },
                );

                Ok(IROp::Assign { name, value })
            }

            IROp::Declare {
                name,
                value,
                mutable,
                dynamic,
            } => {
                let value = self.expr_to_irval_from_value(value)?;

                self.symbols.insert(
                    name.clone(),
                    SymbolInfo {
                        is_mutable: mutable,
                        is_initialized: true,
                        declared_at: None,
                    },
                );

                Ok(IROp::Declare {
                    name,
                    value,
                    mutable,
                    dynamic,
                })
            }

            other => Ok(other),
        }
    }
    // ===============================================================
    // 8. VALUE CONVERSION HELPERS
    // ===============================================================
    //
    // Used when IRVal is already known and just needs validation.
    //
    // DOES NOT lower expressions.
    pub fn expr_to_irval_from_value(&self, val: IRVal) -> Result<IRVal, CompileError> {
        Ok(val)
    }

    // ===============================================================
    // 9. TEMPORARY GENERATION
    // ===============================================================
    //
    // Produces SSA-like temps:
    //
    // __t0, __t1, __t2 ...
    //
    // Used ONLY when lowering complex expressions.
    pub fn emit_temp(&self) -> String {
        let id = self
            .temp_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("__t{}", id)
    }

    // Converts full expression → IRVal (only safe for simple cases)
    pub fn expr_to_irval(&self, expr: Expr) -> Result<IRVal, CompileError> {
        match self.lower_expr(expr)? {
            LoweredExpr::Value(v) => Ok(v),
            LoweredExpr::Op(_) => {
                let tmp = self.emit_temp();
                Ok(IRVal::Var(tmp))
            }
        }
    }

    // ===============================================================
    // 10. SYMBOL / META HELPERS
    // ===============================================================
    //
    // Used for bookkeeping, not lowering.
    pub fn name(&self) -> &str {
        &self.metadata.name
    }
    pub fn resolve_symbols(&self, _ast: &AST) {
        // future scope pass
    }
}

#[derive(Default, Debug)]
pub struct MiddleFeatures {
    pub enable_type_checking: bool,
    pub enable_macro_expansion: bool,
    pub enable_dead_code_analysis: bool,
}
#[derive(Default, Debug)]
pub struct IRConfig {
    pub preserve_raw_blocks: bool,
    pub optimize_early: bool,
}

// #[cfg(test)]
impl Default for MiddlePipeline {
    fn default() -> Self {
        let context = Arc::new(Context::new());
        let config = Arc::new(RwLock::new(CompileConfig::default()));
        let state = Arc::new(RwLock::new(CompileState::default()));
        Self::new(context, config, state)
    }
}

pub fn lower_atomic(expr: Expr) -> Result<IRVal, CompileError> {
    match expr {
        Expr::Number(n) => Ok(IRVal::Number(n)),
        Expr::Bool(b) => Ok(IRVal::Bool(b)),
        Expr::String(s) => Ok(IRVal::Str(s)),
        Expr::Var(v) => Ok(IRVal::Var(v)),

        _ => Err(CompileError::Middle("not atomic".into())),
    }
}
