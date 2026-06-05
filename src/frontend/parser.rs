use serde::Serialize;
use std::iter::Peekable;

use crate::frontend::{
    ast::{BinOp, DeclKind, Expr, Stmt},
    lexer::{Token, lex},
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
                tokens.next(); // skip empty semicolons safely
            }

            _ => {
                let stmt = parse_stmt(&mut tokens)?;

                // enforce semicolon AFTER statement
                if let Some(Token::Semicolon) = tokens.peek() {
                    tokens.next();
                }

                stmts.push(stmt);
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
        // Some(Token::Number(n)) => Ok(Expr::Number(n)),
        Some(Token::Number(n)) => Ok(Expr::Number(n)),

        Some(Token::Ident(name)) => Ok(Expr::Var(name)),

        Some(other) => Err(format!("Unexpected token: {:?}", other)),

        None => Err("Unexpected EOF".into()),
    }
}
fn parse_factor<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_primary(tokens)?;

    loop {
        let op = match tokens.peek() {
            Some(Token::Star) => BinOp::Mul,
            Some(Token::Slash) => BinOp::Div,
            _ => break,
        };

        tokens.next(); // consume operator

        let right = parse_primary(tokens)?;

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
    parse_term(tokens)
}
fn parse_stmt<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    match tokens.peek() {
        Some(Token::Ident(name)) => {
            let name = name.clone();
            tokens.next();

            let kind = match tokens.peek() {
                Some(Token::Equals) => {
                    tokens.next();
                    DeclKind::MutableStatic
                }
                Some(Token::EqualsBang) => {
                    tokens.next();
                    DeclKind::ImmutableStatic
                }
                Some(Token::EqualsQ) => {
                    tokens.next();
                    DeclKind::Dynamic
                }
                _ => return Err("Expected =, =!, or =? after identifier".into()),
            };

            let value = parse_expr(tokens)?;

            // consume semicolon HERE (only place)
            if let Some(Token::Semicolon) = tokens.peek() {
                tokens.next();
            }

            Ok(Stmt::Let { name, kind, value })
        }

        Some(Token::Print) => {
            tokens.next();
            let expr = parse_expr(tokens)?;
            Ok(Stmt::Print { expr })
        }

        _ => {
            let expr = parse_expr(tokens)?;

            Ok(Stmt::ExprStmt { expr })
        }
    }
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
