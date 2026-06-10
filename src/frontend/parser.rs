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

pub fn parse_source(input: &str) -> Result<AST, String> {
    let tokens = lex(input)?;
    parse(tokens)
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
        Some(Token::LParen) => {
            let expr = parse_expr(tokens)?;

            match tokens.next() {
                Some(Token::RParen) => Ok(expr),
                _ => Err("Expected ')'".into()),
            }
        }
        Some(Token::OpenBracket) => {
            let mut items = vec![];

            while let Some(tok) = tokens.peek() {
                if matches!(tok, Token::CloseBracket) {
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

fn parse_expr<I>(tokens: &mut std::iter::Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    parse_comparison(tokens)
}
fn parse_stmt<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    let peek = tokens.peek().cloned();

    match peek {
        // -------------------------
        // CONTROL FLOW
        // -------------------------
        Some(Token::If) => return parse_if(tokens),
        // Some(Token::ElseIf) => return parse_elseif(tokens),
        Some(Token::Unless) => {
            tokens.next();

            let cond = parse_expr(tokens)?;
            let body = parse_block(tokens)?;

            let negated = Expr::Unary {
                op: UnOp::Not,
                expr: Box::new(cond),
            };

            Ok(Stmt::If {
                condition: negated,
                then_branch: body,
                else_branch: None,
            })
        }
        Some(Token::Loop) => return parse_loop(tokens),
        Some(Token::While) => return parse_while(tokens),
        Some(Token::Do) => return parse_do_while(tokens),
        Some(Token::Return) => return parse_return(tokens),

        // -------------------------
        // FUNCTIONS
        // -------------------------
        Some(Token::Function) => return parse_function(tokens),

        // -------------------------
        // BUILTINS
        // -------------------------
        Some(Token::Print) => {
            tokens.next();
            let expr = parse_expr(tokens)?;
            return Ok(Stmt::Print { expr });
        }

        // -------------------------
        // IDENT / ASSIGNMENT
        // -------------------------
        Some(Token::Ident(name)) => {
            let name = name.clone();
            tokens.next();

            match tokens.peek() {
                Some(Token::Equals) | Some(Token::EqualsBang) | Some(Token::EqualsQ) => {
                    let kind = match tokens.next() {
                        Some(Token::Equals) => DeclKind::MutableStatic,
                        Some(Token::EqualsBang) => DeclKind::ImmutableStatic,
                        Some(Token::EqualsQ) => DeclKind::Dynamic,
                        other => return Err(format!("bad assign {:?}", other)),
                    };

                    let value = parse_expr(tokens)?;

                    if let Some(Token::Semicolon) = tokens.peek() {
                        tokens.next();
                    }

                    return Ok(Stmt::Let { name, kind, value });
                }

                _ => {
                    let expr = parse_expr_with_head(Expr::Var(name), tokens)?;
                    return Ok(Stmt::ExprStmt { expr });
                }
            }
        }

        Some(Token::EOF) => {
            return Ok(Stmt::ExprStmt {
                expr: Expr::Number(0.0),
            });
        }
        _ if is_expr_start(tokens.peek()) => {
            let expr = parse_expr(tokens)?;
            Ok(Stmt::ExprStmt { expr })
        }
        _ => Err(format!("Unexpected token in stmt: {:?}", tokens.peek())),
    }
}
fn parse_comparison<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_term(tokens)?;

    loop {
        let op = match tokens.peek() {
            Some(Token::Equality) => BinOp::Eq,
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
        if matches!(tok, Token::CloseBracket) {
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

fn parse_expr_with_head<I>(head: Expr, tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    // treat `head` as already-parsed primary
    let mut left = head;

    loop {
        let op = match tokens.peek() {
            Some(Token::Plus) => BinOp::Add,
            Some(Token::Minus) => BinOp::Sub,
            Some(Token::Star) => BinOp::Mul,
            Some(Token::Slash) => BinOp::Div,
            _ => break,
        };

        tokens.next();

        let right = parse_primary(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
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

        _ => parse_primary(tokens),
    }
}

fn is_expr_start(tok: Option<&Token>) -> bool {
    matches!(
        tok,
        Some(Token::Number(_))
            | Some(Token::String(_))
            | Some(Token::Ident(_))
            | Some(Token::LParen)
            | Some(Token::OpenBracket)
            | Some(Token::Ampersand)
    )
}
