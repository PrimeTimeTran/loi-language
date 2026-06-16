use loi::pipeline::frontend::FrontendPipeline;
mod common {
    include!("../00_common/mod.rs");
}
use common::harness::TestHarness;

#[test]
fn test_middle_pipeline() {
    // 1. Initialize + seed source (if needed for IR generation)
    let mut harness = TestHarness::new().with_source("let x = 10;");

    // 2. Run frontend first (middle depends on AST/state)
    let frontend = harness.build_frontend();
    harness.run_stage(frontend).expect("Frontend failed");

    // 3. Build middle pipeline
    let middle = harness.build_middle();

    // 4. Run middle stage only
    harness.run_stage(middle).expect("Middle pipeline failed");

    // 5. Inspect state (IR / transformed AST depending on your design)
    let ast = harness
        .get_ast()
        .expect("AST should exist after middle pipeline");

    let diagnostics = harness.get_diagnostics();

    println!("AST after middle: {:#?}", ast);
    println!("Diagnostics: {:#?}", diagnostics);

    assert_eq!(diagnostics.error_count, 0);
}
