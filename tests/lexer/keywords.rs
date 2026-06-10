use std::{fs, path::Path};

use loi::frontend::lexer::lex;
use loi::frontend::token::Token;

#[test]
fn test_keyword_tokenization_snap() {
    let path = Path::new("tests/fixtures/lexical/keywords.loi");
    let content = fs::read_to_string(path).expect("Failed to read fixture");
    let tokens = lex(&content).expect("Lexer failed");

    // This command replaces your manual assertions.
    // It captures the entire 'tokens' vector and saves/compares it.
    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_keyword_tokenization() {
    let path = Path::new("tests/fixtures/lexical/keywords.loi");
    let content = fs::read_to_string(path).unwrap();
    let tokens = lex(&content).unwrap();

    // Assert specific keyword tokens exist in the stream
    assert!(tokens.contains(&Token::If));
    assert!(tokens.contains(&Token::While));
    assert!(tokens.contains(&Token::Function));
    assert!(tokens.contains(&Token::Return));

    // Ensure identifiers aren't being mis-lexed as keywords
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t, Token::Ident(s) if s == "x"))
    );
}
