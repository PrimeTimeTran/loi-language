use std::sync::Arc;

///////////////////////////////////////////////////////////////////////////////////////////
// Stage represents the Work. The CompileEngine asks:
// "What is the name of the stage you are currently executing?"
// so it can print logs and handle errors.
///////////////////////////////////////////////////////////////////////////////////////////
use crate::{
    backend::llvm::lower_ast_to_ir,
    compiler::{engine::CompileEngine, env::Env, types::BuildArtifact},
    middle::ir::IR,
    pipeline::{
        CompileError, backend::BackendPipeline, frontend::FrontendPipeline, middle::MiddlePipeline,
    },
};

pub trait Stage: std::fmt::Debug + Send + Sync {
    fn run(&self, engine: &CompileEngine) -> Result<(), CompileError>;
    fn name(&self) -> &str;
}

impl Stage for FrontendPipeline {
    // fn run(&self, engine: &CompileEngine) -> Result<(), CompileError> {
    //     let result = self.perform_compilation();
    //     match result {
    //         Ok(ast) => {
    //             let mut state = self.state.write().unwrap();
    //             state.ast = Some(ast);
    //             let ir = lower_ast_to_ir(state.ast.as_ref().unwrap())?;
    //             state.current_ir = Some(ir);

    //             println!("✅ FINAL AST WRITTEN");
    //             println!("${:?}", state.ast);
    //             println!("🧠 IR WRITTEN");

    //             Ok(())
    //         }
    //         Err(diags) => {
    //             let state = self.state.read().unwrap();
    //             println!("❌ ERROR PATH AST = {:?}", state.ast);
    //             {
    //                 let mut global =
    //                     self.context.diagnostics.write().map_err(|_| {
    //                         CompileError::Frontend("failed to lock diagnostics".into())
    //                     })?;

    //                 for diag in diags.diagnostics {
    //                     global.emit(diag);
    //                 }
    //             }

    //             // optional debug fallback (no-op unless you want logging)
    //             if let Ok(state) = self.state.write() {
    //                 if state.ast.is_none() {
    //                     println!("⚠️ AST is missing after frontend failure");
    //                 }
    //             }

    //             Err(CompileError::Frontend("failure in AST".into()))
    //         }
    //     }
    // }
    fn name(&self) -> &str {
        &self.metadata.name
    }
    fn run(&self, engine: &CompileEngine) -> Result<(), CompileError> {
        println!("FRONTEND START");

        let ast = self.perform_compilation()?;

        {
            let mut state = engine.state.write().unwrap();
            state.ast = Some(ast);
        }

        println!("✅ AST WRITTEN");
        println!("FRONTEND END");

        Ok(())
    }
}

impl Stage for MiddlePipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn run(&self, engine: &CompileEngine) -> Result<(), CompileError> {
        let ast = {
            let state = engine.state.read().unwrap();
            state.ast.clone()
        }
        .ok_or_else(|| CompileError::Middle("missing AST".into()))?;
        let ir_nodes = self.lower_ast(ast);
        let ir = IR::new_from_ops(ir_nodes).with_stage("middle");
        {
            let mut state = engine.state.write().unwrap();
            state.current_ir = Some(ir);
        }
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
        let ir = { engine.state.read().unwrap().current_ir.clone() }
            .ok_or_else(|| CompileError::Backend("missing IR".into()))?;
        let object = self.codegen(ir)?;
        engine.state.write().unwrap().caches.build.current = Some(BuildArtifact::Object(object));
        println!("BACKEND END");
        Ok(())
    }
}
