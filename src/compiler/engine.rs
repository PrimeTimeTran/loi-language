use std::sync::{Arc, RwLock};

use crate::{
    compiler::{config::CompileConfig, state::CompileState},
    context::Context,
    pipeline::{
        backend::BackendPipeline, frontend::FrontendPipeline, middle::MiddlePipeline, stage::Stage,
    },
};

// use crate::backend::symbol::registry::SymbolRegistry;
// use crate::backend::utter::registry::UtterRegistry;
// use crate::build::asset_optimizer::AssetOptimizer;
// use crate::build::output_resolver::OutputResolver;
// use crate::build::service::BundleService;
// use crate::compiler::addon::{BackendRegistry, PassRegistry, PipelineExtensions};
// use crate::compiler::bundler::OutputEmitter;
// use crate::compiler::cache::{CachePolicy, CompilationCache, MemoryCache, PersistentCache};
// use crate::compiler::diagnostic::{CompilerEventBus, Inspector, Logger, Profiler, TraceSystem};
// use crate::compiler::execution::{JobQueue, PluginSystem, PrioritySystem, TaskScheduler};
// use crate::compiler::runtime::{IRRuntime, LoweringRuntime, SymbolRuntime};
// use crate::compiler::safety::{FallbackPipeline, RecoverySystem};
// use crate::compiler::scale::{BuildFarm, DistributedCompiler};
// use crate::context::{CompileContext, Context};
// use crate::development::watcher::{
//     ChangeDetector, FileWatcher, HotReloadManager, IncrementalCompiler,
// };
// use crate::frontend::parser::Parser;
// use crate::interface::CompileEngineProvider;
// use crate::middle::ir::IR;
// use crate::pipeline::frontend::{FrontendFeatures, FrontendPipeline};
// use crate::registry::file_meta::{FileMeta, GroupKey};
// use crate::registry::prog_registry::FileStack;

// =========================================================
// MACRO: SAFE DEFAULT BOOTSTRAP
// =========================================================
/// Provides a consistent way to build large nested compiler structs
/// without repeating boilerplate everywhere.
// macro_rules! compiler_defaults {
//     ($t:ty) => {
//         <$t>::default()
//     };
// }
/// =========================
/// COMPILER ENGINE
/// =========================
///
/// PURPOSE:
/// Orchestrates the entire compilation lifecycle:
/// - frontend parsing + symbol resolution
/// - middle IR transformation + optimization
/// - backend lowering + codegen
/// - bundling + output emission
/// - watch mode + hot reload
///
/// DESIGN GOAL:
/// This should behave like a "compiler runtime kernel".
#[derive(Debug)]
pub struct CompileEngine {
    pub state: Arc<RwLock<CompileState>>,
    pub config: Arc<RwLock<CompileConfig>>,
    // =========================================================
    // PIPELINE STAGES
    /// Frontend: lexing, parsing, AST building, early diagnostics
    /// Middle: IR generation, transformation, optimization passes
    /// Backend: LLVM / WASM / custom codegen backends
    // =========================================================
    pub stages: Vec<Box<dyn Stage>>,
    // Optional experimental pipeline extensions (plugins / future passes)
    // pub extensions: PipelineExtensions,

    // // =========================================================
    // // OUTPUT / BUNDLING SYSTEM
    // // =========================================================
    // /// Final bundling of artifacts (JS/WASM/native/etc.)
    // pub bundler: BundleService,

    // pub resolver: OutputResolver,

    // /// Asset optimization (minify, strip, compress, tree-shake)
    // pub optimizer: AssetOptimizer,

    // /// Output emission strategy (disk, memory, distributed)
    // pub emitter: OutputEmitter,

    // // =========================================================
    // // EXECUTION CONTROL
    // // =========================================================
    // /// Number of worker threads allowed for compilation
    // pub concurrency: usize,

    // /// Enables parallel execution of pipeline stages
    // pub parallel_enabled: bool,

    // /// Scheduler for tasks (VERY important for scaling + incremental builds)
    // pub scheduler: TaskScheduler,

    // /// Work queue for compilation jobs
    // pub job_queue: JobQueue,

    // /// Priority system (hot files, changed symbols, etc.)
    // pub priority_system: PrioritySystem,

    // // =========================================================
    // // INCREMENTAL + HOT RELOAD
    // // =========================================================
    // /// File watcher for dev mode
    // pub watcher: Option<FileWatcher>,

    // /// Hot reload manager (state preservation between recompiles)
    // pub hot_reload: Option<HotReloadManager>,

    // /// Incremental compilation controller
    // pub incremental: IncrementalCompiler,

    // /// Change detector (file + symbol + IR diffing)
    // pub change_detector: ChangeDetector,

    // // =========================================================
    // // CACHING SYSTEM
    // // =========================================================
    // /// Global compilation cache
    // pub cache: CompilationCache,

    // /// Persistent disk cache (cross-run speedups)
    // pub persistent_cache: PersistentCache,

    // /// In-memory fast cache (IR, AST, symbol resolution)
    // pub memory_cache: MemoryCache,

    // /// Cache invalidation rules engine
    // pub cache_policy: CachePolicy,

    // // =========================================================
    // // SYMBOL + IR HOOKS
    // // =========================================================
    // /// Symbol resolution integration layer
    // pub symbol_runtime: SymbolRuntime,

    // /// IR transformation pipeline hook
    // pub ir_runtime: IRRuntime,

    // /// Lowering coordination layer
    // pub lowering_runtime: LoweringRuntime,

    // // =========================================================
    // // LOGGING / DEBUG / INTROSPECTION
    // // =========================================================
    // /// Structured logger
    // pub logger: Logger,

    // /// Tracing system (for performance + compiler visualization)
    // pub tracer: TraceSystem,

    // /// Profiler (stage timing, memory usage, hot paths)
    // pub profiler: Profiler,

    // /// Debug inspector (inspect AST/IR/symbol graph live)
    // pub inspector: Inspector,

    // /// Event bus for compiler events (useful for IDEs, tools)
    // pub event_bus: CompilerEventBus,

    // // =========================================================
    // // ADDON / PLUGIN / EXTENSIBILITY
    // // =========================================================
    // /// Plugin system for custom passes / backends
    // pub plugins: PluginSystem,

    // /// Foreign backend interface (LLVM, Cranelift, WASM, etc.)
    // pub backend_registry: BackendRegistry,

    // /// Custom pass injection system
    // pub pass_registry: PassRegistry,

    // // =========================================================
    // // DISTRIBUTED / FUTURE SCALING
    // // =========================================================
    // /// Remote compilation workers (future distributed builds)
    // pub distributed: Option<DistributedCompiler>,

    // /// Build farm coordinator
    // pub build_farm: Option<BuildFarm>,

    // /// Network cache layer (shared builds across machines)
    // pub network_cache: Option<MemoryCache>,

    // // =========================================================
    // // SAFETY / RECOVERY
    // // =========================================================
    // /// Crash recovery system (resume interrupted builds)
    // pub recovery: RecoverySystem,

    // /// Fallback pipeline if optimization fails
    // pub fallback: FallbackPipeline,

    // /// Safe-mode compiler (minimal optimizations, maximum stability)
    // pub safe_mode: bool,
}

#[derive(Debug)]
pub enum StageError {
    StageFailed(String),
}

impl CompileEngine {
    fn parse(&mut self) {}
    fn analyze(&mut self) {}
    fn lower(&mut self) {}
    fn backend(&mut self) {}
    fn build(&mut self) {}

    pub fn run_all(&self) -> Result<(), StageError> {
        for stage in &self.stages {
            println!("Running: {}...", stage.name());

            // stage.run().map_err(|e| StageError::StageFailed(e.to_string()))?;
        }

        Ok(())
    }
    pub fn new(
        context: Arc<Context>,
        config: Arc<RwLock<CompileConfig>>,
        state: Arc<RwLock<CompileState>>,
    ) -> Self {
        Self {
            state: state.clone(),
            config: config.clone(),
            stages: vec![
                Box::new(FrontendPipeline::new(
                    context.clone(),
                    config.clone(),
                    state.clone(),
                )),
                Box::new(MiddlePipeline::new(
                    context.clone(),
                    config.clone(),
                    state.clone(),
                )),
                Box::new(BackendPipeline::new(context, config, state)),
            ],
        }
    }
}

#[cfg(test)]
impl Default for CompileEngine {
    fn default() -> Self {
        let context = Arc::new(Context::new());
        let config = Arc::new(RwLock::new(CompileConfig::default()));
        let state = Arc::new(RwLock::new(CompileState::default()));
        Self::new(context, config, state)
    }
}
