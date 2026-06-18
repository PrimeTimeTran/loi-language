use std::sync::{Arc, RwLock};

use crate::{
    compiler::{
        cache::MemoryCache, config::Config, diagnostic::DiagnosticStore, env::Env,
        error::CompileError, state::CompileState,
    },
    frontend::ast::AST,
    kernel::KernelContext,
    middle::ir::IROp,
    pipeline::Pipeline,
};

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

#[derive(Default)]
pub struct PipelineContext {
    pub ast: Option<AST>,
    pub ir: Option<Vec<IROp>>,
    pub binary: Option<Vec<u8>>,
}

pub struct Compiler {
    pipelines: Vec<Box<dyn Pipeline>>,
}

impl Compiler {
    pub fn compile(&mut self, kernel: &KernelContext) -> Result<(), CompileError> {
        let mut work = PipelineContext::default();
        let mut state = CompileState::default();
        for pipeline in &mut self.pipelines {
            let name = pipeline.name().to_string();
            // These methods now work because pipeline is &mut
            pipeline
                .setup(&mut state)
                .map_err(|e| CompileError::Stage {
                    stage: format!("{}: setup", name),
                    source: Box::new(e),
                })?;

            // Execution
            pipeline
                .run(kernel, &mut work, &mut state)
                .map_err(|e| CompileError::Stage {
                    stage: name.clone(),
                    source: Box::new(e),
                })?;

            // Lifecycle: Teardown
            pipeline
                .teardown(&mut state)
                .map_err(|e| CompileError::Stage {
                    stage: format!("{}: teardown", name),
                    source: Box::new(e),
                })?;
        }
        Ok(())
    }
}
