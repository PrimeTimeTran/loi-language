use crate::frontend::ast::DeclKind;
use crate::frontend::ast::Expr;
// src/middle/semantic.rs
use crate::frontend::ast::Stmt;
use crate::frontend::lexer;
use crate::frontend::parser::AST;
use crate::frontend::parser::parse;
use crate::middle::ir::{IR, Type, TypedExpr};

pub fn analyze(ast: AST) -> Result<IR, String> {
    let mut body = Vec::new();

    for stmt in ast.stmts {
        match stmt {
            // -------------------------
            // VARIABLE DECLARATION
            // -------------------------
            Stmt::Let { name, kind, value } => {
                let ty = infer_type(&value)?; // assume you have this

                body.push(IR::Declare {
                    name,
                    value: TypedExpr(value, ty),
                    mutable: matches!(kind, DeclKind::MutableStatic | DeclKind::Dynamic),
                    dynamic: matches!(kind, DeclKind::Dynamic),
                });
            }

            // -------------------------
            // ASSIGNMENT
            // -------------------------
            Stmt::Let { name, kind, value } => {
                assert_eq!(name, "x");
                assert!(matches!(kind, DeclKind::MutableStatic));

                match value {
                    Expr::Number(n) => assert_eq!(n, 5.0),
                    _ => panic!("expected number"),
                }
            }

            // -------------------------
            // PRINT
            // -------------------------
            Stmt::Print { expr } => {
                let ty = infer_type(&expr)?;

                body.push(IR::Print {
                    value: TypedExpr(expr, ty),
                });
            }

            // -------------------------
            // EXPRESSION STATEMENT
            // -------------------------
            Stmt::ExprStmt { expr } => {
                let ty = infer_type(&expr)?;

                body.push(IR::ExprStmt {
                    expr: TypedExpr(expr, ty),
                });
            }
        }
    }

    Ok(IR::Module { body })
}

fn infer_type(expr: &Expr) -> Result<Type, String> {
    match expr {
        Expr::Number(_) => Ok(Type::F64),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::String(_) => Ok(Type::Str),

        Expr::Var(_) => Ok(Type::Unknown),

        Expr::Binary { .. } => Ok(Type::F64),
        Expr::Unary { .. } => Ok(Type::F64),

        Expr::Call { .. } => Ok(Type::Unknown),
    }
}

#[test]
fn ast_to_ir() {
    let tokens = lexer::lex("1 + 2").unwrap();

    let ast = parse(tokens).unwrap();

    let ir = analyze(ast);

    assert!(ir.is_ok());
}
