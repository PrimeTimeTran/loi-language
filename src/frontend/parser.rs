use std::iter::Peekable;

use crate::{
    frontend::{
        ast::{Expr, Stmt},
        lexer::Token,
    },
    middle::ir::BinOp,
};

#[derive(Debug, Clone)]
pub struct AST {
    pub stmts: Vec<Stmt>,
}

pub fn parse(tokens: Vec<Token>) -> Result<AST, String> {
    let mut tokens = tokens.into_iter().peekable();
    println!("{:?}", tokens);
    let mut stmts = vec![];

    while let Some(tok) = tokens.peek() {
        match tok {
            Token::EOF => break,
            _ => {
                stmts.push(parse_stmt(&mut tokens)?);
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

            match tokens.peek() {
                Some(Token::ColonEq) | Some(Token::Equals) => {
                    tokens.next();
                    let expr = parse_expr(tokens)?;
                    Ok(Stmt::Assign { name, expr })
                }

                _ => Err("Expected := or = after identifier".into()),
            }
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
    let tokens = crate::frontend::lexer::lex("x := 1 + 2").unwrap();
    let ast = parse(tokens).unwrap();

    assert_eq!(ast.stmts.len(), 1);

    match &ast.stmts[0] {
        Stmt::Assign { name, .. } => {
            assert_eq!(name, "x");
        }
        _ => panic!("Expected assignment"),
    }
}
