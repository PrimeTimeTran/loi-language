use crate::frontend::ast::{AST, DeclKind, Expr, Stmt};
use crate::frontend::lexer;
use crate::frontend::parser::parse;
use crate::middle::ir::{IROp, Span, Type, TypedExpr};

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static SCOPE_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct SemanticAnalyzer {
    symbols: HashMap<String, Type>,
    scope_counter: AtomicUsize,
}
impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            scope_counter: AtomicUsize::new(0),
        }
    }

    pub fn analyze(ast: AST) -> Result<Vec<IROp>, String> {
        let mut body = Vec::new();
        let mut symbols = HashMap::new();
        for stmt in ast.stmts {
            let lowered = analyze_stmt(stmt, &mut symbols)?;
            body.extend(lowered);
        }

        Ok(body)
    }

    pub fn analyze_block(
        block: Vec<Stmt>,
        symbols: &mut HashMap<String, Type>,
    ) -> Result<Vec<IROp>, String> {
        let mut ir = Vec::new();
        for stmt in block {
            // Pass the mutable reference to symbols to each statement
            let lowered = analyze_stmt(stmt, symbols)?;
            ir.extend(lowered);
        }
        Ok(ir)
    }
}
pub fn analyze(ast: AST) -> Result<Vec<IROp>, String> {
    let mut body = Vec::new();
    let mut symbols = HashMap::new();
    for stmt in ast.stmts {
        let lowered = analyze_stmt(stmt, &mut symbols)?;
        body.extend(lowered);
    }

    Ok(body)
}

fn annotate_types(expr: &mut Expr, symbol_table: &HashMap<String, Type>) -> Type {
    match expr {
        Expr::Var(name) => {
            let ty = symbol_table
                .get(name)
                .cloned()
                .expect("Variable not in table");
            ty
        }
        _ => {
            panic!("Type inference failed for expression: {:?}", expr);
        }
    }
}

fn infer_type(expr: &Expr, symbols: &HashMap<String, Type>) -> Result<Type, String> {
    match expr {
        Expr::Number(_) => Ok(Type::F64),
        Expr::Unary { .. } => Ok(Type::F64),
        Expr::Binary { .. } => Ok(Type::F64),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::String(_) => Ok(Type::Str),
        // FIXED: Actually lookup the variable in the symbols map
        Expr::Var(name) => symbols
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined variable: {}", name)),
        Expr::Call { .. } => Ok(Type::Unknown),
        Expr::Assign { right, .. } => infer_type(right, symbols),
        Expr::Array(items) => {
            if items.is_empty() {
                return Ok(Type::Array(Box::new(Type::Unknown)));
            }
            let first_ty = infer_type(&items[0], symbols)?;
            for item in items.iter().skip(1) {
                let ty = infer_type(item, symbols)?;
                if ty != first_ty {
                    return Err("Array elements must have same type".into());
                }
            }
            Ok(Type::Array(Box::new(first_ty)))
        }
        Expr::Index { target, .. } => {
            let inner = infer_type(target, symbols)?;
            match inner {
                Type::Array(elem_ty) => Ok(*elem_ty),
                _ => Ok(Type::Unknown),
            }
        }
        Expr::Member { .. } => Ok(Type::Unknown),
    }
}

fn analyze_block(
    block: Vec<Stmt>,
    symbols: &mut HashMap<String, Type>,
) -> Result<Vec<IROp>, String> {
    let mut ir = Vec::new();
    for stmt in block {
        // Pass the mutable reference to symbols to each statement
        let lowered = analyze_stmt(stmt, symbols)?;
        ir.extend(lowered);
    }
    Ok(ir)
}
fn analyze_stmt(stmt: Stmt, symbols: &mut HashMap<String, Type>) -> Result<Vec<IROp>, String> {
    let dummy_span = Span { start: 0, end: 0 };
    let typed = |e: Expr| wrap_typed(e, symbols, dummy_span.clone());

    match stmt {
        Stmt::Block { body } => analyze_block(body, symbols),

        Stmt::Let { name, value, .. } => {
            let typed_val = typed(value)?;
            symbols.insert(name.clone(), typed_val.ty.clone());
            Ok(one(IROp::Declare {
                name,
                value: typed_val,
                mutable: true,
                dynamic: false,
            }))
        }

        Stmt::Print { expr } => Ok(one(IROp::Print {
            value: typed(expr)?,
        })),

        Stmt::ExprStmt { expr } => Ok(one(IROp::ExprStmt { expr: typed(expr)? })),

        Stmt::While { condition, body } => Ok(one(IROp::While {
            condition: typed(condition)?,
            body: analyze_block(body, symbols)?,
        })),

        Stmt::DoWhile { body, condition } => {
            // 1. Mutate symbols via analyze_block
            let body_ir = analyze_block(body, symbols)?;

            // 2. ONLY NOW, borrow symbols for wrapping the condition
            let typed_condition = wrap_typed(condition, symbols, dummy_span)?;

            Ok(one(IROp::DoWhile {
                body: body_ir,
                condition: typed_condition,
            }))
        }

        Stmt::Loop { body } => Ok(one(IROp::Loop {
            body: analyze_block(body, symbols)?,
        })),
        Stmt::For {
            iterator,
            iterable,
            body,
        } => Ok(one(IROp::For {
            iterator,
            iterable: typed(iterable)?,
            body: analyze_block(body, symbols)?,
        })),
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let id = SCOPE_COUNTER.fetch_add(1, Ordering::SeqCst);

            // Scope isolation: clone symbols for branches
            let mut then_symbols = symbols.clone();
            let mut else_symbols = symbols.clone();

            Ok(one(IROp::If {
                condition: typed(condition)?,
                then_branch: analyze_block(then_branch, &mut then_symbols)?,
                else_branch: else_branch
                    .map_or(Ok(vec![]), |b| analyze_block(b, &mut else_symbols))?,
                scope_id: id,
            }))
        }
        Stmt::Function { name, params, body } => {
            let mut func_symbols = HashMap::new();

            for param in &params {
                func_symbols.insert(param.clone(), Type::F64);
            }

            let ir_body = analyze_block(body, &mut func_symbols)?;

            Ok(one(IROp::Function {
                name,
                params: params.into_iter().map(|p| (p, Type::F64)).collect(),
                body: ir_body,
                return_type: Type::F64,
            }))
        }
        Stmt::Return { value } => Ok(one(IROp::Return {
            value: value.map(typed).transpose()?,
        })),
    }
}

fn one(op: IROp) -> Vec<IROp> {
    vec![op]
}

fn wrap_typed(
    expr: Expr,
    symbols: &HashMap<String, Type>,
    span: Span,
) -> Result<TypedExpr, String> {
    let ty = infer_type(&expr, symbols)?;
    Ok(TypedExpr { expr, ty, span })
}

#[test]
fn ast_to_ir() {
    let tokens = lexer::lex("1 + 2").unwrap();

    let ast = parse(tokens).unwrap();

    let ir = analyze(ast);

    assert!(ir.is_ok());
}
