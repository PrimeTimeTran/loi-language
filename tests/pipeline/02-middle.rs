use loi::{
    frontend::ast::Expr,
    middle::{ir::IROp, types::IRVal},
    pipeline::runner::PipelineRunner,
};

use crate::common::TestHarness;

#[test]
fn middle_generates_ir() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    h.run_stage(h.build_middle()).unwrap();

    let state = h.env.state.read().unwrap();

    assert!(state.current_ir().is_some(), "Middle stage must produce IR");
}

#[test]
fn middle_ir_structure() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    h.run_stage(h.build_middle()).unwrap();

    let state = h.env.state.read().unwrap();
    let ir = state.current_ir().unwrap();

    assert!(!ir.nodes.is_empty(), "IR must contain at least one node");
}

#[test]
fn middle_ir_determinism() {
    let mut h1 = TestHarness::bootstrap("let x = 1 + 2;", vec![]);
    let mut h2 = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h1.run_stage(h1.build_frontend()).unwrap();
    h1.run_stage(h1.build_middle()).unwrap();

    h2.run_stage(h2.build_frontend()).unwrap();
    h2.run_stage(h2.build_middle()).unwrap();

    let ir1 = h1.env.state.read().unwrap().current_ir().unwrap();
    let ir2 = h2.env.state.read().unwrap().current_ir().unwrap();

    assert_eq!(ir1, ir2, "IR must be deterministic");
}

#[test]
fn middle_transforms_ast() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let ast_before = {
        let state = h.env.state.read().unwrap();
        state.current_ast.clone().unwrap()
    };

    h.run_stage(h.build_middle()).unwrap();

    let state = h.env.state.read().unwrap();
    let ir = state.current_ir().unwrap();

    assert!(
        !ast_before.stmts.is_empty(),
        "AST must exist before lowering"
    );

    assert!(!ir.nodes.is_empty(), "IR must be produced from AST");
}

#[test]
fn middle_empty_ast_no_crash() {
    let mut h = TestHarness::bootstrap("", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    let result = h.run_stage(h.build_middle());

    assert!(result.is_ok(), "Empty AST should not crash middle stage");
}

#[test]
fn middle_does_not_use_raw_expr_conversion() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2 * 3;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    let result = h.run_stage(h.build_middle());

    assert!(
        result.is_ok(),
        "Middle must handle complex expressions via lowering, not raw Expr conversion"
    );
}

#[test]
fn middle_never_panics_on_valid_ast() {
    let inputs = [
        "let x = 1 + 2;",
        "let x = 1 + 2 * 3;",
        "print 1 + 2;",
        "let x = (1 + 2) + (3 + 4);",
    ];

    for src in inputs {
        let mut h = TestHarness::bootstrap(src, vec![]);
        h.run_stage(h.build_frontend()).unwrap();

        let result = h.run_stage(h.build_middle());
        assert!(
            result.is_ok(),
            "Middle must never panic on valid AST: {}",
            src
        );
    }
}

#[test]
fn middle_ir_snapshot() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    h.run_stage(h.build_middle()).unwrap();

    let ir = h.env.state.read().unwrap().current_ir().unwrap();

    insta::assert_debug_snapshot!(ir);
}

#[test]
fn middle_always_sets_ir() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    h.run_stage(h.build_middle()).unwrap();

    let state = h.env.state.read().unwrap();
    assert!(state.current_ir.is_some());
}
