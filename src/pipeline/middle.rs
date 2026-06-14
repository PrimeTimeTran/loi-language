use std::sync::{Arc, RwLock};

use crate::backend::symbol::registry::SymbolRegistry;
use crate::compiler::config::CompileConfig;
use crate::compiler::diagnostic::DiagnosticStore;
use crate::compiler::state::CompileState;
use crate::context::Context;
use crate::context::test::TestContext;
use crate::diagnostics;
use crate::frontend::ast::AST;
use crate::interface::CompileEngineProvider;
use crate::middle::ir::{IR, IROp};
use crate::pipeline::Metadata;

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
            metadata: Metadata {
                name: name.to_string(),
                version: "1.0.0".to_string(),
            },
            context,
            config,
            state,
            ir_config: IRConfig::default(),
            features: MiddleFeatures::default(),
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

impl MiddlePipeline {
    pub fn run(&mut self, ast: AST) -> IR {
        let mut ir = IR::default();
        self.resolve_symbols(&ast);
        ir.nodes = self.lower_ast(ast);
        ir.metadata.insert("stage".into(), "middle".into());
        ir
    }

    pub fn resolve_symbols(&mut self, ast: &AST) {
        // future: scope building, imports, exports
    }

    pub fn lower_ast(&self, ast: AST) -> Vec<IROp> {
        ast.stmts.into_iter().map(|stmt| IROp::from(stmt)).collect()
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
