mod common;
use common::TestHarness;
use loi::pipeline::runner::PipelineRunner;

#[test]
fn test_piecemeal() {
    let mut harness = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    let frontend = harness.build_frontend();
    harness.run_stage(frontend).unwrap();

    {
        let state = harness.env.state.read().unwrap();

        assert!(state.ast.is_some());
    }

    let middle = harness.build_middle();
    harness.run_stage(middle).unwrap();

    {
        let state = harness.env.state.read().unwrap();

        assert!(!state.registry.is_empty());
    }

    let backend = harness.build_backend();
    harness.run_stage(backend).unwrap();
}

#[test]
fn test_everything_at_once() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run().unwrap();

    let state = h.env.state.read().unwrap();

    assert!(state.current_artifact().is_some());
}
