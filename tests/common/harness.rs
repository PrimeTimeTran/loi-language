use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    ptr::hash,
    sync::{Arc, RwLock},
};

use loi::{
    backend::{
        symbol::registry::{Symbol, SymbolKind, SymbolRegistry},
        utter::{registry::UtterRegistry, utter::Utter},
    },
    build::build_system::BuildSystem,
    compiler::{
        config::CompileConfig, diagnostic::DiagnosticStore, engine::CompileEngine,
        state::CompileState, types::BuildArtifact,
    },
    context::Context,
    frontend::{ast::AST, lexer, parser},
    init,
    kernel::{Kernel, KernelBuilder},
    middle::semantic,
    pipeline::{
        CompileError,
        backend::{BackendPipeline, BackendTarget, CodegenConfig, OptimizationLevel},
        frontend::{FrontendFeatures, FrontendPipeline},
        middle::{IRConfig, MiddleFeatures, MiddlePipeline},
        runner::PipelineRunner,
        stage::Stage,
    },
    registry::{file_meta::FileMeta, registry::Registry},
    test_utils::TestEnv,
};

use crate::common::MockEngine;

pub enum PipelineTarget {
    Frontend,
    Middle,
    Backend,
    Full,
}

pub struct TestHarness {
    pub kernel: Kernel,
    pub env: TestEnv,
    pub registry: Registry,
    pub engines: HashMap<String, Box<dyn Utter>>,
    // pub engine: CompileEngine,
    pub engine: Arc<CompileEngine>,
}

impl TestHarness {
    pub fn run(&mut self) -> Result<(), CompileError> {
        let mut runner = PipelineRunner::new();

        runner.add_stage(self.build_frontend());
        runner.add_stage(self.build_middle());
        runner.add_stage(self.build_backend());

        runner.run(&self.engine)?;

        Ok(())
    }
}

impl TestHarness {
    pub fn new() -> Self {
        let diagnostics = Arc::new(RwLock::new(DiagnosticStore::default()));
        let context = Arc::new(Context::new());
        let config = Arc::new(RwLock::new(CompileConfig::default()));
        let state = Arc::new(RwLock::new(CompileState::default()));

        let engine = Arc::new(CompileEngine::new(
            context.clone(),
            config.clone(),
            state.clone(),
        ));

        let kernel = KernelBuilder::new()
            .context(context.clone())
            .engine(engine.clone())
            .diagnostics(diagnostics.clone())
            .build();

        let env = TestEnv {
            state: state.clone(),
            config: config.clone(),
            context: context.clone(),
        };
        Self {
            engine,
            kernel,
            env,
            registry: Registry::new(),
            engines: HashMap::new(),
        }
    }
    pub fn with_file(mut self, path: &str) -> Self {
        self.registry.add_file(FileMeta::mock(path));
        self
    }
    pub fn with_source(self, source: &str) -> Self {
        {
            let mut state = self.env.state.write().unwrap();
            state.source = Some(source.to_string());
        }
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

    pub fn get_ast(&self) -> Result<AST, CompileError> {
        let state = self.env.state.read().unwrap();

        state
            .current_ast()
            .ok_or_else(|| CompileError::Frontend("AST missing from test harness".into()))
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
    pub fn bootstrap(source: &str, symbol_data: Vec<(&str, &str, &str)>) -> Self {
        let mut harness = Self::new().with_source(source);
        for (name, val, file) in symbol_data {
            harness = harness.with_symbol(name, val, file);
        }
        harness
    }

    pub fn build_frontend(&self) -> FrontendPipeline {
        FrontendPipeline::new(
            self.env.context.clone(),
            self.env.config.clone(),
            self.env.state.clone(),
        )
        .with_features(FrontendFeatures::default())
    }
    pub fn build_middle(&self) -> MiddlePipeline {
        MiddlePipeline::new(
            self.env.context.clone(),
            self.env.config.clone(),
            self.env.state.clone(),
        )
        .with_ir_config(IRConfig::default())
        .with_features(MiddleFeatures::default())
    }

    pub fn build_backend(&self) -> BackendPipeline {
        BackendPipeline::new(
            self.env.context.clone(),
            self.env.config.clone(),
            self.env.state.clone(),
        )
        .with_target(BackendTarget::default())
        .with_opt_level(OptimizationLevel::default())
        .with_codegen_config(CodegenConfig::default())
        .with_debug(false)
    }

    pub fn run_stage<T: Stage>(&mut self, stage: T) -> Result<(), CompileError> {
        stage.run(&self.engine)
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

// impl TestHarness {
//     pub fn run_stage<T, E>(&self, mut stage: T) -> Result<(), E>
//     where
//         T: Stage<CompileError = E>,
//     {
//         stage.run(self.env.clone())
//     }
// }
