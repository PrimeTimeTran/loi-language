use std::{path::PathBuf, sync::Arc};

use crate::{
    compiler::{
        cache::MemoryCache,
        diagnostic::{DiagnosticStore, Logger},
        engine::CompileEngine,
    },
    context::{CompileContext, Kernel},
};

use crate::compiler::{config::Config, env::Env, state::CompileState};
use std::sync::RwLock;
use thiserror::Error;

// "What"
// It represents the snapshot of the world at a given point in time.
// It holds the data, the state of the compilation, and the
// references to files. It should be "dumb" data—things you pass down
// into your functions to provide the environment needed to compute a result.
#[derive(Clone, Default)]
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
