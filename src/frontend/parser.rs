use serde::Serialize;
use std::iter::Peekable;

use crate::frontend::{
    ast::{AssignOp, BinOp, DeclKind, Expr, Stmt, UnOp},
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

                if let Some(Token::Semicolon) = tokens.peek() {
                    tokens.next();
                }
            }
        }
    }

    Ok(AST { stmts })
}

fn parse_stmt<I>(tokens: &mut Peekable<I>) -> Result<Stmt, String>
where
    I: Iterator<Item = Token>,
{
    match tokens.peek() {
        Some(Token::If) => parse_if(tokens),
        Some(Token::While) => parse_while(tokens),
        Some(Token::Do) => parse_do_while(tokens),
        Some(Token::Return) => parse_return(tokens),
        Some(Token::Function) => parse_function(tokens),

        Some(Token::LBrace) => {
            let body = parse_block(tokens)?;
            Ok(Stmt::Block { body })
        }

        Some(Token::Print) => {
            tokens.next();
            Ok(Stmt::Print {
                expr: parse_expr(tokens)?,
            })
        }
        // 1. "Initial"
        // Fails: p07_declaration_still_creates_let
        // _ => {
        //     let expr = parse_expr(tokens)?;
        //     Ok(Stmt::ExprStmt { expr })
        // }

        // 2. Fix of p07
        // Fails p08_test_variable_declarations
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
        } // Some(Token::Ident(name)) => {
          //     let name = name.clone();
          //     tokens.next();

          //     let op = match tokens.peek() {
          //         Some(Token::Assign | Token::EqualsBang | Token::EqualsQ) => tokens.next().unwrap(),
          //         _ => {
          //             let expr = parse_expr(tokens)?;
          //             return Ok(Stmt::ExprStmt { expr });
          //         }
          //     };

          //     let kind = match op {
          //         Token::Assign => DeclKind::MutableStatic,
          //         Token::EqualsBang => DeclKind::ImmutableStatic,
          //         Token::EqualsQ => DeclKind::Dynamic,
          //         _ => unreachable!(),
          //     };

          //     let value = parse_expr(tokens)?;

          //     Ok(Stmt::Let { name, kind, value })
          // }
          // _ => {
          //     let expr = parse_expr(tokens)?;

          //     if let Expr::Assign { left, right } = expr {
          //         if let Expr::Var(name) = *left {
          //             return Ok(Stmt::Let {
          //                 name,
          //                 kind: DeclKind::MutableStatic,
          //                 value: *right,
          //             });
          //         }

          //         return Ok(Stmt::ExprStmt {
          //             expr: Expr::Assign { left, right },
          //         });
          //     }

          //     Ok(Stmt::ExprStmt { expr })
          // }
    }
}

fn parse_expr<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    parse_assignment(tokens, None)
}
fn is_assignable(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Var(_) | Expr::Member { .. } | Expr::Index { .. }
    )
}

fn parse_assignment<I>(tokens: &mut Peekable<I>, lhs: Option<Expr>) -> Result<Expr, String>
where
    I: Iterator<Item = Token>,
{
    let left = match lhs {
        Some(expr) => expr,
        None => parse_or(tokens)?,
    };

    if let Some(Token::Assign | Token::EqualsBang | Token::EqualsQ) = tokens.peek() {
        let op = tokens.next().unwrap();

        let assign_op = match op {
            Token::Assign => AssignOp::Assign,
            Token::EqualsBang => AssignOp::Immutable,
            Token::EqualsQ => AssignOp::Dynamic,
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
    let mut left = parse_term(tokens)?;

    while let Some(Token::Lt | Token::Gt) = tokens.peek() {
        let op = match tokens.next().unwrap() {
            Token::Lt => BinOp::Lt,
            Token::Gt => BinOp::Gt,
            _ => unreachable!(),
        };

        let right = parse_term(tokens)?;

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
        Some(Token::Number(n)) => Ok(Expr::Number(n)),
        Some(Token::String(s)) => Ok(Expr::String(s)),
        Some(Token::True) => Ok(Expr::Bool(true)),
        Some(Token::False) => Ok(Expr::Bool(false)),

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
    let mut left = parse_unary(tokens)?;
    while let Some(Token::Star | Token::Slash) = tokens.peek() {
        let op = match tokens.next().unwrap() {
            Token::Star => BinOp::Mul,
            Token::Slash => BinOp::Div,
            _ => unreachable!(),
        };
        let right = parse_unary(tokens)?;
        left = Expr::Binary {
            left: Box::new(left),
            op,
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
    tokens.next();

    let body = parse_block(tokens)?;

    Ok(Stmt::Loop { body })
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
        Some(Token::EqualsBang) => DeclKind::ImmutableStatic,
        Some(Token::EqualsQ) => DeclKind::Dynamic,
        Some(t) => return Err(format!("Expected assignment operator, found {:?}", t)),
        None => return Err("Expected assignment operator, reached end of input".to_string()),
    };

    tokens.next();

    let value = parse_expr(tokens)?;

    Ok(Stmt::Let { name, kind, value })
}

fn looks_like_declaration<I>(tokens: &mut Peekable<I>) -> bool
where
    I: Iterator<Item = Token> + Clone,
{
    let mut lookahead = tokens.clone();

    match (lookahead.next(), lookahead.next()) {
        (Some(Token::Ident(_)), Some(Token::Eq | Token::EqualsBang | Token::EqualsQ)) => true,

        _ => false,
    }
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
        Some(Token::EqualsBang) => DeclKind::ImmutableStatic,
        Some(Token::EqualsQ) => DeclKind::Dynamic,
        other => return Err(format!("Expected declaration operator, got {:?}", other)),
    };

    let value = parse_expr(tokens)?;

    Ok(Stmt::Let { name, kind, value })
}

pub fn parse_source(input: &str) -> Result<AST, String> {
    let tokens = lex(input)?;
    parse(tokens)
}

// fn parse_binary<I>(tokens: &mut Peekable<I>) -> Result<Expr, String>
// where
//     I: Iterator<Item = Token>,
// {
//     let mut left = parse_unary(tokens)?;

//     loop {
//         let op = match tokens.peek() {
//             Some(Token::Plus) => BinOp::Add,
//             Some(Token::Minus) => BinOp::Sub,
//             Some(Token::Star) => BinOp::Mul,
//             Some(Token::Slash) => BinOp::Div,
//             _ => break,
//         };

//         tokens.next(); // consume operator

//         let right = parse_unary(tokens)?;

//         left = Expr::Binary {
//             left: Box::new(left),
//             op,
//             right: Box::new(right),
//         };
//     }

//     Ok(left)
// }

fn kind_from_token(tok: &Token) -> DeclKind {
    match tok {
        Token::Assign => DeclKind::MutableStatic,
        Token::EqualsBang => DeclKind::ImmutableStatic,
        Token::EqualsQ => DeclKind::Dynamic,
        _ => unreachable!(),
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
