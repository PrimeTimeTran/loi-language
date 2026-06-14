///////////////////////////////////////////////////////////////////////////////////////////
// Stage represents the Work. The CompileEngine asks:
// "What is the name of the stage you are currently executing?"
// so it can print logs and handle errors.
///////////////////////////////////////////////////////////////////////////////////////////
use crate::pipeline::{backend::BackendPipeline, middle::MiddlePipeline};

pub trait Stage: Send + Sync {
    fn run(&self) -> Result<(), String>;
    fn name(&self) -> &str;
}

impl Stage for MiddlePipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }
    fn run(&self) -> Result<(), String> {
        // 1. Access the shared state
        // let state = self.state.read().map_err(|e| e.to_string())?;

        // // 2. Ensure the previous stage finished successfully
        // let ast = state
        //     .ast
        //     .as_ref()
        //     .ok_or("No AST found - did Frontend fail?")?;

        // // 3. Perform middle-end work
        // println!("MiddlePipeline: Optimizing AST...");
        // let ir = self.generate_ir(ast); // Your internal logic

        // // 4. Update the shared state with the result
        // let mut state = self.state.write().map_err(|e| e.to_string())?;
        // state.ir = Some(ir);

        Ok(())
    }
}

impl Stage for BackendPipeline {
    fn name(&self) -> &str {
        &self.metadata.name
    }
    fn run(&self) -> Result<(), String> {
        // // 1. Access shared state
        // let state = self.state.read().map_err(|e| e.to_string())?;

        // // 2. Ensure IR exists
        // let ir = state
        //     .ir
        //     .as_ref()
        //     .ok_or("No IR found - did Middle stage fail?")?;

        // // 3. Perform backend generation
        // println!("BackendPipeline: Generating code...");
        // let binary = self
        //     .codegen(ir)
        //     .map_err(|e| format!("Codegen error: {}", e))?;

        // // 4. Finalize state
        // let mut state = self.state.write().map_err(|e| e.to_string())?;
        // state.output = Some(binary);

        Ok(())
    }
}
