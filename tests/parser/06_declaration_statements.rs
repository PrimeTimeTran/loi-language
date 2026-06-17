use loi::frontend::ast::{DeclKind, Expr, Stmt};

mod common {
    include!("../00_common/mod.rs");
}
use common::assert_expr;

//
// ============================
// 1. Top-level assignment/decls
// ============================
//

#[test]
fn p01_mixed_decl_and_expr_assignment() {
    assert_expr(
        "x = 5; y = x = 10;",
        "(let x = number(5))\n(let y = (identifier(x) = number(10)))",
    );
}

#[test]
fn p02_decl_does_not_break_expression_assignment() {
    assert_expr(
        "x = 5; z = (x = 10) + 1;",
        "(let x = number(5))\n(let z = ((identifier(x) = number(10)) + number(1)))",
    );
}

#[test]
fn p03_nested_assignment_expression() {
    assert_expr(
        "a = b = c = 5",
        "(let a = (identifier(b) = (identifier(c) = number(5))))",
    );
}

#[test]
fn p04_member_assignment_with_expression_assignment() {
    assert_expr(
        "a.b = c = 3",
        "((identifier(a).b) = (identifier(c) = number(3)))",
    );
}

#[test]
fn p05_index_assignment_chain() {
    assert_expr(
        "a[i] = x = 4",
        "((identifier(a)[identifier(i)]) = (identifier(x) = number(4)))",
    );
}

//
// ============================
// 2. Identifiers / literals
// ============================
//

#[test]
fn p06_parses_identifier() {
    assert_expr("foo", "identifier(foo)");
}

//
// ============================
// 3. Variable declarations
// ============================
//

#[test]
fn p07_declaration_still_creates_let() {
    assert_expr("x = 5;", "(let x = number(5))");
}

#[test]
fn p08_test_variable_declarations() {
    let test_cases = vec![
        ("x = 5;", "(let x = number(5))"),
        ("y =! 10;", "(let y =! number(10))"),
        ("z =? 5;", "(let z =? number(5))"),
        ("a = 0;", "(let a = number(0))"),
    ];

    for (input, expected) in test_cases {
        assert_expr(input, expected);
    }
}

//
// ============================
// 4. Function declarations (NEW)
// ============================
//

#[test]
fn p09_parses_empty_function() {
    assert_expr("fn foo() {}", "(fn foo() {})");
}

#[test]
fn p10_parses_function_with_params() {
    assert_expr("fn add(a, b) {}", "(fn add(a, b) {})");
}

#[test]
fn p11_parses_function_with_body() {
    assert_expr("fn foo() { x = 1; }", "(fn foo() { (let x = number(1)) })");
}

#[test]
fn p12_function_and_variable_mix() {
    assert_expr("fn foo() {} x = 5;", "(fn foo() {})\n(let x = number(5))");
}

//
// ============================
// 5. Nested scope sanity
// ============================
//

#[test]
fn p13_function_body_is_independent_scope() {
    assert_expr(
        "fn foo() { x = 1; y = x = 2; }",
        "(fn foo() { (let x = number(1))\n(let y = (identifier(x) = number(2))) })",
    );
}
