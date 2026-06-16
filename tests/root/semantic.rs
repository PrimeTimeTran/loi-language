use loi::frontend::ast::{BinOp, Expr, Stmt};
use loi::frontend::{lexer::lex, parser::parse};

// #[test]
// fn parse_simple_expr() {
//     let tokens = lex("1 + 2").unwrap();
//     let ast = parse(tokens).unwrap();

//     assert_eq!(ast.stmts.len(), 1);

//     let expr = match &ast.stmts[0] {
//         Stmt::ExprStmt { expr } => expr,
//         _ => panic!("Expected ExprStmt"),
//     };

//     match expr {
//         Expr::Binary { left, op, right } => {
//             assert!(matches!(&**left, Expr::Number(1.0)));
//             assert!(matches!(&**right, Expr::Number(2.0)));
//             assert!(matches!(op, BinOp::Add));
//         }
//         _ => panic!("Expected binary expression"),
//     }
// }
