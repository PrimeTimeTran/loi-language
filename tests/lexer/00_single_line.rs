use loi::frontend::lexer::lex;
use loi::frontend::token::Token;
use std::{fs, path::Path};

mod common {
    include!("../00_common/mod.rs");
}
use common::lexer::LexerTestHarness;

#[test]
fn test_delimiters_snap() {
    LexerTestHarness::from_file("tests/fixtures/lexical/single_line.loi")
        .assert_snapshot("single_line_stream");
}

#[test]
fn test_single_line_comments_logic() {
    let path = Path::new("tests/fixtures/lexical/single_line.loi");
    let content = fs::read_to_string(path).unwrap();
    let tokens = lex(&content).unwrap();

    // Verify comments were skipped (comment_tokens was empty as you saw)
    let has_comments = tokens.iter().any(|t| matches!(t, Token::LineNote));
    assert!(!has_comments, "Comments should have been skipped!");

    // Verify the code logic is still intact
    assert!(tokens.contains(&Token::Ident("x".to_string())));
    assert!(tokens.contains(&Token::Ident("y".to_string())));
    assert!(tokens.contains(&Token::EOF));
}
