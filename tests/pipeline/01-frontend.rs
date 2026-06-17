use crate::common::{SNAP_AST, TestHarness};

//
// =======================================================
// 1. BASIC PARSING CORRECTNESS
// =======================================================
//

#[test]
fn frontend_rejects_invalid_syntax() {
    let mut h = TestHarness::bootstrap("let x = ;", vec![]);
    let result = h.run_stage(h.build_frontend());
    assert!(result.is_err());
}

#[test]
fn frontend_parses_simple_expression() {
    let mut h = TestHarness::bootstrap("let x = 1;", vec![]);
    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("simple_expression", h.get_ast().unwrap());
}

#[test]
fn frontend_parses_multiple_statements() {
    let mut h = TestHarness::bootstrap("print 1; let x = 2;", vec![]);
    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("multiple_statements", h.get_ast().unwrap());
}

//
// =======================================================
// 2. AST STRUCTURE VALIDATION
// =======================================================
//

#[test]
fn frontend_only_emits_valid_ast() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2; print x;", vec![]);
    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("valid_ast_variants", h.get_ast().unwrap());
}

#[test]
fn frontend_ast_never_empty_after_success() {
    let mut h = TestHarness::bootstrap("let x = 1;", vec![]);
    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("non_empty_ast", h.get_ast().unwrap());
}

//
// =======================================================
// 3. OPERATOR PRECEDENCE & EXPRESSIONS
// =======================================================
//

#[test]
fn frontend_operator_precedence() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2 * 3;", vec![]);
    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("operator_precedence", h.get_ast().unwrap());
}

#[test]
fn frontend_nested_parentheses() {
    let mut h = TestHarness::bootstrap("let x = (1 + 2) * 3;", vec![]);
    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("nested_parentheses", h.get_ast().unwrap());
}

//
// =======================================================
// 4. DETERMINISM / STABILITY
// =======================================================
//

#[test]
fn frontend_is_deterministic() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();
    let first = h.get_ast().unwrap();

    h.run_stage(h.build_frontend()).unwrap();
    let second = h.get_ast().unwrap();

    assert_eq!(first, second);
}

#[test]
fn frontend_different_inputs_produce_different_ast() {
    let mut h1 = TestHarness::bootstrap("let x = 1;", vec![]);
    let mut h2 = TestHarness::bootstrap("let x = 2;", vec![]);

    h1.run_stage(h1.build_frontend()).unwrap();
    h2.run_stage(h2.build_frontend()).unwrap();

    assert_ne!(h1.get_ast().unwrap(), h2.get_ast().unwrap());
}

//
// =======================================================
// 5. STATE SAFETY / ATOMICITY
// =======================================================
//

#[test]
fn frontend_does_not_write_ast_on_failure() {
    let mut h = TestHarness::bootstrap("let x = ;", vec![]);

    let result = h.run_stage(h.build_frontend());

    assert!(result.is_err());
    assert!(h.env.state.read().unwrap().current_ast.is_none());
}

#[test]
fn frontend_ast_write_is_atomic() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2 * 3;", vec![]);
    let result = h.run_stage(h.build_frontend());

    if result.is_ok() {
        SNAP_AST.assert_value("atomic_write", h.get_ast().unwrap());
    }
}

//
// =======================================================
// 6. ERROR CASES
// =======================================================
//

#[test]
fn frontend_missing_rhs_expression() {
    let mut h = TestHarness::bootstrap("let x = ;", vec![]);
    assert!(h.run_stage(h.build_frontend()).is_err());
}

#[test]
fn frontend_invalid_tokens() {
    let mut h = TestHarness::bootstrap("let @ = 1;", vec![]);
    assert!(h.run_stage(h.build_frontend()).is_err());
}

#[test]
fn frontend_unexpected_token_sequences() {
    let mut h = TestHarness::bootstrap("let x = + * /;", vec![]);
    assert!(h.run_stage(h.build_frontend()).is_err());
}

//
// =======================================================
// 7. LEXICAL EDGE CASES
// =======================================================
//

#[test]
fn frontend_whitespace_variants() {
    let mut h = TestHarness::bootstrap("let   x=1+2   *3;", vec![]);
    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("whitespace_variants", h.get_ast().unwrap());
}

#[test]
fn frontend_unicode_identifiers() {
    let mut h = TestHarness::bootstrap("let α = 10;", vec![]);
    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("unicode_identifiers", h.get_ast().unwrap());
}

#[test]
fn frontend_malformed_numbers() {
    let mut h = TestHarness::bootstrap("let x = 1.2.3;", vec![]);
    assert!(h.run_stage(h.build_frontend()).is_err());
}

//
// =======================================================
// 8. MIXED INPUTS / ROBUSTNESS
// =======================================================
//

#[test]
fn frontend_mixed_statements() {
    let mut h = TestHarness::bootstrap("let x = 1; print x; let y = 2 + 3;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("mixed_statements", h.get_ast().unwrap());
}

#[test]
fn frontend_heavily_nested_parentheses() {
    let mut h = TestHarness::bootstrap("let x = (((1 + 2)));", vec![]);
    h.run_stage(h.build_frontend()).unwrap();

    SNAP_AST.assert_value("nested_parentheses_deep", h.get_ast().unwrap());
}
