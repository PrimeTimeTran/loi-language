// tests/05_declaration_statements.rs
mod harness;
use loi::frontend::{
    ast::{DeclKind, Expr, Stmt},
    parser::{parse_let, parse_source},
};

use crate::harness::helpers::{fails, parses};

fn assert_let_stmt(input: &str, expected_name: &str, expected_kind: DeclKind, expected_val: f64) {
    let ast = parse_source(input).expect("Parser failed");
    assert_eq!(
        ast.stmts.len(),
        1,
        "Expected 1 statement for input: {}",
        input
    );

    match &ast.stmts[0] {
        Stmt::Let { name, kind, value } => {
            assert_eq!(name, expected_name);
            assert_eq!(kind, &expected_kind);
            if let Expr::Number(n) = value {
                assert_eq!(*n, expected_val);
            } else {
                panic!("Expected number value, got {:?}", value);
            }
        }
        other => panic!("Expected Let statement, got {:?}", other),
    }
}

#[test]
fn p01_mixed_decl_and_expr_assignment() {
    parse_source(
        "
        x = 5;
        y = x = 10;
    ",
    );
}

#[test]
fn p02_decl_does_not_break_expression_assignment() {
    parse_source(
        "
        x = 5;
        z = (x = 10) + 1;
    ",
    );
}

#[test]
fn p03_nested_assignment_expression() {
    parses("a = b = c = 5");
}

#[test]
fn p04_member_assignment_with_expression_assignment() {
    parses("a.b = c = 3");
}

#[test]
fn p05_index_assignment_chain() {
    parses("a[i] = x = 4");
}

#[test]
fn p06_declaration_still_creates_let() {
    assert_let_stmt("x = 5;", "x", DeclKind::MutableStatic, 5.0);
}

#[test]
fn p07_test_variable_declarations() {
    let test_cases = vec![
        ("x = 5;", "x", DeclKind::MutableStatic, 5.0),
        ("y =! 10;", "y", DeclKind::ImmutableStatic, 10.0),
        ("z =? 5;", "z", DeclKind::Dynamic, 5.0),
        ("a = 0;", "a", DeclKind::MutableStatic, 0.0),
    ];

    for (input, name, kind, val) in test_cases {
        assert_let_stmt(input, name, kind, val);
    }
}
