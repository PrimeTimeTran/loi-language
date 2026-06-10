use crate::frontend::ast::DeclKind;
use crate::frontend::ast::Expr;
// src/middle/semantic.rs
use crate::frontend::ast::Stmt;
use crate::frontend::lexer;
use crate::frontend::parser::AST;
use crate::frontend::parser::parse;
use crate::middle::ir::{IROp, Type, TypedExpr};

pub fn analyze(ast: AST) -> Result<IROp, String> {
    let mut body = Vec::new();

    for stmt in ast.stmts {
        match stmt {
            Stmt::For {
                iterator,
                iterable,
                body: block,
            } => {
                let ty = infer_type(&iterable)?;

                body.push(IROp::For {
                    iterator,
                    iterable: TypedExpr(iterable, ty),
                    body: analyze_block(block)?,
                });
            }
            Stmt::Loop { body: block } => {
                body.push(IROp::Loop {
                    body: analyze_block(block)?,
                });
            }
            Stmt::While {
                condition,
                body: block,
            } => {
                let cond_ty = infer_type(&condition)?;

                body.push(IROp::While {
                    condition: TypedExpr(condition, cond_ty),
                    body: analyze_block(block)?,
                });
            }
            Stmt::DoWhile {
                body: stmts,
                condition,
            } => {
                let cond_ty = infer_type(&condition)?;

                body.push(IROp::DoWhile {
                    body: analyze_block(stmts)?,
                    condition: TypedExpr(condition, cond_ty),
                });
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_ty = infer_type(&condition)?;

                let then_ir = analyze_block(then_branch)?;
                let else_ir = match else_branch {
                    Some(b) => analyze_block(b)?,
                    None => vec![],
                };

                body.push(IROp::If {
                    condition: TypedExpr(condition, cond_ty),
                    then_branch: then_ir,
                    else_branch: else_ir,
                });
            }
            Stmt::Return { value } => {
                let value = match value {
                    Some(e) => Some(TypedExpr(e.clone(), infer_type(&e)?)),
                    None => None,
                };

                body.push(IROp::Return { value });
            }
            Stmt::Function {
                name,
                params,
                body: stmts,
            } => {
                let mut ir_body = Vec::new();

                for stmt in stmts {
                    let lowered = analyze_stmt(stmt)?;
                    ir_body.push(lowered);
                }

                body.push(IROp::Function {
                    name,
                    params: params
                        .into_iter()
                        .map(|p| (p, Type::Unknown)) // or real inference later
                        .collect(),

                    body: ir_body,

                    return_type: Type::Unknown,
                });
            }
            // -------------------------
            // VARIABLE DECLARATION
            // -------------------------
            Stmt::Let { name, kind, value } => {
                let ty = infer_type(&value)?; // assume you have this

                body.push(IROp::Declare {
                    name,
                    value: TypedExpr(value, ty),
                    mutable: matches!(kind, DeclKind::MutableStatic | DeclKind::Dynamic),
                    dynamic: matches!(kind, DeclKind::Dynamic),
                });
            }
            // -------------------------
            // PRINT
            // -------------------------
            Stmt::Print { expr } => {
                let ty = infer_type(&expr)?;

                body.push(IROp::Print {
                    value: TypedExpr(expr, ty),
                });
            }

            // -------------------------
            // EXPRESSION STATEMENT
            // -------------------------
            Stmt::ExprStmt { expr } => {
                let ty = infer_type(&expr)?;

                body.push(IROp::ExprStmt {
                    expr: TypedExpr(expr, ty),
                });
            }
        }
    }

    Ok(IROp::Module { body })
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
        Expr::Array(items) => {
            if items.is_empty() {
                return Ok(Type::Array(Box::new(Type::Unknown)));
            }

            let first_ty = infer_type(&items[0])?;

            // optional: enforce homogeneity
            for item in items.iter().skip(1) {
                let ty = infer_type(item)?;
                if ty != first_ty {
                    return Err("Array elements must have same type".into());
                }
            }

            Ok(Type::Array(Box::new(first_ty)))
        }
    }
}

fn analyze_block(block: Vec<Stmt>) -> Result<Vec<IROp>, String> {
    let mut ir = Vec::new();

    for stmt in block {
        ir.push(analyze_stmt(stmt)?);
    }

    Ok(ir)
}
fn analyze_stmt(stmt: Stmt) -> Result<IROp, String> {
    let body: Vec<IROp> = Vec::new();

    let ir = match stmt {
        Stmt::Let { name, kind, value } => {
            let ty = infer_type(&value)?;

            IROp::Declare {
                name,
                value: TypedExpr(value, ty),
                mutable: matches!(kind, DeclKind::MutableStatic | DeclKind::Dynamic),
                dynamic: matches!(kind, DeclKind::Dynamic),
            }
        }

        Stmt::Print { expr } => {
            let ty = infer_type(&expr)?;
            IROp::Print {
                value: TypedExpr(expr, ty),
            }
        }

        Stmt::ExprStmt { expr } => {
            let ty = infer_type(&expr)?;
            IROp::ExprStmt {
                expr: TypedExpr(expr, ty),
            }
        }

        Stmt::While { condition, body } => {
            let cond_ty = infer_type(&condition)?;
            IROp::While {
                condition: TypedExpr(condition, cond_ty),
                body: analyze_block(body)?,
            }
        }

        Stmt::DoWhile { body, condition } => {
            let cond_ty = infer_type(&condition)?;
            IROp::DoWhile {
                body: analyze_block(body)?,
                condition: TypedExpr(condition, cond_ty),
            }
        }

        Stmt::Loop { body } => IROp::Loop {
            body: analyze_block(body)?,
        },

        Stmt::For {
            iterator,
            iterable,
            body,
        } => {
            let ty = infer_type(&iterable)?;
            IROp::For {
                iterator,
                iterable: TypedExpr(iterable, ty),
                body: analyze_block(body)?,
            }
        }

        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let cond_ty = infer_type(&condition)?;

            IROp::If {
                condition: TypedExpr(condition, cond_ty),
                then_branch: analyze_block(then_branch)?,
                else_branch: match else_branch {
                    Some(b) => analyze_block(b)?,
                    None => vec![],
                },
            }
        }

        Stmt::Function { name, params, body } => {
            let ir_body = analyze_block(body)?;

            IROp::Function {
                name,
                params: params.into_iter().map(|p| (p, Type::Unknown)).collect(),
                body: ir_body,
                return_type: Type::Unknown,
            }
        }

        Stmt::Return { value } => IROp::Return {
            value: value.map(|e| TypedExpr(e.clone(), infer_type(&e.clone()).unwrap())),
        },
    };

    Ok(ir)
}

#[test]
fn ast_to_ir() {
    let tokens = lexer::lex("1 + 2").unwrap();

    let ast = parse(tokens).unwrap();

    let ir = analyze(ast);

    assert!(ir.is_ok());
}
