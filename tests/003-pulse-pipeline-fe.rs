use loi::pipeline::frontend::FrontendPipeline;
mod common;
use common::harness::TestHarness;

#[test]
fn test_frontend_pipeline_only() {
    let mut harness = TestHarness::new().with_source("let x = 10;");

    let pipeline = harness.build_frontend();

    // run only frontend stage
    harness
        .run_stage(pipeline)
        .expect("Frontend pipeline failed");

    // verify AST was produced
    let ast = harness
        .get_ast()
        .expect("AST should exist after frontend pipeline");

    // optional debug
    println!("AST: {:#?}", ast);

    // optional sanity check
    assert!(!ast.stmts.is_empty(), "AST should not be empty");
}
