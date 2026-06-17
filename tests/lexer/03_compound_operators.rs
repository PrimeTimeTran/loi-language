use loi::frontend::lexer::lex;
use loi::frontend::token::Token;

mod common {
    include!("../00_common/mod.rs");
}
use common::lexer::LexerTestHarness;
use loi::tok;

#[test]
fn parses_assign_not() {
    LexerTestHarness::new("x =! 10").assert_tokens(vec![
        tok!(ident "x"),
        Token::Immutable,
        tok!(num 10),
    ]);
}

#[test]
fn parses_assign_maybe() {
    LexerTestHarness::new("x =? 10").assert_tokens(vec![
        tok!(ident "x"),
        Token::Dynamic,
        tok!(num 10),
    ]);
}

#[test]
fn does_not_split_compound_assign() {
    let h = LexerTestHarness::new("x =! 10");

    h.assert_contains(Token::Immutable);

    h.assert_no_tokens_of_type(|t| matches!(t, Token::Eq | Token::Neq));
}

#[test]
fn operator_boundary_is_correct() {
    LexerTestHarness::new("a =!b").assert_tokens(vec![
        tok!(ident "a"),
        Token::Immutable,
        tok!(ident "b"),
    ]);
}

#[test]
fn invalid_split_is_not_allowed() {
    let tokens = lex("x =! 10").expect("lex failed");

    assert!(tokens.contains(&Token::Immutable));
    assert!(!tokens.iter().any(|t| matches!(t, Token::Eq)));
}
