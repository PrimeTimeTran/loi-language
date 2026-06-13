use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use anyhow::Error;

use crate::{
    compiler::{
        cache::MemoryCache,
        diagnostic::{DiagnosticStore, Logger},
        engine::CompileEngine,
        execution::{JobQueue, TaskScheduler},
    },
    context::Context,
    interface::{CompileEngineProvider, FileSystemProvider},
};

// "How": It represents the execution machinery. It holds the long-lived
// services, worker threads, and event loops that do work.
// It is the "root" of your application that coordinates the Context
// to achieve a goal.
pub struct Kernel {
    pub context: Arc<Context>,
    pub engine: CompileEngine,
    pub logger: Arc<Logger>,
    pub cache: Arc<MemoryCache>,
    pub job_queue: Arc<JobQueue>,
    pub scheduler: TaskScheduler,
    pub diagnostics: Arc<RwLock<DiagnosticStore>>,
}

impl Kernel {
    pub fn new(context: Arc<Context>, engine: CompileEngine) -> Self {
        Self {
            context,
            engine,
            logger: Arc::new(Logger::default()),
            cache: Arc::new(MemoryCache::new()),
            job_queue: Arc::new(JobQueue::default()),
            scheduler: TaskScheduler::default(),
            diagnostics: Arc::new(RwLock::new(DiagnosticStore::default())),
        }
    }
}
