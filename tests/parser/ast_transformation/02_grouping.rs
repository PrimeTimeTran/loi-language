use crate::common::assert_expr;

#[test]
fn p01_parses_grouping() {
    assert_expr("(123)", "(number(123))");
}

#[test]
fn p02_parses_nested_grouping() {
    assert_expr("(((123)))", "(number(123))");
}

#[test]
fn p03_parses_grouping() {
    assert_expr("(1 + 2)", "((number(1) + number(2)))");
}

#[test]
fn p04_parses_nested_grouping() {
    assert_expr("((1 + 2) * 3)", "((number(1) + number(2)) * number(3))");
}
