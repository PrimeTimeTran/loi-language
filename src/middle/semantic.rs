// src/middle/semantic.rs

use crate::frontend::ast::Stmt;
use crate::frontend::parser::AST;
use crate::middle::ir::{IR, Type, TypedExpr};

pub fn analyze(ast: AST) -> Result<IR, String> {
    let mut body = Vec::new();

    for stmt in ast.stmts {
        match stmt {
            Stmt::Assign { name, expr } => {
                body.push(IR::Assign {
                    name,
                    value: TypedExpr(
                        expr,
                        Type::I32, // temporary default
                    ),
                });
            }

            Stmt::Print { expr } => {
                body.push(IR::Print {
                    value: TypedExpr(expr, Type::I32),
                });
            }

            Stmt::ExprStmt { expr } => {
                body.push(IR::ExprStmt {
                    expr: TypedExpr(expr, Type::I32),
                });
            }
        }
    }

    Ok(IR::Module { body })
}

#[test]
fn ast_to_ir() {
    let tokens = lexer::lex("1 + 2").unwrap();

    let ast = parse(tokens).unwrap();

    let ir = analyze(ast);

    assert!(ir.is_ok());
}
