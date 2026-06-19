///////////////////////////////////////////////////////////////////////////////////////////
// Stage represents the Work. The CompileEngine asks:
// "What is the name of the stage you are currently executing?"
// so it can print logs and handle errors.
///////////////////////////////////////////////////////////////////////////////////////////
use crate::{
    compiler::{engine::CompileEngine, env::Env, types::BuildArtifact},
    middle::ir::IR,
    pipeline::{
        CompileError,
        backend::BackendPipeline,
        frontend::FrontendPipeline,
        middle::{MiddleLoweringLogic, MiddlePipeline, SymbolInfo},
    },
};
use std::{any::Any, collections::HashMap, sync::Arc};

pub trait Stage: std::fmt::Debug + Send + Sync {
    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError>;
    fn name(&self) -> &str;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// --- FRONTEND PIPELINE ---
impl Stage for FrontendPipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError> {
        log_stage!("FRONTEND", "starting...");

        // Pass the tools into the stages if they need them
        for pass in &mut self.passes {
            pass.run(engine)?;
        }

        Ok(())
    }
}
// --- MIDDLE PIPELINE ---

impl Stage for MiddlePipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError> {
        log_stage!("MIDDLE", "starting...");

        {
            let state = engine.state.read().unwrap();
            if state.current_ast.is_none() {
                return Err(CompileError::Middle("AST missing for middle-end".into()));
            }
        }

        for pass in &mut self.passes {
            log_stage!("MIDDLE", "running: {}", pass.name());

            // CRITICAL: We must downcast to see if it's the LoweringStage
            if let Some(lowering_pass) = pass.as_any_mut().downcast_mut::<LoweringStage>() {
                // If it is, we invoke the special pipeline-aware execution
                lowering_pass.run_with_pipeline(engine, &mut self.symbols, &self.temp_counter)?;
            } else {
                // Otherwise, use standard run()
                pass.run(engine)?;
            }
        }

        log_stage!("MIDDLE", "complete");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LoweringStage {
    pub name: String,
    pub logic: Arc<MiddleLoweringLogic>,
}

impl Stage for LoweringStage {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError> {
        Err(CompileError::Middle(
            "LoweringStage must be executed via run_with_pipeline".into(),
        ))
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Keep this as a separate inherent method, not part of the Stage trait
impl LoweringStage {
    pub fn run_with_pipeline(
        &mut self,
        engine: &CompileEngine,
        symbols: &mut HashMap<String, SymbolInfo>,
        counter: &std::sync::atomic::AtomicUsize,
    ) -> Result<(), CompileError> {
        self.logic.execute(engine, symbols, counter)
    }
}

// --- BACKEND PIPELINE ---
impl Stage for BackendPipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn run(&mut self, engine: &CompileEngine) -> Result<(), CompileError> {
        log_stage!("BACKEND", "starting...");

        // 1. Orchestrate internal backend passes
        for pass in &mut self.passes {
            log_stage!("BACKEND", "running: {}", pass.name());
            pass.run(engine)?;
        }

        // 2. Final Codegen (The terminal stage of the backend)
        let ir = { engine.state.read().unwrap().current_ir.clone() }
            .ok_or_else(|| CompileError::Backend("missing IR".into()))?;

        let object = self.codegen(ir)?;

        // 3. Write final artifact
        engine.state.write().unwrap().current_artifact = Some(BuildArtifact::Object(object));

        log_stage!("BACKEND", "complete");
        Ok(())
    }
}
