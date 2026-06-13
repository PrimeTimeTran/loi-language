use loi::frontend::{
    ast::{self, BinOp, Expr},
    lexer::lex,
    parser::parse,
};

mod common;
use common::{SNAP_PARSER, assert_expr, parse_to_ast};

#[test]
fn p01_mul_higher_than_add() {
    assert_expr("1 + 2 * 3", "(1 + (2 * 3))");
}

#[test]
fn parse_simple_expr() {
    let ast = parse_to_ast("1 + 2").expect("Parser failed");

    let stmt = ast.stmts.get(0).expect("Missing statement");
    let expr = if let ast::Stmt::ExprStmt { expr } = stmt {
        expr
    } else {
        panic!("Expected ExprStmt, found {:?}", stmt);
    };

    if let Expr::Binary { left, op, right } = expr {
        assert_eq!(left.as_ref(), &Expr::Number(1.0));
        assert_eq!(right.as_ref(), &Expr::Number(2.0));
        assert!(matches!(op, BinOp::Add));
    } else {
        panic!("Expected binary expression");
    }
}
