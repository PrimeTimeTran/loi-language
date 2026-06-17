use crate::{
    compiler::state::CompileState,
    context::Context,
    frontend::ast::AST,
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
    pub fn compile(&self, global_ctx: &Context) -> Result<(), CompileError> {
        let mut work = PipelineContext::default();
        let mut state = CompileState::default();

        for pipeline in &self.pipelines {
            println!("Running stage: {}", pipeline.name());

            pipeline
                .run(global_ctx, &mut work, &mut state)
                .map_err(|e| CompileError::Stage {
                    stage: pipeline.name().to_string(),
                    source: Box::new(e),
                })?;
        }
        Ok(())
    }

    pub fn execute(&self, global_ctx: &Context) -> Result<(), CompileError> {
        let mut work = PipelineContext::default();
        let mut state = CompileState::default();

        for pipeline in &self.pipelines {
            pipeline.run(global_ctx, &mut work, &mut state)?;
        }
        Ok(())
    }
}
