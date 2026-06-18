use crate::{
    compiler::state::CompileState,
    frontend::ast::AST,
    kernel::KernelContext,
    middle::ir::IROp,
    pipeline::{CompileError, Pipeline},
};

pub mod addon;
pub mod bundler;
pub mod cache;
pub mod compile;
pub mod compile_project;
pub mod config;
pub mod diagnostic;
pub mod engine;
pub mod env;
pub mod error;
pub mod execution;
pub mod runtime;
pub mod safety;
pub mod scale;
pub mod state;
pub mod types;

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
