use std::cell::RefCell;
use std::sync::{Arc, RwLock};

use crate::compiler::config::CompileConfig;
use crate::compiler::diagnostic::{Diagnostic, DiagnosticStore, Logger};
use crate::compiler::engine::CompileEngine;
use crate::compiler::state::CompileState;
use crate::context::test::TestContext;
use crate::context::{Context, Kernel};
use crate::frontend::ast::{AST, Stmt};
use crate::frontend::lexer::Lexer;
use crate::frontend::parser::Parser;
use crate::frontend::token::Token;
use crate::interface::CompileEngineProvider;
use crate::pipeline::provider::PipelineProvider;
use crate::pipeline::stage::Stage;
use crate::pipeline::{Metadata, Pipeline};
use crate::test_utils::TestEnv;

/// FRONTEND PIPELINE
/// Responsible for turning raw source code into a typed AST.
///
/// This is where:
/// - lexing happens
/// - parsing happens
/// - early syntax validation happens
/// - basic diagnostics are generated

pub struct FrontendPipeline {
    pub metadata: Metadata,
    pub context: Arc<Context>,
    pub config: Arc<RwLock<CompileConfig>>,
    pub state: Arc<RwLock<CompileState>>,
    pub lexer: RefCell<Lexer>,
    pub parser: RefCell<Parser>,
    pub features: FrontendFeatures,
}

impl Pipeline for FrontendPipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }
    fn compile(&self) {
        // 1. Acquire the read lock on the entire CompileConfig struct
        // config_guard acts like a reference to CompileConfig (&CompileConfig)
        let config_guard = self.config.read().unwrap();

        // 2. Access the field directly.
        // No second .read() is needed because root is just a PathBuf.
        println!("Frontend compiling in: {:?}", config_guard.root);
    }
}
impl PipelineProvider for FrontendPipeline {
    type Pipeline = FrontendPipeline;
    fn create(&self, env: &TestEnv) -> Self::Pipeline {
        FrontendPipeline::new(env.context.clone(), env.config.clone(), env.state.clone())
    }
}

impl FrontendPipeline {
    pub fn new(
        context: Arc<Context>,
        config: Arc<RwLock<CompileConfig>>,
        state: Arc<RwLock<CompileState>>,
    ) -> Self {
        Self::with_name("FrontendPipeline", context, config, state)
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
            // Explicitly initialize the sub-components
            lexer: RefCell::new(Lexer::default()),
            parser: RefCell::new(Parser::default()),
            features: FrontendFeatures::default(),
        }
    }
}

impl FrontendPipeline {
    fn perform_compilation(&self) -> Result<AST, String> {
        let state = self.state.read().map_err(|e| e.to_string())?;
        let source = state.source.as_ref().ok_or("No source code loaded")?;
        let tokens = self
            .lexer
            .borrow_mut()
            .lex(source)
            .map_err(|e| format!("Lexer error: {:?}", e))?;

        drop(state);

        let mut diag_guard = self
            .context
            .diagnostics
            .write()
            .map_err(|e| e.to_string())?;
        let ast = self
            .parser
            .borrow_mut()
            .parse(tokens, &mut *diag_guard)
            .map_err(|_| "Parser failed unexpectedly".to_string())?;

        if diag_guard.has_errors() {
            return Err("Frontend failed: Parser encountered errors".to_string());
        }

        Ok(ast)
    }
}

impl Stage for FrontendPipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn run(&self) -> Result<(), String> {
        let ast = self.perform_compilation()?;

        let mut state = self.state.write().map_err(|e| e.to_string())?;
        state.ast = Some(ast);
        Ok(())
    }
}

#[derive(Default)]
pub struct FrontendFeatures {
    pub enable_macros: bool,
    pub enable_jsx_like_blocks: bool,
    pub strict_mode: bool,
}
#[cfg(test)]
impl Default for FrontendPipeline {
    fn default() -> Self {
        let context = Arc::new(Context::new());
        let config = Arc::new(RwLock::new(CompileConfig::default()));
        let state = Arc::new(RwLock::new(CompileState::default()));
        Self::new(context, config, state)
    }
}
