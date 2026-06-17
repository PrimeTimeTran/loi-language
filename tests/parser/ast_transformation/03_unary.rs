use crate::common::{ParserTestHarness, assert_expr, helpers::parses};

#[test]
fn p01_parses_negation() {
    assert_expr("-123", "(-number(123))");
}

#[test]
fn p02_parses_logical_not() {
    assert_expr("!true", "(! bool(true))");
}

#[test]
fn p03_parses_nested_unary() {
    assert_expr("--123", "(-(-number(123)))");
}
