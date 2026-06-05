use std::{iter::Peekable, str::Chars};

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
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
    #[token(",")]
    Comma,
    Print,
    EOF,
    Semicolon,
    String(String),
    LBrace,
    RBrace,
    LParen,
    RParen,
}
