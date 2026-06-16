use loi::frontend::lexer::lex;
use loi::frontend::token::Token;
use std::{fs, path::Path};

mod common {
    include!("../common/mod.rs");
}
use common::snapshot::SNAP_LEXER;

#[test]
fn test_single_line_comments_logic() {
    let path = Path::new("tests/fixtures/lexical/single_line.loi");
    let content = fs::read_to_string(path).unwrap();
    let tokens = lex(&content).unwrap();
    let has_comments = tokens.iter().any(|t| matches!(t, Token::LineNote));
    assert!(!has_comments, "Comments should have been skipped!");
    assert!(tokens.contains(&Token::Ident("x".to_string())));
    assert!(tokens.contains(&Token::Ident("y".to_string())));
    assert!(tokens.contains(&Token::EOF));

    SNAP_LEXER.assert("tokens", "# ", " HELLO WORLD");
}
