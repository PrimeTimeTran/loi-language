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

pub trait CompileLike {
    fn diagnostics(&self) -> &DiagnosticStore;
    fn logger(&self) -> &Logger;
    fn cache(&self) -> &MemoryCache;
}
