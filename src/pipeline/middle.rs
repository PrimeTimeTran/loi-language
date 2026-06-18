use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    backend::symbol::registry::SymbolRegistry,
    compiler::{
        config::CompileConfig, context::Context, diagnostic::DiagnosticStore,
        engine::CompileEngine, state::CompileState,
    },
    context::test::TestContext,
    diagnostics,
    frontend::ast::{AST, AssignOp, Expr, Stmt},
    interface::CompileEngineProvider,
    middle::{
        ir::{IR, IROp, TypedExpr},
        types::{IRVal, LoweredExpr, Span, Type},
    },
    pipeline::{
        CompileError, Metadata, Pipeline,
        stage::{LoweringStage, Stage},
    },
};

/// MIDDLE PIPELINE
/// Converts AST → IR and performs semantic analysis.
///
/// This is where:
/// - IR construction
/// - symbol resolution
/// - type checking (future)
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
    pub passes: Vec<Box<dyn Stage>>,
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
            passes: Vec::new(),
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
    pub fn add_pass(mut self, pass: Box<dyn Stage>) -> Self {
        self.passes.push(pass);
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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
        for pass in &mut self.passes {
            if let Some(lowering_pass) = pass.as_any_mut().downcast_mut::<LoweringStage>() {
                // Pass the fields individually
                lowering_pass.run_with_pipeline(engine, &mut self.symbols, &self.temp_counter)?;
            } else {
                pass.run(engine)?;
            }
        }
        Ok(())
    }
    pub fn lower_ast_to_ir(&mut self, ast: AST) -> Result<Vec<IROp>, CompileError> {
        ast.stmts
            .into_iter()
            .map(|stmt| self.lower_stmt(stmt))
            .collect::<Result<Vec<_>, _>>()
    }
    pub fn lower_stmt(&mut self, stmt: Stmt) -> Result<IROp, CompileError> {
        IRGenerator::lower_stmt(stmt, &mut self.symbols, &self.temp_counter)
    }
    pub fn lower_expr(&mut self, expr: Expr) -> Result<LoweredExpr, CompileError> {
        IRGenerator::lower_expr(expr, &mut self.symbols, &self.temp_counter)
    }

    pub fn handle_assignment(
        &mut self,
        left: Expr,
        right: Expr,
        op: AssignOp,
        span: Option<Span>,
    ) -> Result<IROp, CompileError> {
        IRGenerator::handle_assignment(left, right, op, span, &mut self.symbols, &self.temp_counter)
    }

    pub fn lower_op(&mut self, op: IROp) -> Result<IROp, CompileError> {
        IRGenerator::lower_op(op, &mut self.symbols)
    }

    pub fn expr_to_irval(&mut self, expr: Expr) -> Result<IRVal, CompileError> {
        IRGenerator::expr_to_irval(expr, &mut self.symbols, &self.temp_counter)
    }

    pub fn expr_to_irval_from_value(&mut self, val: IRVal) -> Result<IRVal, CompileError> {
        IRGenerator::expr_to_irval_from_value(val, &mut self.symbols)
    }

    pub fn emit_temp(&self) -> String {
        IRGenerator::emit_temp(&self.temp_counter)
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }
    pub fn resolve_symbols(&self, _ast: &AST) {}
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

impl Default for MiddlePipeline {
    fn default() -> Self {
        let context = Arc::new(Context::new());
        let config = Arc::new(RwLock::new(CompileConfig::default()));
        let state = Arc::new(RwLock::new(CompileState::default()));
        Self::new(context, config, state)
    }
}

#[derive(Debug, Clone)]
pub struct MiddleLoweringLogic;

impl MiddleLoweringLogic {
    pub fn execute(
        &self,
        engine: &CompileEngine,
        symbols: &mut HashMap<String, SymbolInfo>,
        counter: &std::sync::atomic::AtomicUsize,
    ) -> Result<(), CompileError> {
        let ast = {
            let state = engine.state.read().unwrap();
            state.current_ast()
        }
        .ok_or_else(|| CompileError::Middle("AST missing".into()))?;
        let ir_nodes = IRGenerator::lower_ast_to_ir(ast, symbols, counter)?;
        let ir = IR::new_from_ops(ir_nodes).with_stage("middle");
        engine.state.write().unwrap().current_ir = Some(ir);
        Ok(())
    }
}

pub struct IRGenerator;

impl IRGenerator {
    pub fn lower_ast_to_ir(
        ast: AST,
        symbols: &mut HashMap<String, SymbolInfo>,
        counter: &std::sync::atomic::AtomicUsize,
    ) -> Result<Vec<IROp>, CompileError> {
        ast.stmts
            .into_iter()
            .map(|stmt| Self::lower_stmt(stmt, symbols, counter))
            .collect::<Result<Vec<_>, _>>()
    }
    pub fn lower_stmt(
        stmt: Stmt,
        symbols: &mut HashMap<String, SymbolInfo>,
        counter: &std::sync::atomic::AtomicUsize,
    ) -> Result<IROp, CompileError> {
        println!("lower_stmt: {:?}", stmt);

        match stmt {
            Stmt::ExprStmt { expr } => Self::lower_expr_stmt(expr, symbols, counter),

            Stmt::Print { expr } => {
                let lowered = Self::lower_expr(expr, symbols, counter)?;
                match lowered {
                    LoweredExpr::Value(v) => Ok(IROp::Print { value: v }),
                    LoweredExpr::Op(op) => Ok(op),
                }
            }

            Stmt::Let { name, value, .. } => {
                let lowered = Self::lower_expr(value, symbols, counter)?;

                match lowered {
                    LoweredExpr::Value(v) => Ok(IROp::Declare {
                        name,
                        value: v,
                        mutable: false,
                        dynamic: false,
                    }),

                    LoweredExpr::Op(op) => match op {
                        // Manually extract the value from the Assign variant
                        IROp::Assign {
                            value: assigned_val,
                            ..
                        } => Ok(IROp::Declare {
                            name,
                            value: assigned_val,
                            mutable: false,
                            dynamic: false,
                        }),

                        IROp::Binary { .. } => {
                            let temp = Self::emit_temp(counter);
                            Ok(IROp::Declare {
                                name,
                                value: IRVal::Var(temp),
                                mutable: false,
                                dynamic: false,
                            })
                        }

                        IROp::Expr { value: expr_val } => Ok(IROp::Declare {
                            name,
                            value: expr_val,
                            mutable: false,
                            dynamic: false,
                        }),

                        other => Err(CompileError::Middle(format!(
                            "unsupported op in let: {:?}",
                            other
                        ))),
                    },
                }
            }
            _ => Err(CompileError::Middle(format!(
                "Unsupported statement: {:?}",
                stmt
            ))),
        }
    }

    pub fn lower_expr_stmt(
        expr: Expr,
        symbols: &mut HashMap<String, SymbolInfo>,
        counter: &std::sync::atomic::AtomicUsize,
    ) -> Result<IROp, CompileError> {
        println!("lower_expr_stmt: {:?}", expr);

        match expr {
            Expr::Assign { left, right, op } => {
                // Pass symbols and counter down to handle_assignment
                Self::handle_assignment(*left, *right, op, None, symbols, counter)
            }
            other => {
                let lowered = Self::lower_expr(other, symbols, counter)?;
                match lowered {
                    LoweredExpr::Value(v) => Ok(IROp::Expr { value: v }),
                    LoweredExpr::Op(op) => Ok(op),
                }
            }
        }
    }
    pub fn lower_expr(
        expr: Expr,
        symbols: &mut HashMap<String, SymbolInfo>,
        counter: &std::sync::atomic::AtomicUsize,
    ) -> Result<LoweredExpr, CompileError> {
        println!("lower_expr: {:?}", expr);

        match expr {
            Expr::Number(_) | Expr::Bool(_) | Expr::String(_) | Expr::Var(_) => {
                Ok(LoweredExpr::Value(match expr {
                    Expr::Number(n) => Ok(IRVal::Number(n)),
                    Expr::Bool(b) => Ok(IRVal::Bool(b)),
                    Expr::String(s) => Ok(IRVal::Str(s)),
                    Expr::Var(v) => Ok(IRVal::Var(v)),
                    _ => Err(CompileError::Middle("not atomic".into())),
                }?))
            }

            Expr::Binary { left, op, right } => {
                // Threading state through recursive calls
                let l = Self::lower_expr(*left, symbols, counter)?;
                let r = Self::lower_expr(*right, symbols, counter)?;

                match (l, r) {
                    (LoweredExpr::Value(_lv), LoweredExpr::Value(_rv)) => {
                        // Using stateful counter passed in
                        let tmp = Self::emit_temp(counter);
                        Ok(LoweredExpr::Value(IRVal::Var(tmp)))
                    }
                    _ => Err(CompileError::Middle("invalid binary lowering".into())),
                }
            }
            _ => Err(CompileError::Middle("unsupported complex expr".into())),
        }
    }
    pub fn handle_assignment(
        left: Expr,
        right: Expr,
        op: AssignOp,
        span: Option<Span>,
        symbols: &mut HashMap<String, SymbolInfo>,
        counter: &std::sync::atomic::AtomicUsize,
    ) -> Result<IROp, CompileError> {
        let name = match left {
            Expr::Var(name) => name,
            _ => return Err(CompileError::Middle("invalid assignment target".into())),
        };

        match op {
            AssignOp::Assign => {
                // Change self.expr_to_irval(...) to Self::expr_to_irval(..., symbols, counter)
                let value = Self::expr_to_irval(right, symbols, counter)?;

                if symbols.contains_key(&name) {
                    Ok(IROp::Assign { name, value })
                } else {
                    symbols.insert(
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
                let value = Self::expr_to_irval(right, symbols, counter)?;

                symbols.insert(
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
                let value = Self::expr_to_irval(right, symbols, counter)?;

                symbols.insert(
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
    pub fn lower_op(
        op: IROp,
        symbols: &mut HashMap<String, SymbolInfo>,
    ) -> Result<IROp, CompileError> {
        match op {
            IROp::Assign { name, value } => {
                let value = Self::expr_to_irval_from_value(value, symbols)?;
                symbols.insert(
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
                let value = Self::expr_to_irval_from_value(value, symbols)?;
                symbols.insert(
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

    pub fn expr_to_irval_from_value(
        val: IRVal,
        _symbols: &mut HashMap<String, SymbolInfo>,
    ) -> Result<IRVal, CompileError> {
        Ok(val)
    }

    pub fn emit_temp(counter: &std::sync::atomic::AtomicUsize) -> String {
        let id = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("__t{}", id)
    }

    pub fn expr_to_irval(
        expr: Expr,
        symbols: &mut HashMap<String, SymbolInfo>,
        counter: &std::sync::atomic::AtomicUsize,
    ) -> Result<IRVal, CompileError> {
        // Use Self::lower_expr instead of self.lower_expr
        match Self::lower_expr(expr, symbols, counter)? {
            LoweredExpr::Value(v) => Ok(v),
            LoweredExpr::Op(_) => {
                // Use Self::emit_temp instead of self.emit_temp
                let tmp = Self::emit_temp(counter);
                Ok(IRVal::Var(tmp))
            }
        }
    }
}
