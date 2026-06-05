use loi::frontend::ast::Expr;

pub fn assert_number(expr: &Expr, expected: i64) {
    match expr {
        Expr::Number(n) => assert_eq!(*n, expected),
        _ => panic!("Expected number"),
    }
}
