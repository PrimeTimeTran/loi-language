use crate::common::assert_expr;

#[test]
fn p01_parses_array_index() {
    assert_expr("a[0]", "(identifier(a)[number(0)])");
}

#[test]
fn p02_parses_nested_index() {
    assert_expr("a[0][1]", "((identifier(a)[number(0)])[number(1)])");
}

#[test]
fn p03_parses_index_expression() {
    assert_expr("a[1 + 2]", "(identifier(a)[(number(1) + number(2))])");
}

#[test]
fn p04_parses_assignment_to_index() {
    assert_expr("a[0] = 5", "((identifier(a)[number(0)]) = number(5))");
}
