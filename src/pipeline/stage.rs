use std::sync::Arc;

///////////////////////////////////////////////////////////////////////////////////////////
// Stage represents the Work. The CompileEngine asks:
// "What is the name of the stage you are currently executing?"
// so it can print logs and handle errors.
///////////////////////////////////////////////////////////////////////////////////////////
use crate::{
    compiler::{engine::CompileEngine, env::Env, types::BuildArtifact},
    middle::ir::IR,
    pipeline::{CompileError, backend::BackendPipeline, middle::MiddlePipeline},
};

pub trait Stage: std::fmt::Debug + Send + Sync {
    // fn run(&self) -> Result<(), ()>;
    fn run(&self, engine: &CompileEngine) -> Result<(), CompileError>;
    fn name(&self) -> &str;
}

// trait PipelineStage {
//     type Error;

//     fn run(&mut self, env: Arc<Env>) -> Result<(), Self::Error>;
// }

impl Stage for MiddlePipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }
    fn run(&self, engine: &CompileEngine) -> Result<(), CompileError> {
        println!("MIDDLE START");
        let ast = { engine.state.read().unwrap().ast.clone() }
            .ok_or_else(|| CompileError::Middle("missing AST".into()))?;
        let ir_nodes = self.lower_ast(ast);
        let ir = IR {
            raw: String::new(),
            nodes: ir_nodes,
            symbols: std::collections::HashMap::new(),
            metadata: {
                let mut m = std::collections::HashMap::new();
                m.insert("stage".into(), "middle".into());
                m
            },
        };
        engine.state.write().unwrap().ir_cache.current = Some(ir);
        println!("MIDDLE END");
        Ok(())
    }
}

impl Stage for BackendPipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }
    fn run(&self, engine: &CompileEngine) -> Result<(), CompileError> {
        println!("BACKEND START");

        let ir = { engine.state.read().unwrap().ir_cache.current.clone() }
            .ok_or_else(|| CompileError::Backend("missing IR".into()))?;

        let object = self.codegen(ir)?;

        engine.state.write().unwrap().build_cache.current = Some(BuildArtifact::Object(object));

        println!("BACKEND END");

        Ok(())
    }
}
