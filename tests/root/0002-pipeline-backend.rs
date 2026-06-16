mod common {
    include!("../common/mod.rs");
}
use common::TestHarness;
use loi::pipeline::{backend::BackendTarget, runner::PipelineRunner};

#[test]
fn backend_requires_ir() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    assert!(h.engine.state.read().unwrap().current_ir().is_some());

    let result = h.run_stage(h.build_backend());

    assert!(result.is_err(), "Backend without IR should fail");
}

#[test]
fn backend_generates_artifact() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    h.run_stage(h.build_middle()).unwrap();
    h.run_stage(h.build_backend()).unwrap();

    let result = h.run_stage(h.build_backend());
    match result {
        Ok(v) => v,
        Err(e) => panic!("LLVM backend failed: {:?}", e),
    }

    let state = h.env.state.read().unwrap();
    assert!(
        state.current_artifact().is_some(),
        "Backend must produce artifact"
    );
}

#[test]
fn backend_target_selection() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    h.run_stage(h.build_middle()).unwrap();

    let backend = h.build_backend().with_target(BackendTarget::LLVM);
    let result = h.run_stage(backend);

    assert!(result.is_ok(), "LLVM backend should succeed");
}

#[test]
fn backend_determinism() {
    let mut h1 = TestHarness::bootstrap("let x = 1 + 2;", vec![]);
    let mut h2 = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    for h in [&mut h1, &mut h2] {
        h.run_stage(h.build_frontend()).unwrap();
        h.run_stage(h.build_middle()).unwrap();
        h.run_stage(h.build_backend()).unwrap();
    }

    let a1 = h1.env.state.read().unwrap().current_artifact().unwrap();
    let a2 = h2.env.state.read().unwrap().current_artifact().unwrap();

    assert_eq!(a1, a2, "Backend must be deterministic");
}
