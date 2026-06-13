use loi::frontend::{
    ast::{self, BinOp, Expr},
    lexer::lex,
    parser::parse,
};

mod harness;

use crate::harness::helpers::{assert_expr, parse_to_ast};
use crate::harness::{SNAP_PARSER, lexer::LexerTestHarness};

#[test]
fn p01_mul_higher_than_add() {
    assert_expr("1 + 2 * 3", "(1 + (2 * 3))");
}

#[test]
fn parse_simple_expr() {
    SNAP_PARSER.assert("parser_is_up", "Hello Parser", "Parser Hello");
    let ast = parse_to_ast("1 + 2");
    assert_eq!(ast.stmts.len(), 1);
    let expr = match &ast.stmts[0] {
        ast::Stmt::ExprStmt { expr } => expr,
        _ => panic!("Expected ExprStmt"),
    };

    match expr {
        Expr::Binary { left, op, right } => {
            match left.as_ref() {
                Expr::Number(1.0) => {}
                _ => panic!(),
            }
            match right.as_ref() {
                Expr::Number(2.0) => {}
                _ => panic!(),
            }
            assert!(matches!(op, BinOp::Add));
        }
        _ => panic!("Expected binary expression"),
    }
}
