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
#[derive(Debug)]
pub struct Kernel {
    pub context: Arc<Context>,
    pub engine: CompileEngine,
    pub logger: Arc<Logger>,
    pub cache: Arc<MemoryCache>,
    pub job_queue: Arc<JobQueue>,
    pub scheduler: TaskScheduler,
    pub diagnostics: Arc<RwLock<DiagnosticStore>>,
}

pub struct KernelBuilder {
    context: Option<Arc<Context>>,
    engine: Option<CompileEngine>,
    logger: Option<Arc<Logger>>,
    cache: Option<Arc<MemoryCache>>,
    job_queue: Option<Arc<JobQueue>>,
    scheduler: Option<TaskScheduler>,
    diagnostics: Option<Arc<RwLock<DiagnosticStore>>>,
}

impl KernelBuilder {
    pub fn new() -> Self {
        Self {
            context: None,
            engine: None,
            logger: None,
            cache: None,
            job_queue: None,
            scheduler: None,
            diagnostics: None,
        }
    }

    pub fn context(mut self, context: Arc<Context>) -> Self {
        self.context = Some(context);
        self
    }

    pub fn engine(mut self, engine: CompileEngine) -> Self {
        self.engine = Some(engine);
        self
    }

    pub fn logger(mut self, logger: Arc<Logger>) -> Self {
        self.logger = Some(logger);
        self
    }

    pub fn cache(mut self, cache: Arc<MemoryCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn job_queue(mut self, queue: Arc<JobQueue>) -> Self {
        self.job_queue = Some(queue);
        self
    }

    pub fn scheduler(mut self, scheduler: TaskScheduler) -> Self {
        self.scheduler = Some(scheduler);
        self
    }

    pub fn diagnostics(mut self, diagnostics: Arc<RwLock<DiagnosticStore>>) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    pub fn build(self) -> Kernel {
        let context = self.context.expect("Kernel requires Context");
        let engine = self.engine.expect("Kernel requires CompileEngine");

        Kernel {
            context,
            engine,
            logger: self.logger.unwrap_or_else(|| Arc::new(Logger::default())),
            cache: self.cache.unwrap_or_else(|| Arc::new(MemoryCache::new())),
            job_queue: self
                .job_queue
                .unwrap_or_else(|| Arc::new(JobQueue::default())),
            scheduler: self.scheduler.unwrap_or_default(),
            diagnostics: self
                .diagnostics
                .unwrap_or_else(|| Arc::new(RwLock::new(DiagnosticStore::default()))),
        }
    }
}
