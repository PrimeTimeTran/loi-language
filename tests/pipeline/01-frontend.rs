use loi::{frontend::ast::Stmt, pipeline::runner::PipelineRunner};

use crate::common::TestHarness;

#[test]
fn frontend_parse_invalid_input() {
    let mut h = TestHarness::bootstrap("let x = ;", vec![]);

    let result = h.run_stage(h.build_frontend());

    assert!(
        result.is_err(),
        "Invalid input should fail frontend parsing"
    );
}

#[test]
fn frontend_parse_expressions() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2 * 3;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let state = h.env.state.read().unwrap();
    let ast = state.current_ast.as_ref().expect("AST missing");

    let has_expr = ast
        .stmts
        .iter()
        .any(|stmt| matches!(stmt, Stmt::ExprStmt { .. } | Stmt::Let { .. }));

    assert!(has_expr, "Expected expression-based statement in AST");
}

#[test]
fn frontend_parse_statements() {
    let mut h = TestHarness::bootstrap("print 1; let x = 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let state = h.env.state.read().unwrap();
    let ast = state.current_ast.as_ref().expect("AST missing");

    assert!(ast.stmts.len() >= 2, "Expected multiple statements parsed");
}

#[test]
fn frontend_ast_invariants() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let state = h.env.state.read().unwrap();
    let ast = state.current_ast.as_ref().expect("AST missing");

    assert!(
        !ast.stmts.is_empty(),
        "AST should never be empty after successful parse"
    );

    for stmt in &ast.stmts {
        match stmt {
            Stmt::ExprStmt { .. } | Stmt::Let { .. } | Stmt::Print { .. } => {}
            _ => panic!("Unexpected statement variant in AST"),
        }
    }
}

#[test]
fn frontend_always_sets_ast() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let state = h.env.state.read().unwrap();

    assert!(
        state.current_ast.is_some(),
        "Frontend must always write AST to state"
    );
}

#[test]
fn frontend_ast_never_cleared_on_success() {
    let mut h = TestHarness::bootstrap("let x = 1;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let state = h.env.state.read().unwrap();
    assert!(
        state.current_ast.is_some(),
        "AST should persist after successful frontend run"
    );
}

#[test]
fn frontend_ast_changes_with_input() {
    let mut h1 = TestHarness::bootstrap("let x = 1;", vec![]);
    let mut h2 = TestHarness::bootstrap("let x = 2;", vec![]);

    h1.run_stage(h1.build_frontend()).unwrap();
    h2.run_stage(h2.build_frontend()).unwrap();

    let ast1 = h1.env.state.read().unwrap().current_ast.clone().unwrap();
    let ast2 = h2.env.state.read().unwrap().current_ast.clone().unwrap();

    assert_ne!(ast1, ast2, "Different inputs must produce different ASTs");
}

#[test]
fn frontend_ast_is_stable_on_re_run() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    let first = h.env.state.read().unwrap().current_ast.clone();

    h.run_stage(h.build_frontend()).unwrap();
    let second = h.env.state.read().unwrap().current_ast.clone();

    assert_eq!(
        first, second,
        "Frontend should be deterministic across repeated runs"
    );
}

#[test]
fn frontend_ast_has_valid_structure() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2; print x;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let state = h.env.state.read().unwrap();
    let ast = state.current_ast.as_ref().unwrap();

    for stmt in &ast.stmts {
        match stmt {
            Stmt::Let { .. } | Stmt::Print { .. } | Stmt::ExprStmt { .. } => {}
            _ => panic!("Invalid AST node produced by frontend"),
        }
    }
}

#[test]
fn frontend_invalid_input_does_not_write_ast() {
    let mut h = TestHarness::bootstrap("let x = ;", vec![]);

    let result = h.run_stage(h.build_frontend());

    assert!(result.is_err(), "Frontend should fail on invalid input");

    let state = h.env.state.read().unwrap();

    assert!(
        state.current_ast.is_none(),
        "AST must not be written on failed frontend"
    );
}

#[test]
fn frontend_ast_write_is_atomic() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2 * 3;", vec![]);

    let result = h.run_stage(h.build_frontend());

    if result.is_ok() {
        let state = h.env.state.read().unwrap();

        assert!(
            state.current_ast.is_some(),
            "AST must be fully written or not written at all"
        );
    }
}
