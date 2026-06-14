use loi::frontend::lexer::lex;
use loi::frontend::token::Token;
use std::{fs, path::Path};

mod common;
use common::lexer::LexerTestHarness;

#[test]
fn test_keyword_tokenization_snap() {
    LexerTestHarness::from_file("tests/fixtures/lexical/keywords.loi")
        .assert_snapshot("keywords_lexical_stream");
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
