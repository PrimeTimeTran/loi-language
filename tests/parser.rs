use loi::{
    frontend::{ast, ast::Expr, lexer::lex, parser::parse},
    middle::ir::BinOp,
};

#[test]
fn parse_simple_expr() {
    let tokens = lex("1 + 2").unwrap();
    let ast = parse(tokens).unwrap();

    assert_eq!(ast.stmts.len(), 1);

    let expr = match &ast.stmts[0] {
        ast::Stmt::ExprStmt { expr } => expr,
        _ => panic!("Expected ExprStmt"),
    };

    match expr {
        Expr::Binary { left, op, right } => {
            match left.as_ref() {
                Expr::Number(1) => {}
                _ => panic!(),
            }
            match right.as_ref() {
                Expr::Number(2) => {}
                _ => panic!(),
            }
            assert!(matches!(op, BinOp::Add));
        }
        _ => panic!("Expected binary expression"),
    }
}
