use serde::Serialize;
use std::iter::Peekable;

use crate::{
    compiler::diagnostic::{self, Diagnostic, DiagnosticStore},
    frontend::{
        ast::{AST, AssignOp, BinOp, DeclKind, Expr, Stmt, UnOp},
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
        parse(&mut tokens, diagnostics)
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

pub fn parse(
    tokens: &mut TokenStream,
    diagnostics: &mut DiagnosticStore,
) -> Result<AST, DiagnosticStore> {
    let mut stmts = Vec::new();
    println!("PARSE PARSE START");

    while let Some(tok) = tokens.peek() {
        if matches!(tok, Token::EOF) {
            break;
        }

        match parse_stmt(tokens, diagnostics) {
            Ok(stmt) => {
                stmts.push(stmt);
            }
            Err(_) => {
                let fatal = diagnostics.emit(Diagnostic::error(
                    "Failed to parse statement",
                    Span::default(),
                ));

                if fatal {
                    return Err(diagnostics.clone());
                }

                tokens.bump();
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

    Ok(AST::new(stmts))
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

    let last_expr = stmts.iter().rev().find_map(|stmt| {
        if let Stmt::ExprStmt { expr } = stmt {
            Some(expr.clone())
        } else {
            None
        }
    });

    Ok(AST::new(stmts))
}

fn parse_stmt(tokens: &mut TokenStream, diagnostics: &mut DiagnosticStore) -> Result<Stmt, String> {
    println!("parse_stmt at: {:?}", tokens.peek());
    match tokens.peek() {
        Some(Token::Print) => {
            println!("matched print");
            tokens.bump();

            Ok(Stmt::Print {
                expr: parse_expr(tokens, diagnostics)?,
            })
        }
        Some(Token::If) => control::parse_if(tokens, diagnostics),
        Some(Token::While) => control::parse_while(tokens, diagnostics),
        Some(Token::Do) => control::parse_do_while(tokens, diagnostics),
        Some(Token::Return) => control::parse_return(tokens, diagnostics),
        Some(Token::Function) => control::parse_function(tokens, diagnostics),

        Some(Token::LBrace) => {
            println!("parse l brace");
            tokens.bump(); // consume '{'
            let body = control::parse_block(tokens, diagnostics)?;
            Ok(Stmt::Block { body })
        }
        _ => {
            println!("parse last arm of statement");
            let expr = parse_expr(tokens, diagnostics)?;

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

                    Ok(Stmt::ExprStmt {
                        expr: Expr::Assign { left, right, op },
                    })
                }

                other => Ok(Stmt::ExprStmt { expr: other }),
            }
        }
    }
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
    if let Some(Token::Assign | Token::Immutable | Token::Dynamic) = tokens.peek() {
        let op = tokens.next().unwrap(); // or tokens.bump()

        let assign_op = match op {
            Token::Assign => AssignOp::Assign,
            Token::Immutable => AssignOp::Immutable,
            Token::Dynamic => AssignOp::Dynamic,
            _ => unreachable!(),
        };

        let right = parse_assignment(tokens, None, diagnostics)?;

        if !is_assignable(&left) {
            diagnostics.emit(Diagnostic::error(
                "Invalid assignment target",
                Span::default(),
            ));

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
        Some(Token::Number(n)) => Ok(Expr::Number(n.clone())),
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

pub fn parse_source(input: &str) -> Result<AST, String> {
    let mut diagnostics = DiagnosticStore::default();

    let tokens = lex(input).map_err(|e| e.to_string())?;

    let mut stream = TokenStream::new(tokens);

    let ast = parse(&mut stream, &mut diagnostics).map_err(|_| "Parse error".to_string())?;

    // if !diagnostics.is_empty() {
    //     return Err("Diagnostics emitted during parse".into());
    // }

    Ok(ast)
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

// #[test]
// fn parse_simple_program() {
//     let tokens = lex("x = 1 + 2;").unwrap();
//     let ast = parse(tokens).unwrap();

//     assert_eq!(ast.stmts.len(), 1);

//     match &ast.stmts[0] {
//         Stmt::Let { name, kind, value } => {
//             assert_eq!(name, "x");
//             assert!(matches!(kind, DeclKind::MutableStatic));
//         }
//         _ => panic!("Expected Let statement"),
//     }
// }
