#[path = "../harness/mod.rs"]
mod harness;
use crate::harness::helpers::assert_expr;

#[test]
fn p01_parses_negation() {
    // Current actual output from your to_sexpr logic: (- 123)
    assert_expr("-123", "(- 123)");
}

#[test]
fn p02_parses_logical_not() {
    assert_expr("!true", "(! true)");
}

#[test]
fn p03_parses_nested_unary() {
    // Parser outputs: (- (- 123))
    assert_expr("--123", "(- (- 123))");
}
