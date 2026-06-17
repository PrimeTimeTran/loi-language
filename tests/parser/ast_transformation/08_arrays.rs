use crate::common::assert_expr;

#[test]
fn p01_parses_empty_array() {
    assert_expr("[]", "array([])");
}

#[test]
fn p02_parses_single_element_array() {
    assert_expr("[1]", "array([number(1)])");
}

#[test]
fn p03_parses_multiple_element_array() {
    assert_expr("[1, 2, 3]", "array([number(1), number(2), number(3)])");
}

#[test]
fn p04_parses_nested_arrays() {
    assert_expr(
        "[[1], [2, 3]]",
        "array([array([number(1)]), array([number(2), number(3)])])",
    );
}
