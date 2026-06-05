// src/frontend/lexer.rs

use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,

    Equals,
    EqualsBang,
    EqualsQ,

    ColonEq,

    Print,
    EOF,
    Semicolon,
    LParen,
    RParen,
}
fn lex_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<f64, String> {
    let mut s = String::new();
    let mut has_dot = false;

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            s.push(c);
            chars.next();
        } else if c == '.' {
            if has_dot {
                return Err("Multiple dots in number".into());
            }
            has_dot = true;
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }

    s.parse::<f64>().map_err(|e| format!("Invalid number: {e}"))
}
pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '#' => {
                chars.next(); // consume '#'

                // skip until end of line
                while let Some(ch) = chars.next() {
                    if ch == '\n' {
                        break;
                    }
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let token = match ident.as_str() {
                    "print" => Token::Print,
                    _ => Token::Ident(ident),
                };

                tokens.push(token);
            }

            '0'..='9' => {
                let num = lex_number(&mut chars)?;
                tokens.push(Token::Number(num));
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }

            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }

            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }

            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            '=' => {
                chars.next();

                match chars.peek() {
                    Some('!') => {
                        chars.next();
                        tokens.push(Token::EqualsBang);
                    }
                    Some('?') => {
                        chars.next();
                        tokens.push(Token::EqualsQ);
                    }
                    Some('=') => return Err("Unsupported ==".to_owned()),
                    _ => tokens.push(Token::Equals),
                }
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }

            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }

            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }

            ' ' | '\n' | '\t' => {
                chars.next(); // skip whitespace
            }

            _ => return Err(format!("Unknown character: {}", c)),
        }
    }

    tokens.push(Token::EOF);

    Ok(tokens)
}

#[test]
fn lex_number_() {
    let tokens = lex("123").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0], Token::Number(123.0));
    assert_eq!(tokens[1], Token::EOF);
}
