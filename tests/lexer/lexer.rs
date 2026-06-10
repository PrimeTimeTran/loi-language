use std::{fs, path::Path};

use loi::frontend::lexer::lex;
use loi::frontend::token::Token;

#[test]
fn lex_number() {
    let tokens = lex("123").unwrap();

    assert_eq!(tokens, vec![Token::Number(123.0), Token::EOF]);
}

#[test]
fn lex_basic_math() {
    let tokens = lex("1 + 2").unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Number(1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::EOF
        ]
    );
}

#[test]
fn lex_subtraction_multiplication_division() {
    let tokens = lex("10 - 2 * 3 / 4").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Number(10.0),
            Token::Minus,
            Token::Number(2.0),
            Token::Star,
            Token::Number(3.0),
            Token::Slash,
            Token::Number(4.0),
            Token::EOF
        ]
    );
}

#[test]
fn lex_parentheses() {
    let tokens = lex("(1 + 2) * 3").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::LParen,
            Token::Number(1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::RParen,
            Token::Star,
            Token::Number(3.0),
            Token::EOF
        ]
    );
}

#[test]
fn lex_nested_parentheses() {
    let tokens = lex("((1 + 2) * (3 + 4))").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::LParen,
            Token::LParen,
            Token::Number(1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::RParen,
            Token::Star,
            Token::LParen,
            Token::Number(3.0),
            Token::Plus,
            Token::Number(4.0),
            Token::RParen,
            Token::RParen,
            Token::EOF
        ]
    );
}

#[test]
fn lex_whitespace_heavy_input() {
    let tokens = lex("   1    +     2   *   3   ").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Number(1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::Star,
            Token::Number(3.0),
            Token::EOF
        ]
    );
}

#[test]
fn lex_multiple_digits() {
    let tokens = lex("123 + 4567").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Number(123.0),
            Token::Plus,
            Token::Number(4567.0),
            Token::EOF
        ]
    );
}

#[test]
fn lex_complex_expression() {
    let tokens = lex("(1 + 2) * (3 - 4) / 5 + 6").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::LParen,
            Token::Number(1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::RParen,
            Token::Star,
            Token::LParen,
            Token::Number(3.0),
            Token::Minus,
            Token::Number(4.0),
            Token::RParen,
            Token::Slash,
            Token::Number(5.0),
            Token::Plus,
            Token::Number(6.0),
            Token::EOF
        ]
    );
}

#[test]
fn lex_empty_input() {
    let tokens = lex("").unwrap();

    assert_eq!(tokens, vec![Token::EOF]);
}

#[test]
fn lex_negative_number() {
    let tokens = lex("-123").unwrap();

    assert_eq!(tokens, vec![Token::Minus, Token::Number(123.0), Token::EOF]);
}

#[test]
fn lex_float_number() {
    let tokens = lex("3.14 + 2.0").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Number(3.14),
            Token::Plus,
            Token::Number(2.0),
            Token::EOF
        ]
    );
}

#[test]
fn lex_invalid_character() {
    let result = lex("1 + @");

    assert!(result.is_err());
}
