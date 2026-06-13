use crate::compiler::diagnostic::{Diagnostic, DiagnosticStore, Logger};
use crate::frontend::ast::{AST, Stmt};
use crate::frontend::lexer::Lexer;
use crate::frontend::parser::Parser;
use crate::frontend::token::Token;

/// FRONTEND PIPELINE
/// Responsible for turning raw source code into a typed AST.
///
/// This is where:
/// - lexing happens
/// - parsing happens
/// - early syntax validation happens
/// - basic diagnostics are generated
pub struct FrontendPipeline {
    pub lexer: Lexer,
    pub parser: Parser,
    pub diagnostics: DiagnosticStore,
    pub features: FrontendFeatures,
}

impl Default for FrontendPipeline {
    fn default() -> Self {
        Self {
            lexer: Lexer::default(),
            parser: Parser::default(),
            diagnostics: DiagnosticStore::default(),
            features: FrontendFeatures::default(),
        }
    }
}

#[derive(Default)]
pub struct FrontendFeatures {
    pub enable_macros: bool,
    pub enable_jsx_like_blocks: bool,
    pub strict_mode: bool,
}

impl FrontendPipeline {
    pub fn run(&mut self, source: &str) -> AST {
        let mut stream = match self.lexer.lex(source) {
            Ok(s) => s,
            Err(_) => {
                println!("Error in Lexer");
                return AST::default();
            }
        };

        println!("FIRST TOKEN: {:?}", stream.peek());
        println!("LEXER DONE");
        self.parser
            .parse(&mut stream, &mut self.diagnostics)
            .unwrap_or_default()
    }
}
