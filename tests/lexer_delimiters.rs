use loi::frontend::lexer::lex;
use loi::frontend::token::Token;
use std::{fs, path::Path};

#[test]
fn test_delimiters_snap() {
    let path = Path::new("tests/fixtures/lexical/delimiters.loi");
    let content = fs::read_to_string(path).expect("Failed to read fixture");
    let tokens = lex(&content).expect("Lexer failed");

    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_delimiter_logic() {
    let content = "( ) { } [ ] , ;";
    let tokens = lex(content).unwrap();

    let delimiter_tokens: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::EOF)).collect();

    assert_eq!(
        delimiter_tokens.len(),
        8,
        "Expected exactly 8 delimiter tokens"
    );

    assert!(delimiter_tokens.contains(&&Token::LParen));
    assert!(delimiter_tokens.contains(&&Token::RParen));
    assert!(delimiter_tokens.contains(&&Token::LBrace));
    assert!(delimiter_tokens.contains(&&Token::RBrace));
    assert!(delimiter_tokens.contains(&&Token::LBracket));
    assert!(delimiter_tokens.contains(&&Token::RBracket));
    assert!(delimiter_tokens.contains(&&Token::Comma));
    assert!(delimiter_tokens.contains(&&Token::Semicolon));
}
