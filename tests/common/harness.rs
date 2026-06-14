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
}

impl TestHarness {
    pub fn run(&mut self) -> Result<(), ()> {
        let mut runner = PipelineRunner::new();

        runner.add_stage(self.build_frontend());
        runner.add_stage(self.build_middle());
        runner.add_stage(self.build_backend());

        runner.run();

        Ok(())
    }
    // pub fn run(&mut self, target: PipelineTarget) -> Result<(), ()> {
    //     match target {
    //         PipelineTarget::Frontend => {
    //             let p = self.build_frontend();
    //             p.run()?;
    //             Ok(())
    //         }

    //         PipelineTarget::Middle => {
    //             let p = self.build_middle();
    //             p.run()?;
    //             Ok(())
    //         }
    //         PipelineTarget::Backend => {
    //             let ir = {
    //                 let state = self.kernel.engine.state.read().map_err(|_| ())?;
    //                 state.current_ir()
    //             }
    //             .ok_or(())?;

    //             let backend = self.build_backend();
    //             let output = backend.run(ir.clone());

    //             // compute stable hash for caching
    //             let hash = {
    //                 use std::collections::hash_map::DefaultHasher;
    //                 use std::hash::{Hash, Hasher};

    //                 let mut hasher = DefaultHasher::new();
    //                 ir.nodes.hash(&mut hasher); // assumes IR { nodes: Vec<IROp> }
    //                 hasher.finish()
    //             };

    //             let mut state = self.kernel.engine.state.write().map_err(|_| ())?;

    //             state.build_cache.insert_artifact(hash, output.clone());

    //             state.build_cache.set_current(BuildArtifact::Llvm(output));

    //             Ok(())
    //         }
    //         PipelineTarget::Full => {
    //             self.kernel.engine.run_all()?;
    //             Ok(())
    //         }
    //     }
    // }
}

impl TestHarness {
    pub fn new() -> Self {
        let diagnostics = Arc::new(RwLock::new(DiagnosticStore::default()));
        let context = Arc::new(Context::new());
        let config = Arc::new(RwLock::new(CompileConfig::default()));
        let state = Arc::new(RwLock::new(CompileState::default()));
        let engine = CompileEngine::new(context.clone(), config.clone(), state.clone());
        let kernel = KernelBuilder::new()
            .context(context.clone())
            .engine(engine)
            .diagnostics(diagnostics)
            .build();
        let env = TestEnv {
            state,
            config,
            context: kernel.context.clone(),
        };

        Self {
            kernel,
            env,
            registry: Registry::new(),
            engines: HashMap::new(),
        }
    }
    pub fn with_source(self, source: &str) -> Self {
        println!("WITH_SOURCE");
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
        state
            .ast
            .clone()
            .ok_or_else(|| "AST missing from get_AST test harness".to_string())
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
        println!("BOOTSTRAP");
        let mut harness = Self::new().with_source(source);
        for (name, val, file) in symbol_data {
            harness = harness.with_symbol(name, val, file);
        }
        harness
    }

    pub fn build_frontend(&self) -> FrontendPipeline {
        println!("FRONTEND DONE");
        FrontendPipeline::new(
            self.env.context.clone(),
            self.env.config.clone(),
            self.env.state.clone(),
        )
        .with_features(FrontendFeatures::default())
    }
    pub fn build_middle(&self) -> MiddlePipeline {
        println!("MIDDLE START");

        MiddlePipeline::new(
            self.env.context.clone(),
            self.env.config.clone(),
            self.env.state.clone(),
        )
        .with_ir_config(IRConfig::default())
        .with_features(MiddleFeatures::default())
    }

    pub fn build_backend(&self) -> BackendPipeline {
        println!("BACKEND START");

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

    pub fn run_stage<T: Stage>(&mut self, stage: T) -> Result<(), ()> {
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
