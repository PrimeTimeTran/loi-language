use crate::pipeline::Pipeline;

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

pub struct Compiler {
    pipelines: Vec<Box<dyn Pipeline>>,
}

impl Compiler {
    pub fn compile(&self) -> Result<(), Compiler> {
        for pipeline in &self.pipelines {
            println!("Running {}", pipeline.name());
            // pipeline.compile();
        }
        Ok(())
    }
}
