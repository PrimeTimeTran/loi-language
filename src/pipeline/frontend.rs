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
// use crate::pipeline::{Metadata, provider::PipelineProvider, stage::Stage};
use crate::pipeline::{CompileError, Metadata, Pipeline, provider::PipelineProvider, stage::Stage};
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

// impl Pipeline for FrontendPipeline {
//     fn name(&self) -> &str {
//         &self.metadata.name
//     }

//     fn compile(&self) -> Result<(), CompileError> {
//         let config_guard = self.config.read().unwrap();
//         println!("Frontend compiling in: {:?}", config_guard.root);
//         Ok(())
//     }
// }

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
        println!("▶ FRONTEND: start compilation");

        // 1. Read state
        println!("▶ FRONTEND: acquiring state lock");
        let state = self.state.read().map_err(|e| {
            println!("❌ FRONTEND: state lock failed: {:?}", e);

            let mut ds = DiagnosticStore::default();
            ds.emit(Diagnostic::error("state lock failed", Span::default()));
            ds
        })?;

        println!("✔ FRONTEND: state lock acquired");

        let source = state.source.as_ref().ok_or_else(|| {
            println!("❌ FRONTEND: no source in state");

            let mut ds = DiagnosticStore::default();
            ds.emit(Diagnostic::error("No source code loaded", Span::default()));
            ds
        })?;

        println!("✔ FRONTEND: source loaded ({} chars)", source.len());

        // 2. Lexer
        println!("▶ FRONTEND: acquiring lexer lock");
        let mut lexer_guard = self.lexer.write().map_err(|e| {
            println!("❌ FRONTEND: lexer lock failed: {:?}", e);

            let mut ds = DiagnosticStore::default();
            ds.emit(Diagnostic::error("lexer lock failed", Span::default()));
            ds
        })?;

        println!("✔ FRONTEND: lexer lock acquired");

        let tokens = lexer_guard.lex(source).map_err(|e| {
            println!("❌ FRONTEND: lexing failed: {:?}", e);

            let mut ds = DiagnosticStore::default();
            ds.emit(Diagnostic::error("lexer error", Span::default()));
            ds
        })?;

        println!("✔ FRONTEND: lexed {:?} tokens", tokens);

        drop(lexer_guard);
        println!("✔ FRONTEND: lexer released");

        // 3. Diagnostics
        println!("▶ FRONTEND: acquiring diagnostics lock");
        let mut diag_guard = self.context.diagnostics.write().map_err(|e| {
            println!("❌ FRONTEND: diagnostics lock failed: {:?}", e);

            let mut ds = DiagnosticStore::default();
            ds.emit(Diagnostic::error(
                "diagnostics lock failed",
                Span::default(),
            ));
            ds
        })?;

        println!("✔ FRONTEND: diagnostics lock acquired");

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
    fn run(&self, engine: &CompileEngine) -> Result<(), CompileError> {
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
                let state = self.state.read().unwrap();
                println!("❌ ERROR PATH AST = {:?}", state.ast);

                // write diagnostics safely
                {
                    let mut global =
                        self.context.diagnostics.write().map_err(|_| {
                            CompileError::Frontend("failed to lock diagnostics".into())
                        })?;

                    for diag in diags.diagnostics {
                        global.emit(diag);
                    }
                }

                // optional debug fallback (no-op unless you want logging)
                if let Ok(state) = self.state.write() {
                    if state.ast.is_none() {
                        println!("⚠️ AST is missing after frontend failure");
                    }
                }

                Err(CompileError::Frontend("failure in AST".into()))
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
