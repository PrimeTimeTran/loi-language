use crate::common::{MockEngine, TestHarness};

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module,
    values::{FunctionValue, PointerValue},
};
use loi::{
    backend::{
        llvm::{LLVM, lower_ast_to_ir},
        symbol::registry::{Symbol, SymbolKind, SymbolRegistry},
        utter::{registry::UtterRegistry, utter::Utter},
    },
    build::build_system::BuildSystem,
    compiler::diagnostic::DiagnosticStore,
    frontend::{
        ast::{AST, BinOp, DeclKind, Expr, Stmt},
        lexer::{self, lex},
        parser::{self, parse},
    },
    middle::{
        ir::{IROp, IrInstruction, LoweredOp, Op, TypedExpr},
        semantic::{self, SemanticAnalyzer},
        types::{IRVal, Span, Type},
    },
    pipeline::{CompileError, frontend::FrontendPipeline},
    registry::{file_meta::FileMeta, registry::Registry},
};
use owo_colors::OwoColorize;
use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub fn clean(s: &str) -> String {
    s.replace(|c: char| c.is_whitespace(), "")
}

struct ParseResult {
    ast: AST,
    diagnostics: DiagnosticStore,
}

pub fn parses(src: &str) -> Result<String, CompileError> {
    let ast = parse_to_ast(src)?;
    Ok(ast.to_sexpr())
}

pub fn parse_to_ast(input: &str) -> Result<AST, CompileError> {
    let (ast, _) = parse_with_diagnostics(input)?;
    Ok(ast)
}

pub fn parse_with_diagnostics(input: &str) -> Result<(AST, DiagnosticStore), CompileError> {
    let mut harness: TestHarness = TestHarness::new().with_source(input);
    let pipeline = harness.build_frontend();
    harness
        .run_stage(pipeline)
        .map_err(|_| CompileError::Frontend("pipeline failed".into()))?;
    let ast = harness.get_ast()?;
    let diagnostics = harness.get_diagnostics();
    Ok((ast, diagnostics))
}

pub fn compile_and_lower<'ctx>(
    context: &'ctx Context,
    input: &str,
) -> Result<LLVM<'ctx>, CompileError> {
    let (ast, diagnostics) = parse_with_diagnostics(input)?;

    diagnostics.check_halt()?;

    let mut ir = ast_to_ir(ast)?;

    ir = finalize_ir(ir);

    Ok(LLVM::new(context, &ir))
}

pub fn fails(input: &str) {
    let result = parse_with_diagnostics(input);
    match result {
        Ok((_, diagnostics)) => assert!(diagnostics.has_errors()),
        Err(_) => {
            println!("Error in test");
        }
    }
}

pub fn ast_to_ir(ast: AST) -> Result<Vec<IROp>, CompileError> {
    lower_ast_to_ir(&ast)
}

fn finalize_ir(mut ir: Vec<IROp>) -> Vec<IROp> {
    if !matches!(ir.last(), Some(IROp::Return { .. })) {
        ir.push(IROp::Return { value: None });
    }
    ir
}

pub fn generate_binary_ir(target: &str, left: IRVal, right: IRVal) -> IROp {
    IROp::Binary {
        left,
        op: BinOp::Add,
        right,
    }
}

pub fn get_test_root() -> PathBuf {
    PathBuf::from("/virtual/root")
}

pub fn file(name: &str) -> FileMeta {
    FileMeta {
        path: PathBuf::from(name),
        ..Default::default()
    }
}

pub fn setup_test_context() -> BuildSystem {
    let registry = Registry::from_files(vec![]);
    let utters = UtterRegistry::new();
    BuildSystem::test()
}

pub fn make_registry(files: &[&str]) -> Registry {
    let mut registry = Registry::new();

    for f in files {
        registry.add_file(FileMeta::mock(f));
    }

    registry
}

pub fn make_engine_with_symbols(symbols: Vec<(&str, Symbol)>) -> HashMap<String, Box<dyn Utter>> {
    let mut mock = MockEngine::new("default");

    for (file, symbol) in symbols {
        mock.add_symbol(file, symbol);
    }

    let mut map: HashMap<String, Box<dyn Utter>> = HashMap::new();
    map.insert("default".to_string(), Box::new(mock));

    map
}

pub fn sym(name: &str, value: &str, file: &str) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind: SymbolKind::Constant,
        value: value.to_string(),
        file: FileMeta::mock(file),
        origin: file.to_string(),
        metadata: HashMap::new(),
    }
}

pub fn run_symbol_pipeline(
    registry: &Registry,
    engines: &HashMap<String, Box<dyn Utter>>,
) -> SymbolRegistry {
    let mut sym = SymbolRegistry::new();
    sym.build_all(registry, engines);
    sym
}

pub fn run_incremental_symbol_pipeline(
    registry: &Registry,
    engines: &HashMap<String, Box<dyn Utter>>,
) -> SymbolRegistry {
    let mut sym = SymbolRegistry::new();

    for stack in &registry.stacks {
        let engine = engines.get("default").unwrap();
        sym.build_incremental(stack, engine.as_ref());
    }

    sym
}

pub fn assert_symbol_exists(sym: &SymbolRegistry, name: &str, file: &str) {
    assert!(
        sym.lookup(name, file).is_some(),
        "expected symbol `{}` in `{}`",
        name,
        file
    );
}

pub fn assert_symbol_missing(sym: &SymbolRegistry, name: &str, file: &str) {
    assert!(
        sym.lookup(name, file).is_none(),
        "expected symbol `{}` NOT in `{}`",
        name,
        file
    );
}

pub fn assert_snapshot_value(label: &str, value: impl std::fmt::Display) {
    insta::assert_snapshot!(label, value.to_string());
}
