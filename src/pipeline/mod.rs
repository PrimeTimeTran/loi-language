///////////////////////////////////////////////////////////////////////////////////////////
// Pipeline represents the Identity. Your GUI, CLI, or
// configuration manager might ask:
// "What are the names of the pipelines you have?"
/// It doesn't care if they are currently executing or what their error results are.
///////////////////////////////////////////////////////////////////////////////////////////
use crate::pipeline::{
    backend::BackendPipeline, frontend::FrontendPipeline, middle::MiddlePipeline,
};

pub mod backend;
pub mod frontend;
pub mod middle;
pub mod original;
pub mod provider;
pub mod stage;

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

pub trait Pipeline {
    fn compile(&self);
    fn name(&self) -> &str;
}
