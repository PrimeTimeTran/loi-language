use std::sync::{Arc, RwLock};

use crate::{
    compiler::{
        PipelineContext,
        cache::MemoryCache,
        diagnostic::{DiagnosticStore, Logger},
        engine::CompileEngine,
        execution::{JobQueue, TaskScheduler},
        state::CompileState,
    },
    context::Context,
    pipeline::{CompileError, Pipeline},
};

// "How": It represents the execution machinery. It holds the long-lived
// services, worker threads, and event loops that do work.
// It is the "root" of your application that coordinates the Context
// to achieve a goal.

#[derive(Debug, Clone)]
pub struct KernelContext {
    pub context: Arc<Context>,
    pub logger: Arc<Logger>,
    pub cache: Arc<MemoryCache>,
    pub job_queue: Arc<JobQueue>,
    pub scheduler: Arc<TaskScheduler>,
    pub diagnostics: Arc<RwLock<DiagnosticStore>>,
}

#[derive(Debug)]
pub struct Kernel {
    pub kernel_ctx: KernelContext,
    pub engine: Arc<CompileEngine>,
}

// 1. The Kernel provides the Services
impl Kernel {
    pub fn run_pipeline(&self, pipeline: &mut dyn Pipeline) -> Result<(), CompileError> {
        let mut work = PipelineContext::default();
        let mut state = CompileState::default();

        pipeline
            .setup(&mut state)
            .map_err(|e| CompileError::Stage {
                stage: format!("{}: setup", pipeline.name()),
                source: Box::new(e),
            })?;

        // Pass the internal KernelContext
        pipeline
            .run(&self.kernel_ctx, &mut work, &mut state)
            .map_err(|e| CompileError::Stage {
                stage: pipeline.name().to_string(),
                source: Box::new(e),
            })?;

        pipeline
            .teardown(&mut state)
            .map_err(|e| CompileError::Stage {
                stage: format!("{}: teardown", pipeline.name()),
                source: Box::new(e),
            })?;

        Ok(())
    }
}

#[derive(Default)]
pub struct KernelBuilder {
    context: Option<Arc<Context>>,
    engine: Option<Arc<CompileEngine>>,
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

    pub fn engine(mut self, engine: Arc<CompileEngine>) -> Self {
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

        let kernel_ctx = KernelContext {
            context,
            logger: self.logger.unwrap_or_else(|| Arc::new(Logger::default())),
            cache: self.cache.unwrap_or_else(|| Arc::new(MemoryCache::new())),
            job_queue: self
                .job_queue
                .unwrap_or_else(|| Arc::new(JobQueue::default())),
            // FIX: Wrap the scheduler in Arc
            scheduler: self
                .scheduler
                .map(Arc::new)
                .unwrap_or_else(|| Arc::new(TaskScheduler::default())),
            diagnostics: self
                .diagnostics
                .unwrap_or_else(|| Arc::new(RwLock::new(DiagnosticStore::default()))),
        };

        Kernel { kernel_ctx, engine }
    }
}
