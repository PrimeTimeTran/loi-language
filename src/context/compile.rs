use crate::compiler::{
    cache::MemoryCache,
    diagnostic::{DiagnosticStore, Logger},
};
use crate::compiler::{config::Config, env::Env, state::CompileState};

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use thiserror::Error;

pub struct CompileContext {
    pub env: Env,
    pub config: Config,
    pub state: CompileState,
    pub diagnostics: DiagnosticStore,
    pub cache: MemoryCache,
}

impl CompileContext {
    pub fn new() -> Self {
        Self {
            env: Env::default(),
            config: Config::default(),
            state: CompileState::default(),
            diagnostics: DiagnosticStore::default(),
            cache: MemoryCache::new(),
        }
    }
}

impl Default for CompileContext {
    fn default() -> Self {
        Self::new()
    }
}

pub trait CompileLike {
    fn diagnostics(&self) -> &DiagnosticStore;
    fn logger(&self) -> &Logger;
    fn cache(&self) -> &MemoryCache;
}

// "What"
// It represents the snapshot of the world at a given point in time.
// It holds the data, the state of the compilation, and the
// references to files. It should be "dumb" data—things you pass down
// into your functions to provide the environment needed to compute a result.
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub env: Env,
    pub config: Config,
    pub diagnostics: Arc<RwLock<DiagnosticStore>>,
    pub cache: MemoryCache,
}

impl Context {
    pub fn new() -> Self {
        Self {
            env: Env::default(),
            config: Config::default(),
            cache: MemoryCache::new(),
            diagnostics: Arc::new(RwLock::new(DiagnosticStore::default())),
        }
    }
}
pub trait ContextLike {
    fn diagnostics(&self) -> &DiagnosticStore;
    fn logger(&self) -> &Logger;
    fn cache(&self) -> &MemoryCache;
}
