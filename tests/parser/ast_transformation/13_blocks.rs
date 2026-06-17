use crate::common::assert_expr;

#[test]
fn p01_parses_empty_block() {
    assert_expr("{}", "block([])");
}

#[test]
fn p02_parses_single_statement_block() {
    assert_expr("{ x = 5 }", "block([(let x = number(5))])");
}

#[test]
fn p03_parses_nested_blocks() {
    assert_expr("{ { x = 5 } }", "block([block([(let x = number(5))])])");
}
