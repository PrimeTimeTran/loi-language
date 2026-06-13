use crate::backend::symbol::registry::SymbolRegistry;
use crate::compiler::diagnostic::DiagnosticStore;
use crate::frontend::ast::AST;
use crate::middle::ir::{IR, IROp};

/// MIDDLE PIPELINE
/// Converts AST → IR and performs semantic analysis.
///
/// This is where:
/// - type checking (future)
/// - symbol resolution
/// - IR construction
/// - macro expansion (future)
#[derive(Default)]
pub struct MiddlePipeline {
    /// Global symbol registry (cross-file resolution)
    pub symbols: SymbolRegistry,

    /// Diagnostics from semantic analysis
    pub diagnostics: DiagnosticStore,

    /// IR generation settings
    pub ir_config: IRConfig,

    /// feature flags for semantic layer
    pub features: MiddleFeatures,
}

#[derive(Default)]
pub struct MiddleFeatures {
    pub enable_type_checking: bool,
    pub enable_macro_expansion: bool,
    pub enable_dead_code_analysis: bool,
}

#[derive(Default)]
pub struct IRConfig {
    pub preserve_raw_blocks: bool,
    pub optimize_early: bool,
}

impl MiddlePipeline {
    pub fn run(&mut self, ast: AST) -> IR {
        let mut ir = IR::default();

        // 1. semantic pass (symbols)
        self.resolve_symbols(&ast);

        // 2. AST → IR lowering
        ir.nodes = self.lower_ast(ast);

        // 3. attach metadata
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
