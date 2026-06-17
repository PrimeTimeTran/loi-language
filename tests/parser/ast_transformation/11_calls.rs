use crate::common::assert_expr;

#[test]
fn p01_parses_empty_call() {
    assert_expr("f()", "(identifier(f)())");
}

#[test]
fn p02_parses_single_arg_call() {
    assert_expr("f(1)", "(identifier(f)(number(1)))");
}

#[test]
fn p03_parses_multiple_args_call() {
    assert_expr(
        "f(1, 2, 3)",
        "(identifier(f)(number(1), number(2), number(3)))",
    );
}

#[test]
fn p04_parses_nested_calls() {
    assert_expr(
        "f(g(1), h(2))",
        "(identifier(f)((identifier(g)(number(1))), (identifier(h)(number(2)))))",
    );
}
