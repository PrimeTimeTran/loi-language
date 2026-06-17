use crate::common::assert_expr;

#[test]
fn p01_parses_integer() {
    assert_expr("123", "number(123)");
}

#[test]
fn p02_parses_float() {
    assert_expr("123.456", "number(123.456)");
}

#[test]
fn p03_parses_string() {
    assert_expr("\"hello\"", "string(hello)");
}

#[test]
fn p04_parses_true() {
    assert_expr("true", "bool(true)");
}

#[test]
fn p05_parses_false() {
    assert_expr("false", "bool(false)");
}

#[test]
fn p06_parses_identifier() {
    assert_expr("foo", "identifier(foo)");
}
