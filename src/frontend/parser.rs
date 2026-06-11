use serde::Serialize;
use std::iter::Peekable;

use crate::frontend::{
    ast::{BinOp, DeclKind, Expr, Stmt, UnOp},
    lexer::lex,
    token::Token,
};

#[derive(Debug, Serialize)]
pub struct AST {
    pub stmts: Vec<Stmt>,
}

pub fn parse(tokens: Vec<Token>) -> Result<AST, String> {
    let mut tokens = tokens.into_iter().peekable();
    let mut stmts = vec![];

    while let Some(tok) = tokens.peek() {
        match tok {
            Token::EOF => break,

            Token::Semicolon => {
                tokens.next();
            }

            _ => {
                let stmt = parse_stmt(&mut tokens)?;
                stmts.push(stmt);

                // optional semicolon after statement
                if let Some(Token::Semicolon) = tokens.peek() {
                    tokens.next();
                }
            }
        }
    }

    Ok(AST { stmts })
}

fn parse_primary<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    match tokens.next() {
        Some(Token::True) => Ok(Expr::Bool(true)),
        Some(Token::False) => Ok(Expr::Bool(false)),
        Some(Token::LParen) => {
            let expr = parse_expr(tokens)?;

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

                items.push(parse_expr(tokens)?);

                if matches!(tokens.peek(), Some(Token::Comma)) {
                    tokens.next();
                }
            }

            Ok(Expr::Array(items))
        }
        Some(Token::Ampersand) => {
            tokens.next();
            let expr = parse_primary(tokens)?;
            Ok(Expr::Unary {
                op: UnOp::AddrOf,
                expr: Box::new(expr),
            })
        }

        Some(Token::Number(n)) => Ok(Expr::Number(n)),
        Some(Token::String(s)) => Ok(Expr::String(s)),
        Some(Token::Ident(name)) => Ok(Expr::Var(name)),

        None => Err("Unexpected EOF".into()),
        Some(other) => Err(format!("Unexpected token: {:?}", other)),
    }
}

fn parse_stmt<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    match tokens.peek().cloned() {
        Some(Token::If) => parse_if(tokens),
        Some(Token::While) => parse_while(tokens),
        Some(Token::Do) => parse_do_while(tokens),
        Some(Token::Return) => parse_return(tokens),
        Some(Token::Function) => parse_function(tokens),
        Some(Token::Print) => {
            tokens.next();
            let expr = parse_expr(tokens)?;
            Ok(Stmt::Print { expr })
        }
        Some(Token::Ident(_)) => {
            let expr = parse_expr(tokens)?;
            Ok(Stmt::ExprStmt { expr })
        }
        // Some(Token::Ident(name)) => {
        //     let name = name.clone();
        //     tokens.next(); // consume ident ONCE

        //     match tokens.peek() {
        //         // assignment
        //         Some(Token::Equals) | Some(Token::EqualsBang) | Some(Token::EqualsQ) => {
        //             let kind = match tokens.next() {
        //                 Some(Token::Equals) => DeclKind::MutableStatic,
        //                 Some(Token::EqualsBang) => DeclKind::ImmutableStatic,
        //                 Some(Token::EqualsQ) => DeclKind::Dynamic,
        //                 other => return Err(format!("bad assign {:?}", other)),
        //             };

        //             let value = parse_expr(tokens)?;

        //             Ok(Stmt::Let { name, kind, value })
        //         }

        //         _ => {
        //             let expr = parse_expr(tokens)?;
        //             Ok(Stmt::ExprStmt { expr })
        //         }
        //     }
        // }

        // -------------------------
        // EOF safety
        // -------------------------
        Some(Token::EOF) => Ok(Stmt::ExprStmt {
            expr: Expr::Number(0.0),
        }),

        // -------------------------
        // fallback: expression statement
        // -------------------------
        Some(_) => {
            let expr = parse_expr(tokens)?;
            Ok(Stmt::ExprStmt { expr })
        }

        None => Err("Unexpected end of input".to_string()),
    }
}
fn parse_expr<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    parse_assignment(tokens)
}

fn is_assignable(expr: &Expr) -> bool {
    match expr {
        Expr::Var(_) => true,
        Expr::Member { .. } => true,
        Expr::Index { .. } => true,
        Expr::Binary { .. } => false,
        Expr::Unary { .. } => false,
        Expr::Call { .. } => false,
        Expr::Array(_) => false,
        Expr::Number(_) => false,
        Expr::Bool(_) => false,
        Expr::String(_) => false,

        _ => false,
    }
}
fn parse_assignment<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let left = parse_or(tokens)?; // ✅ FULL EXPRESSION CHAIN

    if let Some(Token::Equals) = tokens.peek() {
        tokens.next();

        let right = parse_assignment(tokens)?;

        if is_assignable(&left) {
            return Ok(Expr::Assign {
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        return Err("Invalid assignment target".into());
    }

    Ok(left)
}
fn parse_postfix<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut expr = parse_primary(tokens)?;

    loop {
        match tokens.peek() {
            Some(Token::LBracket) => {
                tokens.next();
                let index = parse_expr(tokens)?;

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
                };

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
                        args.push(parse_expr(tokens)?);

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
                    name: match expr {
                        Expr::Var(n) => n,
                        _ => return Err("Invalid call target".into()),
                    },
                    args,
                };
            }

            _ => break,
        }
    }

    Ok(expr)
}
fn parse_binary<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_unary(tokens)?;

    loop {
        let op = match tokens.peek() {
            Some(Token::Plus) => BinOp::Add,
            Some(Token::Minus) => BinOp::Sub,
            Some(Token::Star) => BinOp::Mul,
            Some(Token::Slash) => BinOp::Div,
            _ => break, // <- ONLY operators
        };

        tokens.next(); // consume operator

        let right = parse_unary(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_unary<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    match tokens.peek() {
        Some(Token::Minus) => {
            tokens.next();
            let expr = parse_unary(tokens)?;
            Ok(Expr::Unary {
                op: UnOp::Neg,
                expr: Box::new(expr),
            })
        }

        Some(Token::Not) => {
            tokens.next();
            let expr = parse_unary(tokens)?;
            Ok(Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(expr),
            })
        }

        Some(Token::Ampersand) => {
            tokens.next();
            let expr = parse_unary(tokens)?;
            Ok(Expr::Unary {
                op: UnOp::AddrOf,
                expr: Box::new(expr),
            })
        }

        _ => parse_postfix(tokens),
    }
}

fn is_expr_start(tok: Option<&Token>) -> bool {
    matches!(
        tok,
        Some(Token::Number(_))
            | Some(Token::String(_))
            | Some(Token::Ident(_))
            | Some(Token::LParen)
            | Some(Token::LBracket)
            | Some(Token::Ampersand)
    )
}

fn parse_factor<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_unary(tokens)?;

    loop {
        let op = match tokens.peek() {
            Some(Token::Star) => BinOp::Mul,
            Some(Token::Slash) => BinOp::Div,
            _ => break,
        };

        tokens.next();

        let right = parse_unary(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}
fn parse_term<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_factor(tokens)?;

    loop {
        let op = match tokens.peek() {
            Some(Token::Plus) => BinOp::Add,
            Some(Token::Minus) => BinOp::Sub,
            _ => break,
        };

        tokens.next(); // consume operator

        let right = parse_factor(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_comparison<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_term(tokens)?;

    loop {
        let op = match tokens.peek() {
            Some(Token::BooleanEquality) => BinOp::Eq,
            Some(Token::NotEqual) => BinOp::Neq,
            Some(Token::LessThan) => BinOp::Lt,
            Some(Token::GreaterThan) => BinOp::Gt,
            _ => break,
        };

        tokens.next();

        let right = parse_term(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}
fn parse_and<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_comparison(tokens)?;

    while let Some(Token::BooleanAnd) = tokens.peek() {
        tokens.next();

        let right = parse_comparison(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op: BinOp::And,
            right: Box::new(right),
        };
    }

    Ok(left)
}
fn parse_or<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_and(tokens)?;

    while let Some(Token::BooleanOr) = tokens.peek() {
        tokens.next();

        let right = parse_and(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op: BinOp::Or,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_function<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    tokens.next(); // consume fn

    // function name
    let name = match tokens.next() {
        Some(Token::Ident(n)) => n,
        other => return Err(format!("Expected function name, got {:?}", other)),
    };

    // ---- params ----
    let mut params = vec![];

    match tokens.next() {
        Some(Token::LParen) => {}
        other => return Err(format!("Expected '(', got {:?}", other)),
    }

    while let Some(tok) = tokens.peek() {
        match tok {
            Token::RParen => {
                tokens.next();
                break;
            }

            Token::Ident(p) => {
                params.push(p.clone());
                tokens.next();

                if let Some(Token::Comma) = tokens.peek() {
                    tokens.next();
                }
            }

            _ => return Err("Invalid parameter list".into()),
        }
    }

    // ---- body ----
    let body = parse_block(tokens)?;

    Ok(Stmt::Function { name, params, body })
}

fn parse_while<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    tokens.next(); // consume while

    let cond = parse_expr(tokens)?;
    let body = parse_block(tokens)?;

    Ok(Stmt::While {
        condition: cond,
        body,
    })
}

fn parse_array<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    tokens.next(); // [

    let mut items = vec![];

    while let Some(tok) = tokens.peek() {
        if matches!(tok, Token::RBracket) {
            tokens.next();
            break;
        }

        items.push(parse_expr(tokens)?);

        if matches!(tokens.peek(), Some(Token::Comma)) {
            tokens.next();
        }
    }

    Ok(Expr::Array(items))
}

fn parse_block<I>(tokens: &mut Peekable<I>) -> Result<Vec<Stmt>, String>
where
    I: Iterator<Item = Token>,
{
    match tokens.next() {
        Some(Token::LBrace) => {}
        other => return Err(format!("Expected '{{', got {:?}", other)),
    }

    let mut stmts = vec![];

    while let Some(tok) = tokens.peek() {
        match tok {
            Token::RBrace => {
                tokens.next(); // consume }
                break;
            }

            Token::EOF => {
                return Err("Unclosed block: expected '}'".into());
            }

            Token::Semicolon => {
                tokens.next(); // skip empty ;
                continue;
            }

            _ => {
                let stmt = parse_stmt(tokens)?;
                stmts.push(stmt);

                // optional semicolon after statement
                if let Some(Token::Semicolon) = tokens.peek() {
                    tokens.next();
                }
            }
        }
    }

    Ok(stmts)
}

fn parse_do_while<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    tokens.next(); // do

    let body = parse_block(tokens)?;

    match tokens.next() {
        Some(Token::While) => {}
        other => return Err(format!("Expected 'while' after do-block, got {:?}", other)),
    }

    let condition = parse_expr(tokens)?;

    if let Some(Token::Semicolon) = tokens.peek() {
        tokens.next();
    }

    Ok(Stmt::DoWhile { body, condition })
}

fn parse_if<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    tokens.next();
    let condition = parse_expr(tokens)?;
    let then_branch = parse_block(tokens)?;
    let mut else_branch = None;

    if let Some(Token::Else) = tokens.peek() {
        tokens.next();

        if let Some(Token::If) = tokens.peek() {
            let nested = parse_if(tokens)?;
            else_branch = Some(vec![nested]);
        } else {
            else_branch = Some(parse_block(tokens)?);
        }
    }

    Ok(Stmt::If {
        condition,
        then_branch,
        else_branch,
    })
}

fn parse_return<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    tokens.next(); // consume "return"

    let value = match tokens.peek() {
        Some(Token::Semicolon) | Some(Token::RBrace) | Some(Token::EOF) => None,
        _ => Some(parse_expr(tokens)?),
    };

    // optional semicolon
    if let Some(Token::Semicolon) = tokens.peek() {
        tokens.next();
    }

    Ok(Stmt::Return { value })
}

fn parse_loop<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    tokens.next(); // consume "loop"

    let body = parse_block(tokens)?;

    Ok(Stmt::Loop { body })
}

pub fn parse_source(input: &str) -> Result<AST, String> {
    let tokens = lex(input)?;
    parse(tokens)
}

#[test]
fn parse_simple_program() {
    let tokens = lex("x = 1 + 2;").unwrap();
    let ast = parse(tokens).unwrap();

    assert_eq!(ast.stmts.len(), 1);

    match &ast.stmts[0] {
        Stmt::Let { name, kind, value } => {
            assert_eq!(name, "x");
            assert!(matches!(kind, DeclKind::MutableStatic));
        }
        _ => panic!("Expected Let statement"),
    }
}
