use loi::pipeline::frontend::FrontendPipeline;
mod common;
use common::harness::TestHarness;

#[test]
fn test_backend_pipeline() {
    // 1. Setup + source
    let mut harness = TestHarness::new().with_source("let x = 10;");

    // 2. Frontend must run first (produces AST)
    let frontend = harness.build_frontend();
    harness.run_stage(frontend).expect("Frontend failed");

    // 3. Middle must run next (IR / semantic layer)
    let middle = harness.build_middle();
    harness.run_stage(middle).expect("Middle failed");

    // 4. Build backend pipeline
    let backend = harness.build_backend();

    // 5. Run backend only
    harness.run_stage(backend).expect("Backend failed");

    // 6. Validate final state
    let ast = harness.get_ast().expect("AST should exist after backend");

    let diagnostics = harness.get_diagnostics();

    println!("Final AST: {:#?}", ast);
    println!("Diagnostics: {:#?}", diagnostics);

    assert_eq!(diagnostics.error_count, 0);
}
