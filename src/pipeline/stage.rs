use std::sync::Arc;

///////////////////////////////////////////////////////////////////////////////////////////
// Stage represents the Work. The CompileEngine asks:
// "What is the name of the stage you are currently executing?"
// so it can print logs and handle errors.
///////////////////////////////////////////////////////////////////////////////////////////
use crate::{
    compiler::{engine::CompileEngine, env::Env, types::BuildArtifact},
    middle::ir::IR,
    pipeline::{
        CompileError, backend::BackendPipeline, frontend::FrontendPipeline, middle::MiddlePipeline,
    },
};

pub trait Stage: std::fmt::Debug + Send + Sync {
    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError>;
    fn name(&self) -> &str;
}

impl Stage for FrontendPipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError> {
        log_stage!("FRONTEND", "start");
        let ast = self.perform_compilation()?;
        {
            let mut state = engine.state.write().unwrap();
            state.current_ast = Some(ast);
        }
        log_stage!("FRONTEND", "AST written to state");
        Ok(())
    }
}

impl Stage for MiddlePipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError> {
        log_stage!("MIDDLE", "start");

        // 1. READ AST FROM ENGINE
        let ast = {
            let state = engine.state.read().unwrap();
            state.current_ast()
        }
        .ok_or_else(|| CompileError::Middle("missing AST".into()))?;

        log_stage!("MIDDLE", "AST loaded");

        // 2. SINGLE ENTRY POINT: pipeline owns lowering
        let ir_nodes = self.lower_ast(ast)?;

        log_stage!("MIDDLE", "lowering complete");

        // 3. BUILD IR
        let ir = IR::new_from_ops(ir_nodes).with_stage("middle");

        // 4. WRITE BACK
        {
            let mut state = engine.state.write().unwrap();
            state.current_ir = Some(ir);
        }

        log_stage!("MIDDLE", "IR written");
        Ok(())
    }
}
impl Stage for BackendPipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError> {
        log_stage!("BACKEND", "start");

        let ir = { engine.state.read().unwrap().current_ir.clone() }
            .ok_or_else(|| CompileError::Backend("missing IR".into()))?;

        let object = self.codegen(ir)?;

        let mut state = engine.state.write().unwrap();

        // ✔ THIS is what your test expects
        state.current_artifact = Some(BuildArtifact::Object(object));

        log_stage!("BACKEND", "end");
        Ok(())
    }
}
