use serde::Serialize;
use std::iter::Peekable;

use crate::{
    compiler::diagnostic::{self, Diagnostic, DiagnosticStore},
    frontend::{
        ast::{AST, AssignOp, BinOp, DeclKind, Expr, HashF64, Stmt, UnOp},
        lexer::lex,
        token::Token,
        types::TokenStream,
    },
    middle::types::Span,
};

#[derive(Debug, Default)]
pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(
        &mut self,
        mut tokens: TokenStream,
        diagnostics: &mut DiagnosticStore,
    ) -> Result<AST, DiagnosticStore> {
        parse_program(&mut tokens, diagnostics)
    }

    pub fn parse_incremental(
        &mut self,
        prev: &AST,
        tokens: &mut TokenStream,
        diagnostics: &mut DiagnosticStore,
    ) -> Result<AST, DiagnosticStore> {
        parse_incremental(prev, tokens, diagnostics)
    }
}

pub fn parse_program(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<AST, DiagnosticStore> {
    let mut stmts = Vec::new();

    while let Some(token) = tokens.peek() {
        match tokens.peek() {
            Some(Token::EOF) => break,
            Some(_) => match parse_stmt(tokens, diagnostics) {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    diagnostics.emit(Diagnostic::error(
                        format!("Failed to parse statement: {}", e),
                        Span::default(),
                    ));
                    return Err(diagnostics.clone());
                }
            },
            None => break,
        }
    }

    println!("FINAL STMTS: {:?}", stmts);

    Ok(AST::new(stmts))
}
pub fn parse(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<AST, DiagnosticStore> {
    parse_program(tokens, diagnostics)
}
pub fn parse_incremental(
    prev: &AST,
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<AST, DiagnosticStore> {
    let mut stmts = prev.stmts.clone();

    let mut updated = Vec::new();

    while let Some(tok) = tokens.peek() {
        match tok {
            Token::EOF => {
                tokens.bump();
                break;
            }

            Token::Semicolon => {
                tokens.bump();
            }

            _ => match parse_stmt(tokens, diagnostics) {
                Ok(stmt) => updated.push(stmt),

                Err(_) => {
                    diagnostics.emit(Diagnostic::error(
                        "Incremental parse error",
                        Span::default(),
                    ));

                    tokens.bump();
                }
            },
        }
    }

    stmts.extend(updated);

    // LEAVE FOR CLI/REPL
    // let last_expr = stmts.iter().rev().find_map(|stmt| {
    //     if let Stmt::ExprStmt { expr } = stmt {
    //         Some(expr.clone())
    //     } else {
    //         None
    //     }
    // });

    Ok(AST::new(stmts))
}

fn parse_stmt(tokens: &mut TokenStream, diagnostics: &mut DiagnosticStore) -> Result<Stmt, String> {
    println!("PARSE STMT START");
    println!("PEEK AT STMT START: {:?}", tokens.peek());

    let stmt = match tokens.peek() {
        Some(Token::Let) => parse_let(tokens, diagnostics)?,
        Some(Token::If) => control::parse_if(tokens, diagnostics)?,
        Some(Token::While) => control::parse_while(tokens, diagnostics)?,
        Some(Token::Do) => control::parse_do_while(tokens, diagnostics)?,
        Some(Token::Return) => control::parse_return(tokens, diagnostics)?,
        Some(Token::Function) => control::parse_function(tokens, diagnostics)?,

        Some(Token::Print) => {
            tokens.bump();
            Stmt::Print {
                expr: parse_expr(tokens, diagnostics)?,
            }
        }

        Some(Token::LBrace) => {
            tokens.bump();
            let body = control::parse_block(tokens, diagnostics)?;
            Stmt::Block { body }
        }

        _ => {
            let expr = parse_assignment(tokens, None, diagnostics)?;

            match expr {
                Expr::Assign { left, right, op } if is_simple_var(&left) => {
                    if let Expr::Var(name) = *left {
                        let kind: DeclKind = op.into();
                        Stmt::Let {
                            name,
                            kind,
                            value: flatten_assign(*right),
                        }
                    } else {
                        unreachable!()
                    }
                }

                other => Stmt::ExprStmt { expr: other },
            }
        }
    };

    if matches!(tokens.peek(), Some(Token::Semicolon)) {
        tokens.bump();
    }

    println!("STMT PUSHED: {:?}", stmt);

    Ok(stmt)
}
fn parse_expr(tokens: &mut TokenStream, diagnostics: &mut DiagnosticStore) -> Result<Expr, String> {
    parse_assignment(tokens, None, diagnostics)
}

fn parse_assignment(
    tokens: &mut TokenStream,
    lhs: Option<Expr>,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    let left = match lhs {
        Some(expr) => expr,
        None => parse_or(tokens, diagnostics)?,
    };

    println!("ASSIGN LEFT: {:?}", left);

    let op = match tokens.peek() {
        Some(Token::Assign | Token::Immutable | Token::Dynamic) => tokens.next().unwrap(),
        _ => return Ok(left),
    };

    let assign_op = match op {
        Token::Assign => AssignOp::Assign,
        Token::Immutable => AssignOp::Immutable,
        Token::Dynamic => AssignOp::Dynamic,
        _ => unreachable!(),
    };

    if !is_assignable(&left) {
        diagnostics.emit(Diagnostic::error(
            "Invalid assignment target",
            Span::default(),
        ));
        return Err("Invalid assignment target".into());
    }

    let right = parse_assignment(tokens, None, diagnostics)?;

    Ok(Expr::Assign {
        left: Box::new(left),
        right: Box::new(right),
        op: assign_op,
    })
}

pub fn parse_let(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Stmt, String> {
    tokens.bump();

    let name = match tokens.next() {
        Some(Token::Ident(name)) => name,
        other => {
            diagnostics.emit(Diagnostic::error(
                "Expected identifier after 'let'",
                Span::default(),
            ));
            return Err(format!("expected identifier, found {:?}", other));
        }
    };

    let maybe_op = match tokens.peek() {
        Some(Token::Assign) => {
            tokens.bump();
            Some(AssignOp::Assign)
        }
        Some(Token::Immutable) => {
            tokens.bump();
            Some(AssignOp::Immutable)
        }
        Some(Token::Dynamic) => {
            tokens.bump();
            Some(AssignOp::Dynamic)
        }
        _ => None,
    };

    let (kind, value) = match maybe_op {
        Some(op) => {
            // If we had an operator, we MUST have a value
            let val = parse_assignment(tokens, None, diagnostics)?;
            (op.into(), val)
        }
        None => (DeclKind::Dynamic, Expr::Empty),
    };

    if matches!(tokens.peek(), Some(Token::Semicolon)) {
        tokens.bump();
    }

    Ok(Stmt::Let { name, kind, value })
}
fn parse_equality(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    let mut left = parse_comparison(tokens, diagnostics)?;

    while let Some(tok) = tokens.peek() {
        match tok {
            Token::Eq => {
                tokens.next();

                let right = parse_comparison(tokens, diagnostics)?;

                left = Expr::Binary {
                    left: Box::new(left),
                    op: BinOp::Eq,
                    right: Box::new(right),
                };
            }

            Token::Neq => {
                tokens.next();

                let right = parse_comparison(tokens, diagnostics)?;

                left = Expr::Binary {
                    left: Box::new(left),
                    op: BinOp::Neq,
                    right: Box::new(right),
                };
            }

            _ => break,
        }
    }

    Ok(left)
}
fn parse_or(tokens: &mut TokenStream, diagnostics: &mut DiagnosticStore) -> Result<Expr, String> {
    let mut left = parse_and(tokens, diagnostics)?;

    while matches!(tokens.peek(), Some(Token::Or)) {
        tokens.next();

        let right = parse_and(tokens, diagnostics)?;

        left = Expr::Binary {
            left: Box::new(left),
            op: BinOp::Or,
            right: Box::new(right),
        };
    }

    Ok(left)
}
fn parse_and(tokens: &mut TokenStream, diagnostics: &mut DiagnosticStore) -> Result<Expr, String> {
    let mut left = parse_equality(tokens, diagnostics)?;

    while matches!(tokens.peek(), Some(Token::And)) {
        tokens.next();

        let right = parse_equality(tokens, diagnostics)?;

        left = Expr::Binary {
            left: Box::new(left),
            op: BinOp::And,
            right: Box::new(right),
        };
    }

    Ok(left)
}
fn parse_comparison(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    let mut left = parse_add_sub(tokens, diagnostics)?;

    while let Some(tok) = tokens.peek() {
        let op = match tok {
            Token::Lt => BinOp::Lt,
            Token::Gt => BinOp::Gt,
            _ => break,
        };

        tokens.next();

        let right = parse_add_sub(tokens, diagnostics)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}
fn parse_primary(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    match tokens.next() {
        Some(Token::True) => Ok(Expr::Bool(true)),
        Some(Token::False) => Ok(Expr::Bool(false)),
        Some(Token::Number(n)) => Ok(Expr::Number(HashF64(n))),
        Some(Token::String(s)) => Ok(Expr::String(s.clone())),
        Some(Token::Ident(name)) => {
            if let Some(Token::LParen) = tokens.peek() {
                tokens.next();
                let arg = parse_expr(tokens, diagnostics)?;
                match tokens.next() {
                    Some(Token::RParen) => Ok(Expr::Call {
                        callee: Box::new(Expr::Var(name)),
                        args: vec![arg],
                    }),
                    other => Err(format!("Expected ')', got {:?}", other)),
                }
            } else {
                Ok(Expr::Var(name.clone()))
            }
        }

        Some(Token::Ampersand) => {
            let expr = parse_primary(tokens, diagnostics)?;
            Ok(Expr::Unary {
                op: UnOp::AddrOf,
                expr: Box::new(expr),
            })
        }

        Some(Token::LParen) => {
            let expr = parse_expr(tokens, diagnostics)?;
            match tokens.next() {
                Some(Token::RParen) => Ok(expr),
                other => Err(format!("Expected ')', got {:?}", other)),
            }
        }

        Some(Token::LBracket) => {
            let mut items = vec![];

            while let Some(tok) = tokens.peek() {
                if matches!(tok, Token::RBracket) {
                    tokens.next();
                    break;
                }

                items.push(parse_expr(tokens, diagnostics)?);

                if matches!(tokens.peek(), Some(Token::Comma)) {
                    tokens.next();
                }
            }

            Ok(Expr::Array(items))
        }

        None => Err("Unexpected EOF".into()),

        Some(other) => Err(format!("Unexpected token: {:?}", other)),
    }
}
fn parse_postfix(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    let mut expr = parse_primary(tokens, diagnostics)?;

    loop {
        match tokens.peek() {
            Some(Token::LBracket) => {
                tokens.next();

                let index = parse_expr(tokens, diagnostics)?;

                match tokens.next() {
                    Some(Token::RBracket) => {}
                    other => return Err(format!("Expected ], got {:?}", other)),
                }

                expr = Expr::Index {
                    target: Box::new(expr),
                    index: Box::new(index),
                };
            }

            Some(Token::Dot) => {
                tokens.next();

                let field = match tokens.next() {
                    Some(Token::Ident(name)) => name,
                    other => return Err(format!("Expected ident after ., got {:?}", other)),
                }
                .clone();

                expr = Expr::Member {
                    target: Box::new(expr),
                    field,
                };
            }

            Some(Token::LParen) => {
                tokens.next();

                let mut args = vec![];

                if !matches!(tokens.peek(), Some(Token::RParen)) {
                    loop {
                        args.push(parse_expr(tokens, diagnostics)?);

                        if !matches!(tokens.peek(), Some(Token::Comma)) {
                            break;
                        }

                        tokens.next();
                    }
                }

                match tokens.next() {
                    Some(Token::RParen) => {}
                    other => return Err(format!("Expected ), got {:?}", other)),
                }

                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            }

            _ => break,
        }
    }

    Ok(expr)
}
fn parse_member_and_index_chain(
    mut expr: Expr,
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    loop {
        match tokens.peek() {
            Some(Token::LBracket) => {
                tokens.next();

                let index = parse_expr(tokens, diagnostics)?;

                match tokens.next() {
                    Some(Token::RBracket) => {
                        expr = Expr::Index {
                            target: Box::new(expr),
                            index: Box::new(index),
                        };
                    }
                    _ => return Err("Expected ']' after index".into()),
                }
            }

            Some(Token::Dot) => {
                tokens.next();

                match tokens.next() {
                    Some(Token::Ident(field)) => {
                        expr = Expr::Member {
                            target: Box::new(expr),
                            field: field.clone(),
                        };
                    }
                    _ => return Err("Expected identifier after '.'".into()),
                }
            }

            _ => break,
        }
    }

    Ok(expr)
}
fn parse_unary(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    match tokens.peek() {
        Some(Token::Minus) => {
            tokens.next();

            let expr = parse_unary(tokens, diagnostics)?;

            Ok(Expr::Unary {
                op: UnOp::Neg,
                expr: Box::new(expr),
            })
        }

        Some(Token::Not) => {
            tokens.next();

            let expr = parse_unary(tokens, diagnostics)?;

            Ok(Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(expr),
            })
        }

        Some(Token::Ampersand) => {
            tokens.next();

            let expr = parse_unary(tokens, diagnostics)?;

            Ok(Expr::Unary {
                op: UnOp::AddrOf,
                expr: Box::new(expr),
            })
        }

        _ => parse_postfix(tokens, diagnostics),
    }
}
fn parse_add_sub(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    let mut left = parse_mul_div(tokens, diagnostics)?;

    while let Some(tok) = tokens.peek() {
        let op = match tok {
            Token::Plus => BinOp::Add,
            Token::Minus => BinOp::Sub,
            _ => break,
        };

        tokens.next();

        let right = parse_mul_div(tokens, diagnostics)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}
fn parse_mul_div(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    let mut left = parse_power(tokens, diagnostics)?;

    while let Some(tok) = tokens.peek() {
        let op = match tok {
            Token::Star => BinOp::Mul,
            Token::Slash => BinOp::Div,
            Token::Mod => BinOp::Mod,
            _ => break,
        };

        tokens.next();

        let right = parse_power(tokens, diagnostics)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}
fn parse_array(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    tokens.next(); // consume '['

    let mut items = Vec::new();

    while let Some(tok) = tokens.peek() {
        if matches!(tok, Token::RBracket) {
            tokens.next();
            break;
        }

        items.push(parse_expr(tokens, diagnostics)?);

        if matches!(tokens.peek(), Some(Token::Comma)) {
            tokens.next();
        }
    }

    Ok(Expr::Array(items))
}
fn parse_power(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<Expr, String> {
    // base: unary level
    let mut left = parse_unary(tokens, diagnostics)?;

    // right-associative operator (^ or **)
    if let Some(Token::Power) = tokens.peek() {
        tokens.next();

        let right = parse_power(tokens, diagnostics);

        let right = match right {
            Ok(r) => r,
            Err(e) => return Err(e),
        };

        left = Expr::Binary {
            left: Box::new(left),
            op: BinOp::Power,
            right: Box::new(right),
        };
    }

    Ok(left)
}

mod control {
    use crate::{
        compiler::diagnostic::{Diagnostic, DiagnosticStore},
        frontend::{
            ast::Stmt,
            parser::{parse_expr, parse_stmt},
            token::Token,
            types::TokenStream,
        },
        middle::types::Span,
    };
    use std::iter::Peekable;
    pub fn parse_block(
        tokens: &mut TokenStream,
        diagnostics: &mut DiagnosticStore,
    ) -> Result<Vec<Stmt>, String> {
        match tokens.next() {
            Some(Token::LBrace) => {}
            other => return Err(format!("Expected '{{', got {:?}", other)),
        }

        let mut stmts = Vec::new();

        while let Some(tok) = tokens.peek() {
            match tok {
                Token::RBrace => {
                    tokens.next();
                    break;
                }

                Token::EOF => {
                    diagnostics.emit(Diagnostic::error(
                        "Unclosed block: expected '}'",
                        Span::default(),
                    ));

                    return Err("Unclosed block".into());
                }

                Token::Semicolon => {
                    tokens.next();
                    continue;
                }

                _ => {
                    match parse_stmt(tokens, diagnostics) {
                        Ok(stmt) => stmts.push(stmt),

                        Err(e) => {
                            diagnostics.emit(Diagnostic::error(
                                format!("Statement parse error: {}", e),
                                Span::default(),
                            ));

                            // recovery: skip token
                            tokens.next();
                        }
                    }

                    if matches!(tokens.peek(), Some(Token::Semicolon)) {
                        tokens.next();
                    }
                }
            }
        }

        Ok(stmts)
    }
    pub fn parse_if(
        tokens: &mut TokenStream,
        diagnostics: &mut DiagnosticStore,
    ) -> Result<Stmt, String> {
        println!("parse if");
        tokens.next(); // consume 'if'

        let condition = parse_expr(tokens, diagnostics)?;
        let then_branch = parse_block(tokens, diagnostics)?;

        let mut else_branch = None;

        if let Some(Token::Else) = tokens.peek() {
            tokens.next();

            if let Some(Token::If) = tokens.peek() {
                let nested = parse_if(tokens, diagnostics)?;
                else_branch = Some(vec![nested]);
            } else {
                else_branch = Some(parse_block(tokens, diagnostics)?);
            }
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }
    pub fn parse_while(
        tokens: &mut TokenStream,
        diagnostics: &mut DiagnosticStore,
    ) -> Result<Stmt, String> {
        tokens.next(); // consume 'while'

        let cond = parse_expr(tokens, diagnostics)?;
        let body = parse_block(tokens, diagnostics)?;

        Ok(Stmt::While {
            condition: cond,
            body,
        })
    }
    pub fn parse_do_while(
        tokens: &mut TokenStream,
        diagnostics: &mut DiagnosticStore,
    ) -> Result<Stmt, String> {
        tokens.next(); // consume 'do'

        let body = parse_block(tokens, diagnostics)?;

        match tokens.next() {
            Some(Token::While) => {}
            other => {
                diagnostics.emit(Diagnostic::error(
                    format!("Expected 'while' after do-block, got {:?}", other),
                    Span::default(),
                ));

                return Err("Malformed do-while".into());
            }
        }

        let condition = parse_expr(tokens, diagnostics)?;

        if let Some(Token::Semicolon) = tokens.peek() {
            tokens.next();
        }

        Ok(Stmt::DoWhile { body, condition })
    }
    pub fn parse_return(
        tokens: &mut TokenStream,
        diagnostics: &mut DiagnosticStore,
    ) -> Result<Stmt, String> {
        tokens.next(); // consume 'return'

        let value = match tokens.peek() {
            Some(Token::Semicolon) | Some(Token::RBrace) | Some(Token::EOF) | None => None,

            _ => Some(parse_expr(tokens, diagnostics)?),
        };

        // optional semicolon
        if let Some(Token::Semicolon) = tokens.peek() {
            tokens.next();
        }

        Ok(Stmt::Return { value })
    }
    pub fn parse_loop(
        tokens: &mut TokenStream,
        diagnostics: &mut DiagnosticStore,
    ) -> Result<Stmt, String> {
        tokens.next();

        let body = parse_block(tokens, diagnostics)?;

        Ok(Stmt::Loop { body })
    }
    pub fn parse_function(
        tokens: &mut TokenStream,
        diagnostics: &mut DiagnosticStore,
    ) -> Result<Stmt, String> {
        tokens.bump();
        let name = match tokens.next() {
            Some(Token::Ident(n)) => n.to_string(),
            other => {
                diagnostics.emit(Diagnostic::error(
                    format!("Expected function name, got {:?}", other),
                    Span::default(),
                ));
                return Err("Invalid function declaration".into());
            }
        };

        let mut params = Vec::new();

        match tokens.next() {
            Some(Token::LParen) => {}
            other => {
                diagnostics.emit(Diagnostic::error(
                    format!("Expected '(', got {:?}", other),
                    Span::default(),
                ));
                return Err("Invalid function parameters".into());
            }
        }

        while let Some(tok) = tokens.next() {
            match tok {
                Token::RParen => break,

                Token::Ident(param) => {
                    params.push(param.to_string());
                }

                Token::Comma => continue,

                Token::EOF => {
                    diagnostics.emit(Diagnostic::error(
                        "Unterminated parameter list",
                        Span::default(),
                    ));
                    return Err("Unexpected EOF in params".into());
                }

                _ => {
                    diagnostics.emit(Diagnostic::error("Invalid parameter list", Span::default()));
                    return Err("Invalid parameters".into());
                }
            }
        }

        let body = parse_block(tokens, diagnostics)?;

        Ok(Stmt::Function { name, params, body })
    }
}

fn is_assignable(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Var(_) | Expr::Member { .. } | Expr::Index { .. }
    )
}

// fn is_expr_start(tok: Option<&Token>) -> bool {
//     matches!(
//         tok,
//         Some(Token::Number(_))
//             | Some(Token::String(_))
//             | Some(Token::Ident(_))
//             | Some(Token::LParen)
//             | Some(Token::LBracket)
//             | Some(Token::Ampersand)
//     )
// }

// fn kind_from_token(tok: &Token) -> DeclKind {
//     match tok {
//         Token::Assign => DeclKind::MutableStatic,
//         Token::Immutable => DeclKind::Immutable,
//         Token::Dynamic => DeclKind::Dynamic,
//         _ => unreachable!(),
//     }
// }

// fn looks_like_declaration<I>(tokens: &mut Peekable<I>) -> bool
// where
//     I: Iterator<Item = Token> + Clone,
// {
//     let mut lookahead = tokens.clone();

//     match (lookahead.next(), lookahead.next()) {
//         (Some(Token::Ident(_)), Some(Token::Eq | Token::Immutable | Token::Dynamic)) => true,

//         _ => false,
//     }
// }

fn is_simple_var(expr: &Expr) -> bool {
    matches!(expr, Expr::Var(_))
}
fn flatten_assign(expr: Expr) -> Expr {
    match expr {
        Expr::Assign { left, right, op } => {
            Expr::Assign {
                left,
                right: Box::new(flatten_assign(*right)), // 👈 recursive fix
                op,
            }
        }
        other => other,
    }
}
