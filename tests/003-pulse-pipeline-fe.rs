use loi::pipeline::frontend::FrontendPipeline;
mod common;
use common::harness::TestHarness;

#[test]
fn test_frontend_pipeline() {
    // 1. Initialize, Configure, and Build in one fluent chain
    let harness = TestHarness::new().with_source("your code here");

    // 2. Build the pipeline using the harness's helper
    // This is cleaner than manual cloning of context/config/state
    let pipeline = harness.build_frontend();

    // 3. Run it
    // Note: run_stage consumes harness, so we ignore the return
    // unless we need to inspect the harness afterwards.
    harness.run_stage(pipeline).expect("Frontend failed");
}
