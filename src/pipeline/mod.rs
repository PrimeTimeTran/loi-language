///////////////////////////////////////////////////////////////////////////////////////////
// Pipeline represents the Identity. Your GUI, CLI, or
// configuration manager might ask:
// "What are the names of the pipelines you have?"
/// It doesn't care if they are currently executing or what their error results are.
///////////////////////////////////////////////////////////////////////////////////////////
use crate::{
    compiler::{
        context::PipelineContext, engine::CompileEngine, error::CompileError, state::CompileState,
    },
    kernel::{Kernel, KernelContext},
    pipeline::{backend::BackendPipeline, frontend::FrontendPipeline, middle::MiddlePipeline},
};

pub mod backend;
pub mod frontend;
pub mod middle;
pub mod runner;
pub mod stage;

pub trait Pipeline {
    fn name(&self) -> &str;

    fn run(
        &self,
        kernel_ctx: &KernelContext,
        work: &mut PipelineContext,
        state: &mut CompileState,
    ) -> Result<(), CompileError>;
    fn setup(&mut self, _state: &mut CompileState) -> Result<(), CompileError> {
        Ok(())
    }
    fn teardown(&mut self, _state: &mut CompileState) -> Result<(), CompileError> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Metadata {
    pub name: String,
    pub version: String,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            name: "Pipeline: Unnamed".to_string(),
            version: "0.0.1".to_string(),
        }
    }
}
