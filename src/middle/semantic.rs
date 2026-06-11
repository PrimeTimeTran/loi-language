use crate::frontend::ast::DeclKind;
use crate::frontend::ast::Expr;
use crate::frontend::ast::Stmt;
use crate::frontend::lexer;
use crate::frontend::parser::AST;
use crate::frontend::parser::parse;
use crate::middle::ir::{IROp, Type, TypedExpr};

pub fn analyze(ast: AST) -> Result<Vec<IROp>, String> {
    let mut body = Vec::new();

    for stmt in ast.stmts {
        let lowered = analyze_stmt(stmt)?;
        body.extend(lowered);
    }

    Ok(body)
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
        Expr::Assign { right, .. } => infer_type(right.as_ref()),
        Expr::Array(items) => {
            if items.is_empty() {
                return Ok(Type::Array(Box::new(Type::Unknown)));
            }
            let first_ty = infer_type(&items[0])?;
            for item in items.iter().skip(1) {
                let ty = infer_type(item)?;
                if ty != first_ty {
                    return Err("Array elements must have same type".into());
                }
            }
            Ok(Type::Array(Box::new(first_ty)))
        }
        Expr::Index { target, .. } => {
            let inner = infer_type(target)?;
            match inner {
                Type::Array(elem_ty) => Ok(*elem_ty),
                _ => Ok(Type::Unknown),
            }
        }

        Expr::Member { .. } => Ok(Type::Unknown),
    }
}

fn analyze_block(block: Vec<Stmt>) -> Result<Vec<IROp>, String> {
    let mut ir = Vec::new();
    for stmt in block {
        let lowered = analyze_stmt(stmt)?;
        ir.extend(lowered);
    }
    Ok(ir)
}

fn analyze_stmt(stmt: Stmt) -> Result<Vec<IROp>, String> {
    let typed = |e: Expr| infer_type(&e).map(|ty| TypedExpr(e, ty));
    match stmt {
        Stmt::Block { body } => analyze_block(body),

        Stmt::Let { name, kind, value } => Ok(one(IROp::Declare {
            name,
            value: typed(value)?,
            mutable: matches!(kind, DeclKind::MutableStatic | DeclKind::Dynamic),
            dynamic: matches!(kind, DeclKind::Dynamic),
        })),
        Stmt::Print { expr } => Ok(one(IROp::Print {
            value: typed(expr)?,
        })),

        Stmt::ExprStmt { expr } => Ok(one(IROp::ExprStmt { expr: typed(expr)? })),
        Stmt::While { condition, body } => Ok(one(IROp::While {
            condition: typed(condition)?,
            body: analyze_block(body)?,
        })),
        Stmt::DoWhile { body, condition } => Ok(one(IROp::DoWhile {
            body: analyze_block(body)?,
            condition: typed(condition)?,
        })),
        Stmt::Loop { body } => Ok(one(IROp::Loop {
            body: analyze_block(body)?,
        })),
        Stmt::For {
            iterator,
            iterable,
            body,
        } => Ok(one(IROp::For {
            iterator,
            iterable: typed(iterable)?,
            body: analyze_block(body)?,
        })),
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => Ok(one(IROp::If {
            condition: typed(condition)?,
            then_branch: analyze_block(then_branch)?,
            else_branch: else_branch.map_or(Ok(vec![]), analyze_block)?,
        })),
        Stmt::Function { name, params, body } => Ok(one(IROp::Function {
            name,
            params: params.into_iter().map(|p| (p, Type::Unknown)).collect(),
            body: analyze_block(body)?,
            return_type: Type::Unknown,
        })),
        Stmt::Return { value } => Ok(one(IROp::Return {
            value: value.map(typed).transpose()?,
        })),
    }
}

fn one(op: IROp) -> Vec<IROp> {
    vec![op]
}

fn wrap_typed(expr: Expr) -> Result<TypedExpr, String> {
    let ty = infer_type(&expr)?;
    Ok(TypedExpr(expr, ty))
}

#[test]
fn ast_to_ir() {
    let tokens = lexer::lex("1 + 2").unwrap();

    let ast = parse(tokens).unwrap();

    let ir = analyze(ast);

    assert!(ir.is_ok());
}
