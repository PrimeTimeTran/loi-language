use loi::frontend::lexer::lex;
use loi::frontend::token::Token;
use std::{fs, path::Path};

#[test]
fn test_operators_snap() {
    let path = Path::new("tests/fixtures/lexical/operators.loi");
    let content = fs::read_to_string(path).expect("Failed to read fixture");
    let tokens = lex(&content).expect("Lexer failed");

    insta::assert_debug_snapshot!(tokens);
}

#[test]
fn test_operator_logic() {
    let content = "+ - * / == != < > <= >= && || ! =";
    let tokens = lex(content).unwrap();
    let ops: Vec<_> = tokens.iter().filter(|t| !matches!(t, Token::EOF)).collect();

    // println!("test_operator_logic {:?}", tokens);
    assert_eq!(ops.len(), 14, "Expected exactly 14 operator tokens");

    assert!(ops.contains(&&Token::EqCheck));
    assert!(ops.contains(&&Token::Minus));
    assert!(ops.contains(&&Token::Star));
    assert!(ops.contains(&&Token::Slash));
    assert!(ops.contains(&&Token::EqCheck));
    assert!(ops.contains(&&Token::Neq));
    assert!(ops.contains(&&Token::Lt));
    assert!(ops.contains(&&Token::Gt));
    assert!(ops.contains(&&Token::Le));
    assert!(ops.contains(&&Token::Ge));
    assert!(ops.contains(&&Token::BoolAnd));
    assert!(ops.contains(&&Token::BoolOr));
    assert!(ops.contains(&&Token::Not));
    assert!(ops.contains(&&Token::Equals));
}
