// tests/05_assignment_expression.rs
mod harness;
use loi::frontend::{
    ast::{DeclKind, Expr, Stmt},
    parser::parse_source,
};

use crate::harness::helpers::{fails, parses};

#[test]
fn p01_parses_simple_assignment() {
    parses("x = 5");
}

#[test]
fn p02_parses_assignment_rhs_expression() {
    parses("x = 1 + 2 * 3");
}

#[test]
fn p03_assignment_is_right_associative() {
    parses("x = y = 5");
}

#[test]
fn p04_rejects_literal_assignment() {
    fails("5 = x");
}

#[test]
fn p05_rejects_binary_expr_assignment() {
    fails("(a + b) = 3");
}
