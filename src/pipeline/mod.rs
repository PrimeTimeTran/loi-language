use crate::{
    compiler::{PipelineContext, engine::CompileEngine, state::CompileState},
    context::Context,
    kernel::KernelContext,
    pipeline::{backend::BackendPipeline, frontend::FrontendPipeline, middle::MiddlePipeline},
};

pub mod backend;
pub mod frontend;
pub mod middle;
// pub mod provider;
pub mod runner;
pub mod stage;

///////////////////////////////////////////////////////////////////////////////////////////
// Pipeline represents the Identity. Your GUI, CLI, or
// configuration manager might ask:
// "What are the names of the pipelines you have?"
/// It doesn't care if they are currently executing or what their error results are.
///////////////////////////////////////////////////////////////////////////////////////////
// EXPLANATION:
// Source
//   |
//   v
// Frontend Pipeline
//   - lex
//   - parse
//   - validate
//   - AST
//   |
//   v
// Middle Pipeline
//   - type checking
//   - lowering
//   - optimization
//   - IR
//   |
//   v
// Backend Pipeline
//   - code generation
//   - linking
//   - artifact
// pub trait Pipeline {
//     fn name(&self) -> &str;
//     fn run(
//         &self,
//         ctx: &KernelContext,
//         engine: &CompileEngine,
//         state: &mut CompileState,
//     ) -> Result<(), CompileError>;
// }

pub trait Pipeline {
    fn name(&self) -> &str;

    fn run(
        &self,
        global_ctx: &Context,       // Read-only environment
        work: &mut PipelineContext, // Mutable work-in-progress
        state: &mut CompileState,   // Mutable compiler state
    ) -> Result<(), CompileError>;
    fn setup(&mut self, _state: &mut CompileState) -> Result<(), CompileError> {
        Ok(())
    }
    fn teardown(&mut self, _state: &mut CompileState) -> Result<(), CompileError> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum CompileError {
    Frontend(String),
    Middle(String),
    Backend(String),
    Stage {
        stage: String,
        source: Box<dyn std::error::Error>,
    },
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

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Frontend(e) => write!(f, "Frontend error: {}", e),
            CompileError::Middle(e) => write!(f, "Middle error: {}", e),
            CompileError::Backend(e) => write!(f, "Backend error: {}", e),
            CompileError::Stage { stage, source } => {
                write!(f, "Stage error in {}: {}", stage, source)
            }
        }
    }
}

impl std::error::Error for CompileError {}
