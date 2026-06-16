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
    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError> {
        let ast = {
            let state = engine.state.read().unwrap();
            state.current_ast()
        }
        .ok_or_else(|| CompileError::Middle("missing AST".into()))?;
        let ir_nodes = self.lower_ast(ast)?;
        let ir = IR::new_from_ops(ir_nodes).with_stage("middle");
        {
            let mut state = engine.state.write().unwrap();
            state.current_ir = Some(ir);
        }
        Ok(())
    }

    pub fn lower_ast(&mut self, ast: AST) -> Result<Vec<IROp>, CompileError> {
        ast.stmts
            .into_iter()
            .map(|stmt| self.lower_stmt(stmt))
            .collect::<Result<Vec<_>, _>>()
    }
    pub fn lower_expr(&self, expr: Expr) -> Result<LoweredExpr, CompileError> {
        println!("lower_expr: {:?}", expr);
        match expr {
            Expr::Number(_) | Expr::Bool(_) | Expr::String(_) | Expr::Var(_) => {
                Ok(LoweredExpr::Value(lower_atomic(expr)?))
            }

            Expr::Binary { .. } | Expr::Unary { .. } | Expr::Array(_) | Expr::Assign { .. } => {
                Ok(LoweredExpr::Op(lower_expr_as_op(expr)?))
            }

            _ => Err(CompileError::Middle("unsupported complex expr".into())),
        }
    }
    pub fn lower_expr_stmt(&mut self, expr: Expr) -> Result<IROp, CompileError> {
        println!("lower_expr_stmt: {:?}", expr);
        match expr {
            Expr::Assign { left, right, op } => self.handle_assignment(*left, *right, op, None),

            other => {
                let lowered = self.lower_expr(other)?;

                match lowered {
                    LoweredExpr::Value(v) => Ok(IROp::Expr { value: v }),

                    LoweredExpr::Op(_) => {
                        let tmp = self.emit_temp();
                        Ok(IROp::Expr {
                            value: IRVal::Var(tmp),
                        })
                    }
                }
            }
        }
    }
    pub fn lower_stmt(&mut self, stmt: Stmt) -> Result<IROp, CompileError> {
        println!("lower_stmt: {:?}", stmt);
        match stmt {
            Stmt::ExprStmt { expr } => self.lower_expr_stmt(expr),

            Stmt::Print { expr } => match self.lower_expr(expr)? {
                LoweredExpr::Value(v) => Ok(IROp::Print { value: v }),

                LoweredExpr::Op(op) => {
                    let temp = self.emit_temp();

                    Ok(IROp::Print {
                        value: IRVal::Var(temp),
                    })
                }
            },

            Stmt::Let { name, value, .. } => match self.lower_expr(value)? {
                LoweredExpr::Value(v) => Ok(IROp::Declare {
                    name,
                    value: v,
                    mutable: false,
                    dynamic: false,
                }),

                LoweredExpr::Op(op) => {
                    let ir = self.lower_op(op)?;

                    match ir {
                        IROp::Assign { name: _, value } => Ok(IROp::Declare {
                            name,
                            value,
                            mutable: false,
                            dynamic: false,
                        }),

                        _ => Err(CompileError::Middle("unsupported op in let".into())),
                    }
                }
            },

            _ => Err(CompileError::Middle(format!(
                "Unsupported statement: {:?}",
                stmt
            ))),
        }
    }
    fn lower_op(&mut self, op: IROp) -> Result<IROp, CompileError> {
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
    fn expr_to_irval_from_value(&self, val: IRVal) -> Result<IRVal, CompileError> {
        Ok(val)
    }
    fn handle_assignment(
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
    fn expr_to_irval(&self, expr: Expr) -> Result<IRVal, CompileError> {
        match self.lower_expr(expr)? {
            LoweredExpr::Value(v) => Ok(v),
            LoweredExpr::Op(_) => {
                let tmp = self.emit_temp();
                Ok(IRVal::Var(tmp))
            }
        }
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    pub fn resolve_symbols(&self, _ast: &AST) {
        // future scope pass
    }
    fn emit_temp(&self) -> String {
        let id = self
            .temp_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("__t{}", id)
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

pub fn lower_expr_as_value(expr: Expr) -> Result<IRVal, CompileError> {
    match expr {
        Expr::Number(_) | Expr::Bool(_) | Expr::String(_) | Expr::Var(_) => lower_atomic(expr),

        _ => Err(CompileError::Middle(
            "non-atomic expression cannot be lowered to IRVal".into(),
        )),
    }
}

pub fn lower_expr_as_op(expr: Expr) -> Result<IROp, CompileError> {
    match expr {
        Expr::Binary { left, op, right } => {
            let l = lower_expr_as_value(*left)?;
            let r = lower_expr_as_value(*right)?;

            Ok(IROp::Binary {
                left: l,
                op,
                right: r,
            })
        }

        Expr::Unary { op, expr } => {
            let v = lower_expr_as_value(*expr)?;

            Ok(IROp::Unary { op, value: v })
        }

        Expr::Assign { left, right, .. } => {
            let value = lower_expr_as_value(*right)?;

            match *left {
                Expr::Var(name) => Ok(IROp::Assign { name, value }),

                _ => Err(CompileError::Middle("invalid assignment target".into())),
            }
        }
        Expr::Array(items) => {
            let vals = items
                .into_iter()
                .map(lower_expr_as_value)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(IROp::Array { values: vals })
        }

        _ => Err(CompileError::Middle("unsupported complex expr".into())),
    }
}
