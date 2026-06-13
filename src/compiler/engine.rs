use clap::Parser;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, PointerValue};
use owo_colors::OwoColorize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::backend::symbol::registry::SymbolRegistry;
use crate::backend::utter::registry::UtterRegistry;
use crate::build::asset_optimizer::AssetOptimizer;
use crate::build::output_resolver::OutputResolver;
use crate::build::service::BundleService;
use crate::compiler::addon::{BackendRegistry, PassRegistry, PipelineExtensions};
use crate::compiler::bundler::OutputEmitter;
use crate::compiler::cache::{CachePolicy, CompilationCache, MemoryCache, PersistentCache};
use crate::compiler::context::CompilerContext;
use crate::compiler::diagnostic::{CompilerEventBus, Inspector, Logger, Profiler, TraceSystem};
use crate::compiler::execution::{JobQueue, PluginSystem, PrioritySystem, TaskScheduler};
use crate::compiler::runtime::{IRRuntime, LoweringRuntime, SymbolRuntime};
use crate::compiler::safety::{FallbackPipeline, RecoverySystem};
use crate::compiler::scale::{BuildFarm, DistributedCompiler};
use crate::development::watcher::{
    ChangeDetector, FileWatcher, HotReloadManager, IncrementalCompiler,
};
use crate::frontend::ast::AST;
use crate::middle::ir::IR;
use crate::pipeline::backend::BackendPipeline;
use crate::pipeline::frontend::FrontendPipeline;
use crate::pipeline::middle::MiddlePipeline;
use crate::registry::file_meta::{FileMeta, GroupKey};
use crate::registry::registry::FileStack;

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
#[derive(Default)]
pub struct CompilerEngine {
    // =========================================================
    // PIPELINE STAGES
    // =========================================================
    /// Frontend: lexing, parsing, AST building, early diagnostics
    pub frontend: FrontendPipeline,

    /// Middle: IR generation, transformation, optimization passes
    pub middle: MiddlePipeline,

    /// Backend: LLVM / WASM / custom codegen backends
    pub backend: BackendPipeline,

    /// Optional experimental pipeline extensions (plugins / future passes)
    pub extensions: PipelineExtensions,

    // =========================================================
    // OUTPUT / BUNDLING SYSTEM
    // =========================================================
    /// Final bundling of artifacts (JS/WASM/native/etc.)
    pub bundler: BundleService,

    /// Resolves output paths, module graphs, and artifact placement
    pub resolver: OutputResolver,

    /// Asset optimization (minify, strip, compress, tree-shake)
    pub optimizer: AssetOptimizer,

    /// Output emission strategy (disk, memory, distributed)
    pub emitter: OutputEmitter,

    // =========================================================
    // EXECUTION CONTROL
    // =========================================================
    /// Number of worker threads allowed for compilation
    pub concurrency: usize,

    /// Enables parallel execution of pipeline stages
    pub parallel_enabled: bool,

    /// Scheduler for tasks (VERY important for scaling + incremental builds)
    pub scheduler: TaskScheduler,

    /// Work queue for compilation jobs
    pub job_queue: JobQueue,

    /// Priority system (hot files, changed symbols, etc.)
    pub priority_system: PrioritySystem,

    // =========================================================
    // INCREMENTAL + HOT RELOAD
    // =========================================================
    /// File watcher for dev mode
    pub watcher: Option<FileWatcher>,

    /// Hot reload manager (state preservation between recompiles)
    pub hot_reload: Option<HotReloadManager>,

    /// Incremental compilation controller
    pub incremental: IncrementalCompiler,

    /// Change detector (file + symbol + IR diffing)
    pub change_detector: ChangeDetector,

    // =========================================================
    // CACHING SYSTEM
    // =========================================================
    /// Global compilation cache
    pub cache: CompilationCache,

    /// Persistent disk cache (cross-run speedups)
    pub persistent_cache: PersistentCache,

    /// In-memory fast cache (IR, AST, symbol resolution)
    pub memory_cache: MemoryCache,

    /// Cache invalidation rules engine
    pub cache_policy: CachePolicy,

    // =========================================================
    // SYMBOL + IR HOOKS
    // =========================================================
    /// Symbol resolution integration layer
    pub symbol_runtime: SymbolRuntime,

    /// IR transformation pipeline hook
    pub ir_runtime: IRRuntime,

    /// Lowering coordination layer
    pub lowering_runtime: LoweringRuntime,

    // =========================================================
    // LOGGING / DEBUG / INTROSPECTION
    // =========================================================
    /// Structured logger
    pub logger: Logger,

    /// Tracing system (for performance + compiler visualization)
    pub tracer: TraceSystem,

    /// Profiler (stage timing, memory usage, hot paths)
    pub profiler: Profiler,

    /// Debug inspector (inspect AST/IR/symbol graph live)
    pub inspector: Inspector,

    /// Event bus for compiler events (useful for IDEs, tools)
    pub event_bus: CompilerEventBus,

    // =========================================================
    // ADDON / PLUGIN / EXTENSIBILITY
    // =========================================================
    /// Plugin system for custom passes / backends
    pub plugins: PluginSystem,

    /// Foreign backend interface (LLVM, Cranelift, WASM, etc.)
    pub backend_registry: BackendRegistry,

    /// Custom pass injection system
    pub pass_registry: PassRegistry,

    // =========================================================
    // DISTRIBUTED / FUTURE SCALING
    // =========================================================
    /// Remote compilation workers (future distributed builds)
    pub distributed: Option<DistributedCompiler>,

    /// Build farm coordinator
    pub build_farm: Option<BuildFarm>,

    /// Network cache layer (shared builds across machines)
    pub network_cache: Option<MemoryCache>,

    // =========================================================
    // SAFETY / RECOVERY
    // =========================================================
    /// Crash recovery system (resume interrupted builds)
    pub recovery: RecoverySystem,

    /// Fallback pipeline if optimization fails
    pub fallback: FallbackPipeline,

    /// Safe-mode compiler (minimal optimizations, maximum stability)
    pub safe_mode: bool,
}

impl CompilerEngine {
    pub fn new() -> Self {
        let context = CompilerContext::new();
        let logger = Logger::default();
        let tracer = TraceSystem::new();
        let profiler = Profiler::new();

        let cache_policy = CachePolicy::default();
        let memory_cache = MemoryCache::new();
        let persistent_cache = PersistentCache::new();
        let cache = CompilationCache::new();

        let scheduler = TaskScheduler::new();
        let job_queue = JobQueue::new();
        let priority_system = PrioritySystem::new();

        let frontend = FrontendPipeline::new(context, logger.clone());
        let middle = MiddlePipeline::new(logger.clone());
        let backend = BackendPipeline::new(logger.clone());

        let symbol_runtime = SymbolRuntime::new();
        let ir_runtime = IRRuntime::new();
        let lowering_runtime = LoweringRuntime::new();

        let bundler = BundleService::new();
        let resolver = OutputResolver::new();
        let optimizer = AssetOptimizer::new();
        let emitter = OutputEmitter::default();

        let plugins = PluginSystem::new();
        let backend_registry = BackendRegistry::new();
        let pass_registry = PassRegistry::new();

        let incremental = IncrementalCompiler::new();
        let change_detector = ChangeDetector::new();

        let watcher = Some(FileWatcher::new());
        let hot_reload = Some(HotReloadManager::new());

        let recovery = RecoverySystem::new();
        let fallback = FallbackPipeline::new();

        let inspector = Inspector::new();
        let event_bus = CompilerEventBus::new();

        let distributed = None;
        let build_farm = None;
        let network_cache = None;

        Self {
            frontend,
            middle,
            backend,

            extensions: PipelineExtensions::default(),

            bundler,
            resolver,
            optimizer,
            emitter,

            concurrency: num_cpus::get(),
            parallel_enabled: true,
            scheduler,
            job_queue,
            priority_system,

            watcher,
            hot_reload,
            incremental,
            change_detector,

            cache,
            persistent_cache,
            memory_cache,
            cache_policy,

            symbol_runtime,
            ir_runtime,
            lowering_runtime,

            logger,
            tracer,
            profiler,
            inspector,
            event_bus,

            plugins,
            backend_registry,
            pass_registry,

            distributed,
            build_farm,
            network_cache,

            recovery,
            fallback,
            safe_mode: false,
        }
    }
}
