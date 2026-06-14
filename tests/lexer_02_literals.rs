use loi::frontend::lexer::lex;
use loi::frontend::token::Token;
use std::{fs, path::Path};

mod common;
use common::lexer::LexerTestHarness;

#[test]
fn test_delimiters_snap() {
    LexerTestHarness::from_file("tests/fixtures/lexical/literals.loi")
        .assert_snapshot("literals_lexical_stream");
}

#[test]
fn test_literal_logic() {
    let content = r#"123 45.67 "string_literal" true false"#;
    let tokens = lex(content).unwrap();

    // Filter out EOF
    let lits: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::EOF)).collect();

    assert_eq!(lits.len(), 5, "Expected 5 literal tokens");

    // Verify types (Adjust Token variants to your specific Enum structure)
    assert!(lits.iter().any(|t| matches!(t, Token::Number(_))));
    assert!(lits.iter().any(|t| matches!(t, Token::Number(_))));
    assert!(lits.iter().any(|t| matches!(t, Token::String(_))));
    assert!(lits.iter().any(|t| matches!(t, Token::True)));
    assert!(lits.iter().any(|t| matches!(t, Token::False)));
}
