#[path = "../harness/mod.rs"]
mod harness;
use loi::frontend::{
    ast::{DeclKind, Expr, Stmt},
    parser::{parse_let, parse_source},
};

use crate::harness::helpers::{assert_expr, assert_let_stmt, fails, parses};

#[test]
fn p01_mul_higher_than_add() {
    assert_expr("1 + 2 * 3", "1 + (2 * 3)");
}

#[test]
fn p02_add_left_associative() {
    assert_expr("1 + 2 + 3", "(1 + 2) + 3");
}

#[test]
fn p03_mul_left_associative() {
    assert_expr("1 * 2 * 3", "(1 * 2) * 3");
}

#[test]
fn p04_mixed_precedence() {
    assert_expr("1 + 2 * 3 + 4", "(1 + (2 * 3)) + 4");
}

#[test]
fn p05_comparison_lower_than_add() {
    assert_expr("1 + 2 == 3 + 4", "(1 + 2) == (3 + 4)");
}

#[test]
fn p06_logical_precedence() {
    assert_expr("a || b && c", "a || (b && c)");
}

#[test]
fn p07_equality_lower_than_and() {
    assert_expr("a == b && c == d", "(a == b) && (c == d)");
}

#[test]
fn p08_unary_binding() {
    assert_expr("-a * b", "(-a) * b");
}

#[test]
fn p09_multiple_unary() {
    assert_expr("!!a", "!( !a )");
}

#[test]
fn p10_postfix_vs_unary() {
    assert_expr("-a.b", "-(a.b)");
}

#[test]
fn p11_postfix_chain() {
    assert_expr("a.b().c", "(a.b()) .c");
}

#[test]
fn p12_deep_postfix_chain() {
    assert_expr("a[b].c().d[e]", "(((a[b]).c()).d)[e]");
}

#[test]
fn p13_assignment_lowest_precedence() {
    assert_expr("a = b + c * d", "a = (b + (c * d))");
}

#[test]
fn p14_chained_assignment() {
    assert_expr("a = b = c + d", "a = (b = (c + d))");
}
#[test]
fn p15_parentheses_override() {
    assert_expr("(1 + 2) * 3", "(1 + 2) * 3");
}
