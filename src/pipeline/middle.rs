use std::sync::{Arc, Mutex, RwLock};

use crate::backend::symbol::registry::SymbolRegistry;
use crate::compiler::config::CompileConfig;
use crate::compiler::diagnostic::DiagnosticStore;
use crate::compiler::engine::CompileEngine;
use crate::compiler::state::CompileState;
use crate::context::Context;
use crate::context::test::TestContext;
use crate::diagnostics;
use crate::frontend::ast::AST;
use crate::interface::CompileEngineProvider;
use crate::middle::ir::{IR, IROp};
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
}

// impl Pipeline for MiddlePipeline {
//     fn name(&self) -> &str {
//         &self.metadata.name
//     }

//     fn compile(&self, engine: &CompileEngine) -> Result<(), CompileError> {
//         println!("MIDDLE START");

//         let ast = { engine.state.read().unwrap().ast.clone() }
//             .ok_or_else(|| CompileError::Middle("missing AST".into()))?;

//         let ir = engine.run(ast);

//         engine.state.write().unwrap().ir_cache.current = Some(ir);

//         println!("MIDDLE END (IR written)");

//         Ok(())
//     }
// }
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
    fn run(&self, engine: &CompileEngine) -> Result<(), CompileError> {
        // let ast = { engine.state.read().unwrap().ast.clone() }
        //     .ok_or_else(|| CompileError::Middle("missing AST".into()))?;

        let ast = engine
            .state
            .read()
            .unwrap()
            .ast
            .clone()
            .ok_or_else(|| CompileError::Middle("missing AST".into()))?;

        let ir = IR {
            raw: String::new(),
            nodes: self.lower_ast(ast),
            symbols: std::collections::HashMap::new(),
            metadata: {
                let mut m = std::collections::HashMap::new();
                m.insert("stage".into(), "middle".into());
                m
            },
        };
        engine.state.write().unwrap().ir_cache.current = Some(ir);
        Ok(())
    }

    pub fn resolve_symbols(&self, _ast: &AST) {
        // future scope pass
    }

    pub fn lower_ast(&self, ast: AST) -> Vec<IROp> {
        ast.stmts.into_iter().map(IROp::from).collect()
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
