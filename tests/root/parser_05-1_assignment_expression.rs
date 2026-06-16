use loi::frontend::ast::{DeclKind, Expr, Stmt};

mod common {
    include!("../00_common/mod.rs");
}
use common::{assert_expr, fails, parses};

#[test]
fn p01_parses_simple_assignment() {
    assert_expr("x = 5", "(let x = 5)");
}

#[test]
fn p02_parses_assignment_rhs_expression() {
    // Note: If "x = 1 + 2 * 3" is also treated as a let, use "(let x = (1 + (2 * 3)))"
    assert_expr("x = 1 + 2 * 3", "(let x = (1 + (2 * 3)))");
}

#[test]
fn p03_assignment_is_right_associative() {
    assert_expr("x = y = 5", "(let x = (y = 5))");
}

#[test]
fn p04_rejects_literal_assignment() {
    fails("5 = x");
}

#[test]
fn p05_rejects_binary_expr_assignment() {
    fails("(a + b) = 3");
}
