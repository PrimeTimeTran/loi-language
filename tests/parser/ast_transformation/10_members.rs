use crate::common::assert_expr;

#[test]
fn p01_parses_member_access() {
    assert_expr("a.b", "(identifier(a).b)");
}

#[test]
fn p02_parses_member_chain() {
    assert_expr("a.b.c", "((identifier(a).b).c)");
}

#[test]
fn p03_parses_member_assignment() {
    assert_expr("a.b = 5", "((identifier(a).b) = number(5))");
}
