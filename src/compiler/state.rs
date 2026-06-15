use clap::Parser;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, PointerValue};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use uuid::Uuid;

use crate::backend::symbol::registry::SymbolRegistry;
use crate::backend::utter::registry::UtterRegistry;
use crate::build::asset_optimizer::AssetOptimizer;
use crate::build::output_resolver::OutputResolver;
use crate::compiler::diagnostic::DiagnosticStore;
use crate::compiler::types::BuildArtifact;
use crate::frontend::ast::AST;
use crate::middle::ir::{IR, IROp, LoweredOp};
use crate::registry::file_meta::{FileMeta, GroupKey};
use crate::registry::registry::{FileStack, Registry};
use crate::{
    compiler::config::CompileConfig,
    pipeline::{CompileError, Metadata, Pipeline},
};

use std::collections::HashSet;

/// =========================
/// SYMBOL ID
/// =========================
/// Purpose:
/// Eventually this should NOT be a u64.
/// Replace with a stable interned ID or (ModuleId, LocalId)
pub type SymbolId = u64;

/// =========================
/// FILE GRAPH
/// =========================
/// Purpose:
/// Tracks file-level relationships:
/// - imports / includes
/// - compilation order hints
/// - dependency edges between modules
#[derive(Default, Debug, Clone)]
pub struct FileGraph {
    /// file -> files it imports
    pub imports: HashMap<Uuid, Vec<Uuid>>,

    /// file -> files that depend on it (reverse edges)
    pub dependents: HashMap<Uuid, Vec<Uuid>>,

    /// optional: topologically sorted compilation order cache
    pub topo_order: Vec<Uuid>,

    /// cycle detection cache (file-level)
    pub cycles: Vec<Vec<Uuid>>,
}

/// =========================
/// DEPENDENCY GRAPH (SYMBOL LEVEL)
/// =========================
/// Purpose:
/// Fine-grained dependency tracking between symbols.
/// Used for incremental recompilation.
#[derive(Default, Debug, Clone)]
pub struct DependencyGraph {
    /// symbol -> symbols it depends on
    pub forward: HashMap<SymbolId, HashSet<SymbolId>>,

    /// symbol -> symbols that depend on it
    pub reverse: HashMap<SymbolId, HashSet<SymbolId>>,

    /// cached invalidation closure sets
    pub transitive_closure_cache: HashMap<SymbolId, HashSet<SymbolId>>,

    /// cycle detection (rare but important for recursive definitions)
    pub cycles: Vec<Vec<SymbolId>>,
}

/// =========================
/// SYMBOL INDEX
/// =========================
/// Purpose:
/// Fast lookup layer for symbol resolution
/// (names, scopes, modules, shadowing rules)
#[derive(Default, Debug, Clone)]
pub struct SymbolIndex {
    /// name -> symbol ids (supports overloading / shadowing)
    pub by_name: HashMap<String, Vec<SymbolId>>,

    /// module/file -> symbols defined in it
    pub by_file: HashMap<Uuid, Vec<SymbolId>>,

    /// fully qualified name cache (e.g. module::sub::symbol)
    pub fqns: HashMap<String, SymbolId>,

    /// scope stack cache (for fast lookup during parsing)
    pub scope_stack: Vec<Vec<SymbolId>>,
}

/// =========================
/// IR CACHE
/// =========================
/// Purpose:
/// Stores intermediate representation per file/module
#[derive(Default, Debug, Clone)]
pub struct IRCache {
    pub per_file: HashMap<Uuid, IR>,
    pub per_symbol: HashMap<SymbolId, IR>,
    pub dedup_cache: HashMap<u64, IR>,
    pub ir_versions: HashMap<Uuid, u32>,
    pub current: Option<IR>,
}

#[derive(Debug, Clone)]
pub struct LoweredIR {
    pub nodes: Vec<IROp>,
}

/// =========================
/// LOWERED CACHE (POST IR OPTIMIZATION)
/// =========================
/// Purpose:
/// Stores optimized / backend-ready IR
#[derive(Default, Debug, Clone)]
pub struct LoweredCache {
    /// file -> lowered IR
    pub per_file: HashMap<Uuid, Vec<LoweredOp>>,

    /// symbol-level lowered fragments
    pub per_symbol: HashMap<SymbolId, Vec<LoweredOp>>,

    /// backend-specific cache (LLVM / WASM / etc.)
    pub backend_cache: HashMap<String, Vec<u8>>,

    /// optimization pass versioning
    pub opt_pass_version: u32,

    pub current: Option<LoweredIR>,
}

/// =========================
/// BUILD CACHE
/// =========================
/// Purpose:
/// Speeds up rebuilds by caching:
/// - IR → object code
/// - file hash → build artifacts
#[derive(Default, Debug, Clone)]
pub struct BuildCache {
    /// file hash -> compiled artifact bytes
    pub object_cache: HashMap<u64, Vec<u8>>,

    /// file hash -> IR snapshot
    pub ir_cache: HashMap<u64, IR>,

    /// symbol hash -> compiled output
    pub symbol_cache: HashMap<u64, Vec<u8>>,

    /// timestamps for cache validation
    pub timestamps: HashMap<PathBuf, u64>,

    /// global cache version (invalidate everything if bumped)
    pub cache_version: u32,
    pub current: Option<BuildArtifact>,
}

impl BuildCache {
    pub fn insert_artifact(&mut self, hash: u64, artifact: Vec<u8>) {
        self.object_cache.insert(hash, artifact);
    }

    pub fn insert_ir(&mut self, hash: u64, ir: IR) {
        self.ir_cache.insert(hash, ir);
    }

    pub fn insert_symbol(&mut self, hash: u64, output: Vec<u8>) {
        self.symbol_cache.insert(hash, output);
    }

    pub fn set_current(&mut self, artifact: BuildArtifact) {
        self.current = Some(artifact);
    }
}

/// Final backend output
// CompilerState is the *mutable brain* of the compiler.
//
// IMPORTANT DESIGN NOTE:
// - Env = immutable "what world am I in"
// - State = evolving "what do I know so far"
// - Engine = "what do I do with it"
//
// This struct should be safe to discard/rebuild from scratch,
// except for caches if you later introduce persistent incremental builds.
#[derive(Debug)]
pub struct CompileState {
    pub source: Option<String>,
    pub ast: Option<AST>,

    // =========================
    // FILE SYSTEM MODEL
    // =========================
    /// Canonical registry of all known files
    pub registry: Registry,

    /// Directed graph of file relationships (imports/includes)
    pub file_graph: FileGraph,

    /// Symbol-level dependency graph (who depends on whom)
    pub dependency_graph: DependencyGraph,

    // =========================
    // SYMBOL SYSTEM
    // =========================
    /// Global symbol storage (definitions, metadata, ownership)
    pub symbols: SymbolRegistry,

    /// Fast lookup index (name → symbol id, scoped resolution, etc.)
    pub symbol_index: SymbolIndex,

    // =========================
    // IR PIPELINE
    // =========================
    /// Cached intermediate representations per file/module
    pub ir: Option<IR>,
    pub ir_cache: IRCache,
    /// Lowered IR (post-optimization / pre-codegen cache)
    pub lowered_cache: LoweredCache,

    // =========================
    // INCREMENTAL COMPILATION
    // =========================
    /// Files that must be recompiled due to changes
    pub dirty_files: HashSet<Uuid>,

    /// Symbols that are invalidated (definition changed or moved)
    pub dirty_symbols: HashSet<SymbolId>,

    // =========================
    // CACHING / PERFORMANCE
    // =========================
    /// File content hashes (used for change detection)
    /// NOTE: PathBuf is acceptable early-stage, but later you may want FileId instead
    pub content_hashes: HashMap<PathBuf, u64>,

    /// Build-level cache (IR → object code, etc.)
    pub build_cache: BuildCache,

    // =========================
    // DIAGNOSTICS
    // =========================
    /// Central diagnostic store (errors, warnings, spans, trace info)
    // pub diagnostics: DiagnosticStore,

    // =========================
    // VERSIONING (CRITICAL FOR LONG-TERM STABILITY)
    // =========================
    /// Compiler semantic version (used for cache invalidation + reproducibility)
    pub compiler_version: String,

    /// IR format version (must bump when IR structure changes)
    pub ir_version: u32,
}

impl CompileState {
    pub fn current_ir(&self) -> Option<IR> {
        self.ir_cache.current.clone()
    }

    pub fn current_lowered_ir(&self) -> Option<LoweredIR> {
        self.lowered_cache.current.clone()
    }

    pub fn current_artifact(&self) -> Option<BuildArtifact> {
        self.build_cache.current.clone()
    }
    pub fn registry_is_empty(&self) -> bool {
        self.registry.is_empty()
    }
}
impl Default for CompileState {
    fn default() -> Self {
        Self {
            source: None,
            ast: None,
            registry: Registry::default(),
            file_graph: FileGraph::default(),
            dependency_graph: DependencyGraph::default(),
            symbols: SymbolRegistry::default(),
            symbol_index: SymbolIndex::default(),
            ir: None,
            ir_cache: IRCache::default(),
            lowered_cache: LoweredCache::default(),
            dirty_files: HashSet::new(),
            dirty_symbols: HashSet::new(),
            content_hashes: HashMap::new(),
            build_cache: BuildCache::default(),
            // diagnostics: DiagnosticStore::default(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            ir_version: 1,
        }
    }
}
