use serde::Serialize;
use std::iter::Peekable;

use crate::frontend::{
    ast::{AST, AssignOp, BinOp, DeclKind, Expr, Stmt, UnOp},
    lexer::lex,
    token::Token,
};

pub fn parse(tokens: Vec<Token>) -> Result<AST, String> {
    let mut tokens = tokens.into_iter().peekable();
    let mut stmts = vec![];

    while let Some(tok) = tokens.peek() {
        match tok {
            Token::EOF => {
                tokens.next();
                break; // Exit
            }
            Token::Semicolon => {
                tokens.next();
            }
            _ => {
                let stmt = parse_stmt(&mut tokens)?;
                stmts.push(stmt);

                if let Some(Token::Semicolon) = tokens.peek() {
                    tokens.next();
                }
            }
        }
    }

    let last_expr = stmts.iter().rev().find_map(|stmt| {
        if let Stmt::ExprStmt { expr } = stmt {
            Some(expr.clone())
        } else {
            None
        }
    });
    let mut ast = AST::new();
    ast.stmts = stmts;
    ast.expr = last_expr;

    Ok(ast)
}

fn parse_stmt<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    match tokens.peek() {
        Some(Token::If) => control::parse_if(tokens),
        Some(Token::While) => control::parse_while(tokens),
        Some(Token::Do) => control::parse_do_while(tokens),
        Some(Token::Return) => control::parse_return(tokens),
        Some(Token::Function) => control::parse_function(tokens),
        Some(Token::LBrace) => {
            let body = control::parse_block(tokens)?;
            Ok(Stmt::Block { body })
        }
        Some(Token::Print) => {
            tokens.next();
            Ok(Stmt::Print {
                expr: parse_expr(tokens)?,
            })
        }
        _ => {
            let expr = parse_expr(tokens)?;
            match expr {
                Expr::Assign { left, right, op } => {
                    if let Expr::Var(name) = *left {
                        let kind = match op {
                            AssignOp::Assign => DeclKind::MutableStatic,
                            AssignOp::Immutable => DeclKind::ImmutableStatic,
                            AssignOp::Dynamic => DeclKind::Dynamic,
                        };

                        return Ok(Stmt::Let {
                            name,
                            kind,
                            value: *right,
                        });
                    }

                    return Ok(Stmt::ExprStmt {
                        expr: Expr::Assign { left, right, op },
                    });
                }

                other => Ok(Stmt::ExprStmt { expr: other }),
            }
        }
    }
}

fn parse_expr<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    parse_assignment(tokens, None)
}

fn parse_assignment<I>(tokens: &mut Peekable<I>, lhs: Option<Expr>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let left = match lhs {
        Some(expr) => expr,
        None => parse_or(tokens)?,
    };

    if let Some(Token::Assign | Token::Immutable | Token::Dynamic) = tokens.peek() {
        let op = tokens.next().unwrap();

        let assign_op = match op {
            Token::Assign => AssignOp::Assign,
            Token::Immutable => AssignOp::Immutable,
            Token::Dynamic => AssignOp::Dynamic,
            _ => unreachable!(),
        };

        let right = parse_assignment(tokens, None)?;

        if !is_assignable(&left) {
            return Err("Invalid assignment target".into());
        }

        return Ok(Expr::Assign {
            left: Box::new(left),
            right: Box::new(right),
            op: assign_op,
        });
    }

    Ok(left)
}

fn parse_equality<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_comparison(tokens)?;

    while let Some(tok) = tokens.peek() {
        match tok {
            Token::Eq => {
                tokens.next();
                let right = parse_comparison(tokens)?;
                left = Expr::Binary {
                    left: Box::new(left),
                    op: BinOp::Eq,
                    right: Box::new(right),
                };
            }

            Token::Neq => {
                tokens.next();
                let right = parse_comparison(tokens)?;
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

fn parse_or<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_and(tokens)?;
    println!("OR peek: {:?}", tokens.peek());
    while let Some(Token::Or) = tokens.peek() {
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

fn parse_and<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_equality(tokens)?;
    println!("AND peek: {:?}", tokens.peek());
    while let Some(Token::And) = tokens.peek() {
        tokens.next();
        let right = parse_equality(tokens)?;
        left = Expr::Binary {
            left: Box::new(left),
            op: BinOp::And,
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_comparison<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    // Point this to the base of the math chain: parse_add_sub
    let mut left = parse_add_sub(tokens)?;

    while let Some(Token::Lt | Token::Gt) = tokens.peek() {
        let op = match tokens.next().unwrap() {
            Token::Lt => BinOp::Lt,
            Token::Gt => BinOp::Gt,
            _ => unreachable!(),
        };

        // Also update the right-hand side to parse_add_sub
        let right = parse_add_sub(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}
fn parse_primary<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    match tokens.next() {
        Some(Token::True) => Ok(Expr::Bool(true)),
        Some(Token::False) => Ok(Expr::Bool(false)),
        Some(Token::Number(n)) => Ok(Expr::Number(n)),
        Some(Token::String(s)) => Ok(Expr::String(s)),
        Some(Token::Ident(name)) => Ok(Expr::Var(name)),
        Some(Token::Ampersand) => {
            tokens.next();
            let expr = parse_primary(tokens)?;
            Ok(Expr::Unary {
                op: UnOp::AddrOf,
                expr: Box::new(expr),
            })
        }
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

        None => Err("Unexpected EOF".into()),
        Some(other) => Err(format!("Unexpected token: {:?}", other)),
    }
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
                    callee: Box::new(expr),
                    args,
                };
            }
            _ => break,
        }
    }

    Ok(expr)
}
fn parse_member_and_index_chain<I>(mut expr: Expr, tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    loop {
        match tokens.peek() {
            // Handle Indexing: expr[index]
            Some(Token::LBracket) => {
                tokens.next(); // Consume '['
                let index = parse_expr(tokens)?;
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
            // Handle Member Access: expr.field
            Some(Token::Dot) => {
                tokens.next(); // Consume '.'
                match tokens.next() {
                    Some(Token::Ident(field)) => {
                        expr = Expr::Member {
                            target: Box::new(expr),
                            field,
                        };
                    }
                    _ => return Err("Expected identifier after '.'".into()),
                }
            }
            // If no more chainable tokens, stop
            _ => break,
        }
    }
    Ok(expr)
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

fn parse_add_sub<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_mul_div(tokens)?;
    while let Some(Token::Plus | Token::Minus) = tokens.peek() {
        let op = match tokens.next().unwrap() {
            Token::Plus => BinOp::Add,
            Token::Minus => BinOp::Sub,
            _ => unreachable!(),
        };
        let right = parse_mul_div(tokens)?;
        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_mul_div<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_power(tokens)?;

    while let Some(tok) = tokens.peek() {
        let op = match tok {
            Token::Star => BinOp::Mul,
            Token::Slash => BinOp::Div,
            Token::Mod => BinOp::Mod,
            _ => break,
        };

        tokens.next();
        // Point this to parse_power as well
        let right = parse_power(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }
    Ok(left)
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

// 2. Power is the next layer down.
fn parse_power<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    // Start by checking for Unary, then Postfix
    let mut left = parse_unary(tokens)?;

    if let Some(Token::Power) = tokens.peek() {
        tokens.next();
        // Right-associative: recursive call
        let right = parse_power(tokens)?;
        left = Expr::Binary {
            left: Box::new(left),
            op: BinOp::Power,
            right: Box::new(right),
        };
    }
    Ok(left)
}
fn parse_exponentiation<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let mut left = parse_postfix(tokens)?; // Move down to postfix/primary

    if let Some(Token::Power) = tokens.peek() {
        tokens.next(); // Consume '^' or '**'
        // Recursively call parse_exponentiation for right-associativity
        let right = parse_exponentiation(tokens)?;

        left = Expr::Binary {
            left: Box::new(left),
            op: BinOp::Power,
            right: Box::new(right),
        };
    }

    Ok(left)
}
mod control {
    use std::iter::Peekable;

    use crate::frontend::{
        ast::Stmt,
        parser::{parse_expr, parse_stmt},
        token::Token,
    };

    pub fn parse_block<I>(tokens: &mut Peekable<I>) -> Result<Vec<Stmt>, String>
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
                    tokens.next();
                    break;
                }

                Token::EOF => {
                    return Err("Unclosed block: expected '}'".into());
                }

                Token::Semicolon => {
                    tokens.next();
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

    pub fn parse_if<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
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
    pub fn parse_while<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
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
    pub fn parse_do_while<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
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

    pub fn parse_return<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
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

    pub fn parse_loop<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
    where
        I: Iterator<Item = Token>,
    {
        tokens.next();

        let body = parse_block(tokens)?;

        Ok(Stmt::Loop { body })
    }

    pub fn parse_function<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
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
}

pub fn parse_let<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    let name = match tokens.next() {
        Some(Token::Ident(n)) => n,
        Some(t) => return Err(format!("Expected identifier, found {:?}", t)),
        None => return Err("Expected identifier, reached end of input".to_string()),
    };

    let kind = match tokens.peek() {
        Some(Token::Eq) => DeclKind::MutableStatic,
        Some(Token::Immutable) => DeclKind::ImmutableStatic,
        Some(Token::Dynamic) => DeclKind::Dynamic,
        Some(t) => return Err(format!("Expected assignment operator, found {:?}", t)),
        None => return Err("Expected assignment operator, reached end of input".to_string()),
    };

    tokens.next();

    let value = parse_expr(tokens)?;

    Ok(Stmt::Let { name, kind, value })
}

fn parse_declaration<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    let name = match tokens.next() {
        Some(Token::Ident(name)) => name,
        other => return Err(format!("Expected identifier, got {:?}", other)),
    };

    let kind = match tokens.next() {
        Some(Token::Eq) => DeclKind::MutableStatic,
        Some(Token::Immutable) => DeclKind::ImmutableStatic,
        Some(Token::Dynamic) => DeclKind::Dynamic,
        other => return Err(format!("Expected declaration operator, got {:?}", other)),
    };

    let value = parse_expr(tokens)?;

    Ok(Stmt::Let { name, kind, value })
}

pub fn parse_source(input: &str) -> Result<AST, String> {
    let tokens = lex(input)?;
    parse(tokens)
}

fn is_assignable(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Var(_) | Expr::Member { .. } | Expr::Index { .. }
    )
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

fn kind_from_token(tok: &Token) -> DeclKind {
    match tok {
        Token::Assign => DeclKind::MutableStatic,
        Token::Immutable => DeclKind::ImmutableStatic,
        Token::Dynamic => DeclKind::Dynamic,
        _ => unreachable!(),
    }
}

fn looks_like_declaration<I>(tokens: &mut Peekable<I>) -> bool
where
    I: Iterator<Item = Token> + Clone,
{
    let mut lookahead = tokens.clone();

    match (lookahead.next(), lookahead.next()) {
        (Some(Token::Ident(_)), Some(Token::Eq | Token::Immutable | Token::Dynamic)) => true,

        _ => false,
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
