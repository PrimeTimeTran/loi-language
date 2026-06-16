use loi::{
    backend::{
        symbol::registry::{Symbol, SymbolKind, SymbolRegistry},
        utter::{registry::UtterRegistry, utter::Utter},
    },
    build::build_system::BuildSystem,
    frontend::{lexer, parser},
    middle::semantic::{self, SemanticAnalyzer},
    registry::{file_meta::FileMeta, registry::Registry},
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::harness::mock_engine::MockEngine;

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

// pub fn compile_project(
//     registry: Registry,
//     engines: HashMap<String, Box<dyn Utter>>,
// ) -> CompileResult {
//     let symbols = run_symbol_pipeline(&registry, &engines);
//     let ir = lower_to_ir(&registry, &symbols);
//     let llvm = generate_llvm(&ir);

//     CompileResult { symbols, ir, llvm }
// }
