use std::cell::RefCell;
use std::sync::{Arc, RwLock};

use tracing::Instrument;

use crate::compiler::diagnostic::Severity;
use crate::compiler::{
    config::CompileConfig,
    diagnostic::{Diagnostic, DiagnosticStore, Logger},
    engine::CompileEngine,
    state::CompileState,
};
use crate::context::Context;
use crate::context::test::TestContext;
use crate::frontend::{
    ast::{AST, Stmt},
    parser::Parser,
    token::Token,
    types::Lexer,
};
use crate::interface::CompileEngineProvider;
use crate::middle::types::Span;
use crate::pipeline::{Metadata, Pipeline, provider::PipelineProvider, stage::Stage};
use crate::test_utils::TestEnv;

/// FRONTEND PIPELINE
/// Responsible for turning raw source code into a typed AST.
///
/// This is where:
/// - lexing happens
/// - parsing happens
/// - early syntax validation happens
/// - basic diagnostics are generated
#[derive(Debug)]
pub struct FrontendPipeline {
    pub metadata: Metadata,
    pub context: Arc<Context>,
    pub config: Arc<RwLock<CompileConfig>>,
    pub state: Arc<RwLock<CompileState>>,
    pub lexer: Arc<RwLock<Lexer>>,
    pub parser: Arc<RwLock<Parser>>,
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
            lexer: Arc::new(RwLock::new(Lexer::default())),
            parser: Arc::new(RwLock::new(Parser::default())),
            features: FrontendFeatures::default(),
        }
    }
    pub fn with_features(mut self, features: FrontendFeatures) -> Self {
        self.features = features;
        self
    }
}

impl FrontendPipeline {
    fn perform_compilation(&self) -> Result<AST, DiagnosticStore> {
        // 1. Read state
        let state = self.state.read().map_err(|e| {
            let mut ds = DiagnosticStore::default();
            let diag = Diagnostic::new(
                format!("Failed source code loaded: {}", e),
                Span::default(), // no real span here
                Severity::Error,
            )
            .with_code("ELOCK001")
            .with_note("Lexer was locked by another thread or poisoned")
            .with_suggestion("Retry compilation or ensure single-threaded access");

            ds.emit(diag);
            ds
        })?;

        let source = state.source.as_ref().ok_or_else(|| {
            let mut ds = DiagnosticStore::default();

            let diag = Diagnostic::new("No source code loaded", Span::default(), Severity::Error)
                .with_code("E1002")
                .with_note("Compiler state does not contain source input")
                .with_suggestion("Load source code before running frontend pipeline");

            ds.emit(diag);
            ds
        })?;

        let source = state.source.as_ref().ok_or_else(|| {
            let mut ds = DiagnosticStore::default();
            ds.emit(Diagnostic::error("No source code loaded", Span::default()));
            ds
        })?;
        // 🔥 DEBUG CHECKPOINT AST (force visibility)
        // {
        //     if let Ok(mut state) = self.state.write() {
        //         state.ast = Some(AST::new(vec![])); // empty AST marker
        //         println!("🔥 CHECKPOINT AST WRITTEN");
        //     }
        // }

        // 2. Lexing
        let mut lexer_guard = self.lexer.write().map_err(|e| {
            let mut ds = DiagnosticStore::default();
            let diag = Diagnostic::new(
                format!("Failed to acquire lexer lock: {}", e),
                Span::default(), // no real span here
                Severity::Error,
            )
            .with_code("ELOCK001")
            .with_note("Lexer was locked by another thread or poisoned")
            .with_suggestion("Retry compilation or ensure single-threaded access");

            ds.emit(diag);
            ds
        })?;

        let tokens = lexer_guard.lex(source).map_err(|e| {
            let mut ds = DiagnosticStore::default();
            let diag = Diagnostic::new(
                format!("Error in lexer: {:?}", e),
                Span::default(), // no real span here
                Severity::Error,
            )
            .with_code("ELOCK001")
            .with_note("Lexer was locked by another thread or poisoned")
            .with_suggestion("Retry compilation or ensure single-threaded access");
            ds.emit(diag);
            ds
        })?;

        drop(lexer_guard);

        // 3. Diagnostics + parser
        let mut diag_guard = self.context.diagnostics.write().map_err(|e| {
            let mut ds = DiagnosticStore::default();
            let diag = Diagnostic::new(
                format!("Error in lexer: {:?}", e),
                Span::default(), // no real span here
                Severity::Error,
            )
            .with_code("ELOCK001")
            .with_note("Lexer was locked by another thread or poisoned")
            .with_suggestion("Retry compilation or ensure single-threaded access");
            ds.emit(diag);
            ds
        })?;

        let mut parser_guard = self.parser.write().map_err(|e| {
            let mut ds = DiagnosticStore::default();
            let diag = Diagnostic::new(
                format!("Error in parser: {:?}", e),
                Span::default(), // no real span here
                Severity::Error,
            )
            .with_code("ELOCK002")
            .with_note("Parser was locked by another thread or poisoned")
            .with_suggestion("Retry compilation or ensure single-threaded access");
            ds.emit(diag);
            ds
        })?;

        let ast = parser_guard.parse(tokens, &mut *diag_guard).map_err(|_| {
            let mut ds = DiagnosticStore::default();
            let diag =
                Diagnostic::new(format!("Error in parser"), Span::default(), Severity::Error)
                    .with_code("ELOCK001")
                    .with_note("Lexer was locked by another thread or poisoned")
                    .with_suggestion("Retry compilation or ensure single-threaded access");
            ds.emit(diag);
            ds
        })?;

        drop(parser_guard);

        if diag_guard.has_errors() {
            return Err(diag_guard.clone());
        }

        Ok(ast)
    }
}

impl Stage for FrontendPipeline {
    fn run(&self) -> Result<(), ()> {
        let result = self.perform_compilation();
        match result {
            Ok(ast) => {
                let mut state = self.state.write().unwrap();
                state.ast = Some(ast);
                println!("✅ FINAL AST WRITTEN");
                println!("${:?}", state.ast);
                Ok(())
            }
            Err(diags) => {
                let state = self.state.write().unwrap();
                println!("❌ ERROR PATH AST = {:?}", state.ast);
                println!("${:?}", state.ast);
                let mut global = self.context.diagnostics.write().map_err(|_| ())?;

                for diag in diags.diagnostics {
                    global.emit(diag);
                }

                // 🔥 DEBUG: try to persist whatever AST exists
                if let Ok(mut state) = self.state.write() {
                    if state.ast.is_none() {
                        // optional debug fallback
                        state.ast = None;
                    }
                }

                Err(())
            }
        }
    }
    fn name(&self) -> &str {
        &self.metadata.name
    }
}
#[derive(Debug, Default)]
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
