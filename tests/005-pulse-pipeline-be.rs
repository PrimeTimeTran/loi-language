use loi::{compiler::state::CompileState, context::Context, pipeline::frontend::FrontendPipeline};
use std::sync::{Arc, RwLock};

use common::harness::TestHarness;

#[test]
fn test_frontend_pipeline() {
    let harness = TestHarness::new();
    harness.load_source("your code here".to_string());

    // Construct the specific pipeline
    let pipeline = FrontendPipeline::new(
        harness.env.context.clone(),
        harness.env.config.clone(),
        harness.env.state.clone(),
    );

    // Run it using the generic harness method
    harness.run_stage(pipeline).expect("Frontend failed");
}
