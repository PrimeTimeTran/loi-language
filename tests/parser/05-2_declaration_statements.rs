use loi::frontend::ast::{DeclKind, Expr, Stmt};

mod common {
    include!("../00_common/mod.rs");
}
use common::assert_expr;

#[test]
fn p01_mixed_decl_and_expr_assignment() {
    assert_expr("x = 5; y = x = 10;", "(let x = 5)\n(let y = (x = 10))");
}

#[test]
fn p02_decl_does_not_break_expression_assignment() {
    assert_expr(
        "x = 5; z = (x = 10) + 1;",
        "(let x = 5)\n(let z = ((x = 10) + 1))",
    );
}

#[test]
fn p03_nested_assignment_expression() {
    assert_expr("a = b = c = 5", "(let a = (b = (c = 5)))");
}

#[test]
fn p04_member_assignment_with_expression_assignment() {
    assert_expr("a.b = c = 3", "((a.b) = (c = 3))");
}

#[test]
fn p05_index_assignment_chain() {
    assert_expr("a[i] = x = 4", "((a[i]) = (x = 4))");
}

#[test]
fn p06_parses_identifier() {
    assert_expr("foo", "foo");
}

#[test]
fn p07_declaration_still_creates_let() {
    assert_expr("x = 5;", "(let x = 5)");
}

#[test]
fn p08_test_variable_declarations() {
    let test_cases = vec![
        ("x = 5;", "(let x = 5)"),
        ("y =! 10;", "(let y =! 10)"),
        ("z =? 5;", "(let z =? 5)"),
        ("a = 0;", "(let a = 0)"),
    ];

    for (input, expected) in test_cases {
        assert_expr(input, expected);
    }
}
