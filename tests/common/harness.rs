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
    build::build_system::BuildSystem,
    compiler::diagnostic::DiagnosticStore,
    frontend::{ast::AST, lexer, parser},
    init,
    kernel::Kernel,
    middle::semantic,
    pipeline::{frontend::FrontendPipeline, stage::Stage},
    registry::{file_meta::FileMeta, registry::Registry},
    test_utils::TestEnv,
};

use crate::common::MockEngine;

pub struct TestHarness {
    pub kernel: Kernel,
    pub env: TestEnv,
    pub registry: Registry,
    pub engines: HashMap<String, Box<dyn Utter>>,
}

impl TestHarness {
    pub fn new() -> Self {
        let kernel = init::init();
        Self {
            kernel,
            env: TestEnv::new(),
            registry: Registry::new(),
            engines: HashMap::new(),
        }
    }
    pub fn with_source(self, source: &str) -> Self {
        {
            let mut state = self.env.state.write().unwrap();
            state.source = Some(source.to_string());
        }
        self
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

        let engine = self
            .engines
            .entry("default".to_string())
            .or_insert_with(|| Box::new(MockEngine::new("default")));

        if let Some(mock) = engine.as_any_mut().downcast_mut::<MockEngine>() {
            mock.add_symbol(file, sym);
        }
        self
    }

    pub fn get_ast(&self) -> Result<AST, String> {
        let state = self.env.state.read().unwrap();
        state.ast.clone().ok_or_else(|| "AST missing".to_string())
    }
    pub fn get_diagnostics(&self) -> DiagnosticStore {
        self.env.context.diagnostics.read().unwrap().clone()
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

impl TestHarness {
    pub fn build_frontend(&self) -> FrontendPipeline {
        FrontendPipeline::new(
            self.env.context.clone(),
            self.env.config.clone(),
            self.env.state.clone(),
        )
    }

    pub fn bootstrap(source: &str, symbol_data: Vec<(&str, &str, &str)>) -> Self {
        let mut harness = Self::new().with_source(source);
        for (name, val, file) in symbol_data {
            harness = harness.with_symbol(name, val, file);
        }
        harness
    }

    pub fn run_full_suite(self) -> Result<SymbolRegistry, String> {
        let pipeline = self.build_frontend();
        self.run_stage(pipeline)
            .map_err(|_| "Pipeline failed".to_string())?;

        let sym = self.run_incremental();
        Ok(sym)
    }
    pub fn run_stage<T: Stage>(&self, stage: T) -> Result<(), ()> {
        stage.run()
    }
    pub fn run_pipeline(&self) -> SymbolRegistry {
        let mut sym = SymbolRegistry::new();
        sym.build_all(&self.registry, &self.engines);
        sym
    }
    pub fn run_incremental(&self) -> SymbolRegistry {
        let mut sym = SymbolRegistry::new();
        for stack in &self.registry.stacks {
            let engine = self.engines.get("default").expect("No engine found");
            sym.build_incremental(stack, engine.as_ref());
        }
        sym
    }
}
