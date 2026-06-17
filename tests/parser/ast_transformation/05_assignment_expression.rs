use loi::frontend::ast::{DeclKind, Expr, Stmt};

use crate::common::{ParserTestHarness, assert_expr, fails, helpers::parses};

#[test]
fn p01_parses_simple_assignment() {
    assert_expr("x = 5", "(let x = number(5))");
}

#[test]
fn p02_parses_assignment_rhs_expression() {
    assert_expr(
        "x = 1 + 2 * 3",
        "(let x = (number(1) + (number(2) * number(3))))",
    );
}

#[test]
fn p03_assignment_is_right_associative() {
    assert_expr("x = y = 5", "(let x = (identifier(y) = number(5)))");
}

#[test]
fn p04_rejects_literal_assignment() {
    fails("5 = x");
}

#[test]
fn p05_rejects_binary_expr_assignment() {
    fails("(a + b) = 3");
}
