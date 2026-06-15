use crate::{
    compiler::engine::CompileEngine,
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
pub trait Pipeline {
    fn compile(&self, engine: &CompileEngine) -> Result<(), CompileError>;
    fn name(&self) -> &str;
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

#[derive(Clone, Debug)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    // Add other shared things here like version, status, etc.
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            name: "Pipeline: Unnamed".to_string(),
            version: "0.0.1".to_string(),
        }
    }
}
