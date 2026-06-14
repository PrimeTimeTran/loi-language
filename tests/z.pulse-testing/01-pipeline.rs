use crate::common::{PipelineTarget, TestHarness};

#[test]
fn test_piecemeal() {
    // let harness = TestHarness::new()
    //     .with_source("let x = 10;")
    //     .with_symbol("x", "10", "main.loi");
    // let p = harness.build_frontend();
    // harness.run_stage(p).expect("Pipeline failed");
    // let syms = harness.run_incremental();
    // harness.assert_symbol_exists(&syms, "x", "main.loi");
    let mut h = TestHarness::new().with_source("foo");
    h.run(PipelineTarget::Frontend).unwrap();
    let ast = h.get_ast().unwrap();
}

#[test]
fn test_everything_at_once() {
    // let syms = TestHarness::bootstrap("let x = 10;", vec![("x", "10", "main.loi")])
    //     .run_full_suite()
    //     .expect("Full suite failed");

    // assert!(syms.lookup("x", "main.loi").is_some());

    let mut h = TestHarness::new().with_source("foo");
    h.run(PipelineTarget::Full).unwrap();
    let ast = h.get_ast().unwrap();
}
