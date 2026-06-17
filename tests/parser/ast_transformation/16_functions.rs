use crate::common::assert_expr;

#[test]
fn p01_parses_empty_function() {
    assert_expr("fn foo() {}", "fn(foo, [], block([]), none)");
}

#[test]
fn p02_parses_function_with_params() {
    assert_expr(
        "fn add(a, b, c) { x = 1 }",
        "fn(add, [a, b, c], block([(let x = number(1))]), none)",
    );
}

#[test]
fn p03_parses_function_with_return() {
    assert_expr(
        "fn foo() { return 42 }",
        "fn(foo, [], block([return(number(42))]), number(42))",
    );
}

#[test]
fn p04_parses_nested_function_calls() {
    assert_expr(
        "fn f() { return g(h(1)) }",
        "fn(f, [], block([return(identifier(g)(identifier(h)(number(1))))]), identifier(g)(identifier(h)(number(1))))",
    );
}
