use crate::common::assert_expr;

#[test]
fn p01_parses_integer() {
    assert_expr("123", "number(123)");
}

#[test]
fn p02_parenthesis_override_precedence() {
    assert_expr("(4 + 2) * 3", "((number(4) + number(2)) * number(3))");
}

#[test]
fn p03_comparison_lower_than_addition() {
    assert_expr("1 + 2 < 5", "((number(1) + number(2)) < number(5))");
}

#[test]
fn p04_equality_lower_than_comparison() {
    assert_expr("1 == 2 < 3", "((number(1) == number(2)) < number(3))");
}

#[test]
fn p05_logical_and_lower_than_equality() {
    assert_expr(
        "a == b && c == d",
        "((identifier(a) == identifier(b)) && (identifier(c) == identifier(d)))",
    );
}

#[test]
fn p06_logical_or_lower_than_and() {
    assert_expr(
        "a || b && c",
        "(identifier(a) || (identifier(b) && identifier(c)))",
    );
}
