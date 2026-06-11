use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use loi::{
    backend::{
        symbol::registry::{Symbol, SymbolKind, SymbolRegistry},
        utter::{registry::UtterRegistry, utter::Utter},
    },
    build_system::BuildSystem,
    frontend::{lexer, parser},
    middle::semantic,
    registry::{file_meta::FileMeta, registry::Registry},
};

use crate::harness::mock_engine::MockEngine;

pub struct TestHarness {
    pub registry: Registry,
    pub engines: HashMap<String, Box<dyn Utter>>,
}

impl TestHarness {
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
            engines: HashMap::new(),
        }
    }

    pub fn with_file(mut self, path: &str) -> Self {
        self.registry.add_file(FileMeta::mock(path));
        self
    }

    pub fn with_symbol(mut self, name: &str, value: &str, file: &str) -> Self {
        let sym = Symbol {
            name: name.to_string(),
            kind: SymbolKind::Constant,
            value: value.to_string(),
            file: FileMeta::mock(file),
            origin: file.to_string(),
            metadata: HashMap::new(),
        };

        // Ensure engine exists or create default
        let engine = self
            .engines
            .entry("default".to_string())
            .or_insert_with(|| Box::new(MockEngine::new("default")));

        // This assumes MockEngine has a downcastable or mutable interface
        // You might need to adjust this depending on your actual MockEngine API
        if let Some(mock) = engine.as_any_mut().downcast_mut::<MockEngine>() {
            mock.add_symbol(file, sym);
        }
        self
    }

    /// Run the full symbol pipeline
    pub fn run_pipeline(&self) -> SymbolRegistry {
        let mut sym = SymbolRegistry::new();
        sym.build_all(&self.registry, &self.engines);
        sym
    }

    /// Run the incremental symbol pipeline
    pub fn run_incremental(&self) -> SymbolRegistry {
        let mut sym = SymbolRegistry::new();
        for stack in &self.registry.stacks {
            let engine = self.engines.get("default").expect("No engine found");
            sym.build_incremental(stack, engine.as_ref());
        }
        sym
    }

    pub fn assert_symbol_exists(&self, sym: &SymbolRegistry, name: &str, file: &str) {
        assert!(
            sym.lookup(name, file).is_some(),
            "expected symbol `{}` in `{}`",
            name,
            file
        );
    }

    pub fn assert_symbol_missing(&self, sym: &SymbolRegistry, name: &str, file: &str) {
        assert!(
            sym.lookup(name, file).is_none(),
            "expected symbol `{}` NOT in `{}`",
            name,
            file
        );
    }
}
