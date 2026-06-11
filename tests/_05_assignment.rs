mod harness;
use loi::frontend::{
    ast::{DeclKind, Expr, Stmt},
    parser::parse_source,
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
fn p01_parses_simple_assignment() {
    parses("x = 5");
}

#[test]
fn p02_parses_assignment_rhs_expression() {
    parses("x = 1 + 2 * 3");
}

#[test]
fn p03_assignment_is_right_associative() {
    parses("x = y = 5");
}

#[test]
fn p04_rejects_literal_assignment() {
    fails("5 = x");
}

#[test]
fn p05_rejects_binary_expr_assignment() {
    fails("(a + b) = 3");
}

#[test]
fn test_variable_declarations() {
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
