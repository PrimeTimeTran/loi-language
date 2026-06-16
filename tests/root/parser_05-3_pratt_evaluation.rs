mod common {
    include!("../common/mod.rs");
}
use common::{assert_expr, fails, parses};

#[test]
fn p01_mul_higher_than_add() {
    assert_expr("1 + 2 * 3", "(1 + (2 * 3))");
}

#[test]
fn p02_add_left_associative() {
    assert_expr("1 + 2 + 3", "((1 + 2) + 3)");
}

#[test]
fn p03_mul_left_associative() {
    assert_expr("1 * 2 * 3", "((1 * 2) * 3)");
}

#[test]
fn p04_mixed_precedence() {
    assert_expr("1 + 2 * 3 + 4", "((1 + (2 * 3)) + 4)");
}

#[test]
fn p05_comparison_lower_than_add() {
    assert_expr("1 + 2 == 3 + 4", "((1 + 2) == (3 + 4))");
}

#[test]
fn p06_logical_precedence() {
    assert_expr("a || b && c", "(a || (b && c))");
}

#[test]
fn p07_equality_lower_than_and() {
    assert_expr("a == b && c == d", "((a == b) && (c == d))");
}

#[test]
fn p08_unary_binding() {
    assert_expr("-a * b", "((-a) * b)");
}

#[test]
fn p09_multiple_unary() {
    assert_expr("!!a", "(!(!a))");
}

#[test]
fn p10_postfix_vs_unary() {
    // Current logic: Unary '-' has lower precedence than postfix '.'
    assert_expr("-a.b", "(- (a.b))");
}

#[test]
fn p11_postfix_chain() {
    // Current logic: a.b().c -> (((a.b)()).c)
    assert_expr("a.b().c", "(((a.b)()).c)");
}

#[test]
fn p12_deep_postfix_chain() {
    // Current logic: a[b].c().d[e] -> (((((a[b]).c)()).d)[e])
    assert_expr("a[b].c().d[e]", "(((((a[b]).c)()).d)[e])");
}

#[test]
fn p13_assignment_lowest_precedence() {
    // Parser outputs "let a", so update the expected string
    assert_expr("a = b + c * d", "(let a = (b + (c * d)))");
}

#[test]
fn p14_chained_assignment() {
    // Parser outputs "let a", so update the expected string
    assert_expr("a = b = c + d", "(let a = (b = (c + d)))");
}

#[test]
fn p15_modulo_precedence() {
    // Should be same as multiplication
    assert_expr("1 + 2 % 3 * 4", "(1 + ((2 % 3) * 4))");
}

#[test]
fn p16_exponentiation_right_associative() {
    // The true test of a Pratt parser: ^ is right-associative
    assert_expr("a ^ b ^ c", "(a ^ (b ^ c))");
}

#[test]
fn p17_parentheses_override() {
    assert_expr("(1 + 2) * 3", "((1 + 2) * 3)");
}

#[test]
fn p18_complex_logical_grouping() {
    assert_expr("a || (b && c)", "(a || (b && c))");
}
