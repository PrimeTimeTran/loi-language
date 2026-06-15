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

#[derive(Debug, Default)]
pub struct CompilerCaches {
    pub build: BuildCache,
    pub lowered: LoweredCache,
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
#[derive(Debug, Clone)]
pub struct LoweredIR {
    pub nodes: Vec<IROp>,
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
    // ========================================
    // === 1. WORLD MODEL
    // "What exists in this project?"
    // ========================================
    /// Canonical registry of all known files/modules.
    /// Owns file identity and metadata.
    pub registry: Registry,

    /// Directed graph of file relationships.
    /// Tracks imports/includes between files.
    pub file_graph: FileGraph,

    /// Symbol dependency graph.
    /// Tracks which symbols depend on other symbols.
    pub dependency_graph: DependencyGraph,

    // ========================================
    // === 2. SEMANTIC MODEL
    // "What does everything mean?"
    // ========================================
    /// Global symbol storage.
    /// Contains definitions, ownership, metadata, and resolved symbols.
    pub symbols: SymbolRegistry,

    /// Fast symbol lookup index.
    /// Handles name resolution, scopes, and symbol id lookup.
    pub symbol_index: SymbolIndex,

    // ========================================
    // === 3. INCREMENTAL COMPILATION
    // "What needs rebuilding?"
    // ========================================
    /// Files invalidated by source changes.
    /// Used to avoid recompiling unchanged files.
    pub dirty_files: HashSet<Uuid>,

    /// Symbols invalidated by definition changes.
    /// Used for dependency-aware recompilation.
    pub dirty_symbols: HashSet<SymbolId>,

    /// Content fingerprints used for change detection.
    /// Later can migrate from PathBuf -> FileId.
    pub content_hashes: HashMap<PathBuf, u64>,

    // ========================================
    // === 4. ACTIVE COMPILATION STATE
    // "What am I compiling right now?"
    // ========================================
    /// Current source input.
    /// Used by single-file compilation and REPL workflows.
    pub source: Option<String>,

    /// Parsed syntax tree from current source.
    pub ast: Option<AST>,

    /// Current intermediate representation.
    /// Produced by frontend/middle pipeline.
    pub current_ir: Option<IR>,

    /// Backend-ready IR after optimization/lowering.
    /// Consumed by LLVM/backend pipeline.
    pub current_lowered_ir: Option<LoweredIR>,

    /// Final compilation output.
    /// Executable/object/library/etc.
    pub current_artifact: Option<BuildArtifact>,

    // ========================================
    // === 5. CACHING / PERFORMANCE
    // "What have we already computed?"
    // ========================================
    /// Persistent compiler caches.
    /// Stores reusable IR, lowered IR, and build artifacts.
    pub caches: CompilerCaches,

    // ========================================
    // === 6. DIAGNOSTICS
    // "What did compilation report?"
    // ========================================
    /// Central diagnostics storage.
    /// Errors, warnings, spans, and compiler messages.
    // pub diagnostics: DiagnosticStore,

    // ========================================
    // === 7. VERSIONING
    // "Are cached results still valid?"
    // ========================================

    /// Compiler version.
    /// Used for cache invalidation and reproducible builds.
    pub compiler_version: String,

    /// IR format version.
    /// Increment when IR structure changes.
    pub ir_version: u32,
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
impl CompileState {
    pub fn current_ir(&self) -> Option<IR> {
        self.current_ir.clone()
    }
    pub fn current_lowered_ir(&self) -> Option<LoweredIR> {
        self.current_lowered_ir.clone()
    }
    pub fn current_artifact(&self) -> Option<BuildArtifact> {
        self.current_artifact.clone()
    }
    pub fn registry_is_empty(&self) -> bool {
        self.registry.is_empty()
    }
}
impl Default for CompileState {
    fn default() -> Self {
        Self {
            registry: Registry::default(),
            file_graph: FileGraph::default(),
            dependency_graph: DependencyGraph::default(),

            symbols: SymbolRegistry::default(),
            symbol_index: SymbolIndex::default(),

            dirty_files: HashSet::new(),
            dirty_symbols: HashSet::new(),
            content_hashes: HashMap::new(),

            source: None,
            ast: None,
            current_ir: None,
            current_lowered_ir: None,
            current_artifact: None,

            caches: CompilerCaches::default(),

            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            ir_version: 1,
        }
    }
}
