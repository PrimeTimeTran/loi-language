use crate::compiler::diagnostic::{Diagnostic, DiagnosticStore, Logger};
use crate::frontend::ast::{AST, Stmt};
use crate::frontend::lexer::Lexer;
use crate::frontend::parser::Parser;

/// FRONTEND PIPELINE
/// Responsible for turning raw source code into a typed AST.
///
/// This is where:
/// - lexing happens
/// - parsing happens
/// - early syntax validation happens
/// - basic diagnostics are generated
#[derive(Default)]
pub struct FrontendPipeline {
    /// Lexer configuration (future: unicode rules, strict mode, etc.)
    pub lexer: Lexer,

    /// Parser configuration (precedence rules, experimental syntax flags)
    pub parser: Parser,

    /// Diagnostics collected during parsing
    pub diagnostics: DiagnosticStore,

    /// Feature flags (experimental syntax, macros, etc.)
    pub features: FrontendFeatures,
}

#[derive(Default)]
pub struct FrontendFeatures {
    pub enable_macros: bool,
    pub enable_jsx_like_blocks: bool,
    pub strict_mode: bool,
}

impl FrontendPipeline {
    pub fn run(&mut self, source: &str) -> AST {
        let mut stream = match self.lexer.lex(source, &mut self.diagnostics) {
            Ok(s) => s,
            Err(_) => return AST::default(),
        };

        self.parser
            .parse(&mut stream, &mut self.diagnostics)
            .unwrap_or_default()
    }
}
