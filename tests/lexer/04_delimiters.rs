use loi::frontend::lexer::lex;
use loi::frontend::token::Token;
use std::{fs, path::Path};

mod harness;
use crate::harness::lexer::LexerTestHarness;

#[test]
fn test_delimiters_snap() {
    LexerTestHarness::from_file("tests/fixtures/lexical/delimiters.loi")
        .assert_snapshot("delimiters_lexical_stream");
}

#[test]
fn test_delimiter_logic() {
    let content = "( ) { } [ ] , ;";
    let tokens = lex(content).unwrap();

    // Verify all 7 delimiters are present (ignoring EOF)
    let delimiter_tokens: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::EOF)).collect();

    assert_eq!(
        delimiter_tokens.len(),
        8,
        "Expected exactly 8 delimiter tokens"
    );

    // Check for specific tokens (adjust these to match your enum names!)
    assert!(delimiter_tokens.contains(&&Token::LParen));
    assert!(delimiter_tokens.contains(&&Token::RParen));
    assert!(delimiter_tokens.contains(&&Token::LBrace));
    assert!(delimiter_tokens.contains(&&Token::RBrace));
    assert!(delimiter_tokens.contains(&&Token::LBracket));
    assert!(delimiter_tokens.contains(&&Token::RBracket));
    assert!(delimiter_tokens.contains(&&Token::Comma));
    assert!(delimiter_tokens.contains(&&Token::Semicolon));
}
