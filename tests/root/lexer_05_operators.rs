use loi::frontend::lexer::lex;
use loi::frontend::token::Token;
use std::{fs, path::Path};

mod common {
    include!("../common/mod.rs");
}
use common::lexer::LexerTestHarness;
#[test]
fn test_operators_snap() {
    LexerTestHarness::from_file("tests/fixtures/lexical/operators.loi")
        .assert_snapshot("operators_lexical_stream");
}

#[test]
fn test_operator_logic() {
    let content = "+ - * / == != < > <= >= && || ! =";
    let tokens = lex(content).unwrap();
    let ops: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::EOF)).collect();

    assert_eq!(ops.len(), 14, "Expected exactly 14 operator tokens");

    assert!(ops.contains(&&Token::Eq));
    assert!(ops.contains(&&Token::Minus));
    assert!(ops.contains(&&Token::Star));
    assert!(ops.contains(&&Token::Slash));
    assert!(ops.contains(&&Token::Eq));
    assert!(ops.contains(&&Token::Neq));
    assert!(ops.contains(&&Token::Lt));
    assert!(ops.contains(&&Token::Gt));
    assert!(ops.contains(&&Token::Le));
    assert!(ops.contains(&&Token::Ge));
    assert!(ops.contains(&&Token::And));
    assert!(ops.contains(&&Token::Or));
    assert!(ops.contains(&&Token::Not));
    assert!(ops.contains(&&Token::Eq));
}
